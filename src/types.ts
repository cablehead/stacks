import { JSXInternal } from "preact/src/jsx";

import { Signal, signal } from "@preact/signals";

import { invoke } from "@tauri-apps/api/tauri";
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
}

export interface Layer {
  items: Item[];
  selected: Item;
  is_focus: boolean;
}

export interface Nav {
  root: Layer;
  sub?: Layer;
}

/*
export interface State {
  root: Scru128Id[];
  items: { [id: string]: Item };
  content_meta: { [key: string]: ContentMeta };
  matches?: Set<SSRI>;
}

export interface Layer {
  items: ItemMeta[];
  selected: ItemMeta;
}

export interface Neo {
  root: Layer;
  sub?: Layer;
  focusedId: Scru128Id;
}

enum FocusType {
  ID,
  FIRST,
}

export class Focus {
  type: FocusType;
  id?: Scru128Id;

  constructor(type: FocusType, id?: Scru128Id) {
    this.type = type;
    this.id = id;
  }

  static first(): Focus {
    return new Focus(FocusType.FIRST);
  }

  static id(id: Scru128Id): Focus {
    return new Focus(FocusType.ID, id);
  }

  curr(stack: Stack) {
    if (!this.id || this.type === FocusType.FIRST) {
      const firstStackId = stack.state.value.root[0];
      const firstStack = stack.state.value.items[firstStackId];
      const children = stack.getChildren(firstStack);
      if (firstStack && children[0]) return children[0];
      return firstStackId;
    }
    return this.id;
  }
}
*/

export const CAS = (() => {
  const cache: Map<string, string> = new Map();
  const signalCache: Map<string, Signal<string>> = new Map();

  async function get(hash: SSRI): Promise<string> {
    const cachedItem = cache.get(hash);
    if (cachedItem !== undefined) {
      return cachedItem;
    }
    const content: string = await invoke("store_get_content", { hash: hash });
    cache.set(hash, content);
    return content;
  }

  function getSignal(hash: SSRI): Signal<string> {
    const cachedSignal = signalCache.get(hash);
    if (cachedSignal !== undefined) {
      return cachedSignal;
    }
    const ret: Signal<string> = signal("");
    (async () => {
      ret.value = await get(hash);
    })();
    signalCache.set(hash, ret);
    return ret;
  }

  return {
    get,
    getSignal,
  };
})();

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
  }

  selected(): Item {
    const nav = this.nav.value;
    if (nav.sub) return nav.sub.selected;
    return nav.root.selected;
  }

  getContent(hash: SSRI): Signal<string | undefined> {
    return CAS.getSignal(hash);
  }

  reset() {
    this.filter.clear();
    // this.selected.value = Focus.first();
    // this.lastSelected = new Map();
  }

  async triggerCopy() {
    const item = this.selected();
    if (!item) return;
    await invoke("store_copy_to_clipboard", {
      sourceId: item.id,
    });
    hide();
  }

  selectUp(): void {
  }

  selectDown(): void {
  }

  selectRight(): void {
  }

  selectLeft(): void {
  }

  select(id: string): void {
    console.log(id);
  }
}

export interface Action {
  name: string;
  keys?: (string | JSXInternal.Element)[];
  trigger?: (stack: Stack) => void;
  canApply?: (stack: Stack) => boolean;
  matchKeyEvent?: (event: KeyboardEvent) => boolean;
}
