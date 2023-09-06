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
  hash?: SSRI;
  mime_type: string;
  content_type: string;
  terse: string;
  tiktokens: number;
}

export function itemGetContent(item: Item): Signal<string | undefined> {
  return item.hash ? CAS.getSignal(item.hash) : signal("b2ggaGFpCg==");
}

export function itemGetTerse(item: Item): string {
  return item.hash ? item.terse : "oh hai";
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
    console.log("CONSTRUCT");
    this.filter = createFilter();
    this.nav = signal(nav);

    this.initListener();
    effect(() => {
      this.set_filter(this.filter.curr.value, this.filter.content_type.value);
    });
  }

  async initListener() {
    console.log("CREATE D1");
    const d1 = await listen("refresh-items", () => {
      console.log('listen("refresh-items');
      this.refresh();
    });
    const d2 = await listen("foo", (payload) => {
      console.log('listen("foo', payload);
    });
    if (import.meta.hot) {
      import.meta.hot.dispose(() => {
        console.log("DISPOSE");
        if (d1) d1();
        if (d2) d2();
      });
    }
  }

  selected(): Item | undefined {
    const nav = this.nav.value;
    if (nav.sub && nav.sub.is_focus) return nav.sub.selected;
    return nav.root?.selected;
  }

  async getRoot(): Promise<Item[]> {
    return await invoke<Item[]>("store_get_root", {});
  }

  async undo() {
    await invoke<Item[]>("store_undo", {});
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
}

export interface Action {
  name: string;
  keys?: (string | JSXInternal.Element)[];
  trigger?: (stack: Stack) => void;
  canApply?: (stack: Stack) => boolean;
  matchKeyEvent?: (event: KeyboardEvent) => boolean;
}

export const CAS = (() => {
  const signalCache: Map<string, Signal<string>> = new Map();

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
