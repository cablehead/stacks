import { JSXInternal } from "preact/src/jsx";

import { effect, Signal, signal } from "@preact/signals";

import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import { hide } from "tauri-plugin-spotlight-api";

const Scru128IdBrand = Symbol("Scru128Id");
export type Scru128Id = string & { readonly brand: typeof Scru128IdBrand };
const SSRIBrand = Symbol("SSRI");
export type SSRI = string & { readonly brand: typeof SSRIBrand };

export interface Item {
  id: Scru128Id;
  stack_id?: Scru128Id;
  last_touched: Scru128Id;
  touched: Scru128Id[];
  hash: SSRI;
  mime_type: string;
  content_type: string;
  terse: string;
  tiktokens: number;
  ephemeral: boolean;
  locked: boolean;
  ordered: boolean;
  cross_stream: boolean;
}

export function itemGetContent(item: Item): string {
  const ret = item.ephemeral
    ? EphemeralCAS.getSignal(item).value
    : CAS.getSignal(item.hash).value;
  return ret;
}

export interface Layer {
  items: Item[];
  selected: Item;
  is_focus: boolean;
}

export interface Nav {
  root?: Layer;
  sub?: Layer;
  undo?: Item;
}

const createFilter = () => {
  const curr = signal("");
  const content_type = signal("All");
  return {
    curr,
    content_type,
    dirty: () => curr.value != "" || content_type.value != "All",
    clear: () => {
      curr.value = "";
      content_type.value = "All";
    },
  };
};

export class Stack {
  filter: {
    curr: Signal<string>;
    content_type: Signal<string>;
    dirty: () => boolean;
    clear: () => void;
  };

  nav: Signal<Nav>;

  constructor(nav: Nav) {
    this.filter = createFilter();
    this.nav = signal(nav);

    this.initListener();
    effect(() => {
      this.set_filter(this.filter.curr.value, this.filter.content_type.value);
    });
  }

  async initListener() {
    const d1 = await listen("refresh-items", () => {
      this.refresh();
    });
    if (import.meta.hot) {
      import.meta.hot.dispose(() => {
        if (d1) d1();
      });
    }
  }

  // returns the item which is currently focused
  selected(): Item | undefined {
    const nav = this.nav.value;
    if (nav.sub && nav.sub.is_focus) return nav.sub.selected;
    return nav.root?.selected;
  }

  // returns the currently selected stack item: which may not be the current
  // focus
  selected_stack(): Item | undefined {
    return this.nav.value.root?.selected;
  }

  // returns the currently selected leaf item: which may not be the current
  // focus
  selected_item(): Item | undefined {
    return this.nav.value.sub?.selected;
  }

  async getRoot(): Promise<Item[]> {
    return await invoke<Item[]>("store_get_root", {});
  }

  async undo() {
    await invoke<Item[]>("store_undo", {});
  }

  async touch() {
    const item = this.selected();
    if (!item) return;
    await invoke<Item[]>("store_touch", {
      sourceId: item.id,
    });
  }

  async refresh() {
    this.nav.value = await invoke<Nav>("store_nav_refresh", {});
  }

  async reset() {
    this.nav.value = await invoke<Nav>("store_nav_reset", {});
    this.filter.clear();
  }

  async triggerCopy() {
    const item = this.selected();
    if (!item) return;
    await invoke("store_copy_to_clipboard", {
      sourceId: item.id,
    });
    hide();
  }

  async select(id: string) {
    this.nav.value = await invoke<Nav>("store_nav_select", { focusedId: id });
  }

  async set_filter(filter: string, contentType: string) {
    this.nav.value = await invoke<Nav>("store_nav_set_filter", {
      filter: filter,
      contentType: contentType,
    });
  }

  async selectUp() {
    this.nav.value = await invoke<Nav>("store_nav_select_up", {});
  }

  async selectDown() {
    this.nav.value = await invoke<Nav>("store_nav_select_down", {});
  }

  async selectRight() {
    this.nav.value = await invoke<Nav>("store_nav_select_right", {});
  }

  async selectLeft() {
    this.nav.value = await invoke<Nav>("store_nav_select_left", {});
  }

  async moveUp() {
    const item = this.selected();
    if (!item) return;
    await invoke("store_move_up", { sourceId: item.id });
  }

  async moveDown() {
    const item = this.selected();
    if (!item) return;
    await invoke("store_move_down", { sourceId: item.id });
  }
}

export interface Action {
  name: string;
  keys?: (string | JSXInternal.Element)[];
  trigger?: (stack: Stack) => void;
  canApply?: (stack: Stack) => boolean;
  matchKeyEvent?: (event: KeyboardEvent) => boolean;
}

export const EphemeralCAS = (() => {
  const signalCache: Map<Scru128Id, Signal<string>> = new Map();

  function getSignal(item: Item): Signal<string> {
    let ret = signalCache.get(item.id);
    if (!ret) {
      ret = new Signal("");
      signalCache.set(item.id, ret);
    }

    (async () => {
      const content = await invoke<string>("store_get_content", {
        hash: item.hash,
      });
      // TODO: content can be null: my guess is because the content hash has
      // already been replaced in the backend cache: we should fetch by
      // item.id, not hash
      if (content) {
        ret.value = content;
      }
    })();

    return ret;
  }

  return {
    getSignal,
  };
})();

export const PreviewCAS = (() => {
  const signalCache: Map<
    SSRI,
    { contentType: string; previewSignal: Signal<string> }
  > = new Map();

  function getSignal(item: Item): Signal<string> {
    const cacheEntry = signalCache.get(item.hash);

    if (cacheEntry && cacheEntry.contentType === item.content_type) {
      return cacheEntry.previewSignal;
    }

    // If the hash is not found or the content type has changed, create and cache a new signal.
    const newPreviewSignal: Signal<string> = signal("");
    signalCache.set(item.hash, {
      contentType: item.content_type,
      previewSignal: newPreviewSignal,
    });

    (async () => {
      newPreviewSignal.value = await invoke("store_get_preview", { item });
    })();

    return newPreviewSignal;
  }

  return { getSignal };
})();

export const CAS = (() => {
  const signalCache: Map<SSRI, Signal<string>> = new Map();

  function getSignal(hash: SSRI): Signal<string> {
    const cachedSignal = signalCache.get(hash);
    if (cachedSignal !== undefined) {
      return cachedSignal;
    }
    const ret: Signal<string> = signal("");
    (async () => {
      ret.value = await invoke("store_get_content", { hash: hash });
    })();
    signalCache.set(hash, ret);
    return ret;
  }

  return {
    getSignal,
  };
})();
