import { JSXInternal } from "preact/src/jsx";

import { effect, Signal, signal } from "@preact/signals";

import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";

const Scru128IdBrand = Symbol("Scru128Id");
export type Scru128Id = string & { readonly brand: typeof Scru128IdBrand };
const SSRIBrand = Symbol("SSRI");
export type SSRI = string & { readonly brand: typeof SSRIBrand };

export interface Content {
  mime_type: string;
  content_type: string;
  terse: string;
  tiktokens: number;
  words: number;
  chars: number;
  preview: string;
}

export interface Cacheable {
  id: Scru128Id;
  hash: SSRI;
  ephemeral: boolean;
}

export function getContent(item: Cacheable): Signal<Content | null> {
  if (item.ephemeral) {
    return ContentCache.byId(item.id);
  }
  ContentCache.clearId(item.id);
  return ContentCache.byHash(item.hash);
}

export const ContentCache = (() => {
  const hashCache: Map<SSRI, Signal<Content | null>> = new Map();
  const idCache: Map<Scru128Id, Signal<Content | null>> = new Map();

  function byHash(hash: SSRI): Signal<Content | null> {
    let ret = hashCache.get(hash);
    if (!ret) {
      ret = new Signal(null);
      hashCache.set(hash, ret);
      (async () => {
        ret.value = await invoke("store_get_content", {
          hash: hash,
        });
      })();
    }
    return ret;
  }

  function byId(id: Scru128Id): Signal<Content | null> {
    let ret = idCache.get(id);
    if (!ret) {
      ret = new Signal(null);
      idCache.set(id, ret);
    }
    return ret;
  }

  function clearId(id: Scru128Id) {
    idCache.delete(id);
  }

  async function initListener() {
    const d1 = await listen(
      "streaming",
      (event: { payload: [Scru128Id, Content] }) => {
        const [id, content] = event.payload;
        console.log("streaming", id, content);
        const cache = byId(id);
        cache.value = content;
      },
    );

    const d2 = await listen(
      "content",
      (event: { payload: SSRI }) => {
        const hash = event.payload;
        console.log("content", hash);
        let ret = hashCache.get(hash);
        if (ret) {
          (async () => {
            ret.value = await invoke("store_get_content", { hash: hash });
          })();
        }
      },
    );

    if (import.meta.hot) {
      import.meta.hot.dispose(() => {
        if (d1) d1();
        if (d2) d2();
      });
    }
  }
  initListener();

  return {
    byHash,
    byId,
    clearId,
  };
})();

export interface Item {
  id: Scru128Id;
  stack_id?: Scru128Id;
  name: string;
  last_touched: Scru128Id;
  touched: Scru128Id[];
  hash: SSRI;
  ephemeral: boolean;
  locked: boolean;
  ordered: boolean;
  cross_stream: boolean;
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

  // toggles the current stack's lock status
  toggleLock() {
    const curr = this.nav.value.root?.selected;
    if (!curr) return;
    const command = curr.locked ? "store_stack_unlock" : "store_stack_lock";
    invoke(command, { sourceId: curr.id });
  }

  // is the current stack locked?
  isLocked(): boolean {
    return !!this.nav.value.root?.selected.locked;
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
    await invoke("spotlight_hide");
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

  async selectDownStack() {
    this.nav.value = await invoke<Nav>("store_nav_select_down_stack", {});
  }

  async selectUpStack() {
    this.nav.value = await invoke<Nav>("store_nav_select_up_stack", {});
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
