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

function debounce<T>(
  this: T,
  func: (...args: any[]) => void,
  delay: number,
): (...args: any[]) => void {
  let debounceTimer: NodeJS.Timeout;
  return function (this: T, ...args: any[]) {
    const context = this;
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => func.apply(context, args), delay);
  };
}

// refreshNav maintains the provided stack's items: reactively, based on the
// stack's current filter and content type
const innerRefreshNav = async (stack: Stack) => {
  const filter = stack.filter.curr.value;
  const contentType = stack.filter.content_type.value;

  const args = {
    filter: filter,
    contentType: contentType,
    focusedId: stack.focused_id,
    // Include the hash of the focused parent tack item, if it exists
    // stack: stack.parent?.item.value?.hash,
  };

  // Get the hash of the currently focused item
  // const currItem = stack.item.peek()?.hash;

  // Set the new list of items from the backend
  const nav = await invoke<Nav>("store_list_items", args);
  // if (state.matches) state.matches = new Set(state.matches);
  stack.nav.value = nav;

  /*
    const selectedId = stack.selected.value.curr(stack);
    const selected = stack.state.value.items[selectedId];
    console.log("UPDATE", selectedId, selected);
    if (selected) return;

    const last = stack.lastKnown;
    if (!last) return;

    const peers = stack.getPeers(last);
    console.log("PEERS", peers);
    let next = peers.find((id) => id < last.id) || peers[peers.length - 1] ||
      last.stack_id ||
      stack.state.value.root[0];
    console.log("NEXT", last.id, next);
    if (next) stack.select(next);
    */
};

const refreshNav = debounce(innerRefreshNav, 50);

export class Stack {
  filter: {
    curr: Signal<string>;
    content_type: Signal<string>;
    dirty: () => boolean;
    clear: () => void;
  };

  nav: Signal<Nav>;

  focused_id: string | null;

  constructor(nav: Nav) {
      console.log("CONSTRUCT");
    this.filter = createFilter();
    this.nav = signal(nav);
    this.focused_id = null;
  }

  refreshNav() {
    refreshNav(this);
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

  async selectDown() {
    const args = {
      filter: "",
      contentType: "",
      focusedId: this.focused_id,
    };
    const nav = await invoke<Nav>("store_select_down", args);
    this.nav.value = nav;
  }

  selectRight(): void {
  }

  selectLeft(): void {
  }

  select(id: string): void {
    this.focused_id = id;
    this.refreshNav();
  }
}

export interface Action {
  name: string;
  keys?: (string | JSXInternal.Element)[];
  trigger?: (stack: Stack) => void;
  canApply?: (stack: Stack) => boolean;
  matchKeyEvent?: (event: KeyboardEvent) => boolean;
}
