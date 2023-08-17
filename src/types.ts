import { JSXInternal } from "preact/src/jsx";

import { computed, Signal, signal } from "@preact/signals";

import { invoke } from "@tauri-apps/api/tauri";

const Scru128IdBrand = Symbol("Scru128Id");
export type Scru128Id = string & { readonly brand: typeof Scru128IdBrand };
const SSRIBrand = Symbol("SSRI");
export type SSRI = string & { readonly brand: typeof SSRIBrand };

export interface Item {
  id: Scru128Id;
  last_touched: string;
  touched: string[];
  hash: SSRI;
  stack_id: Scru128Id | null;
  children: Scru128Id[];
}

export interface ContentMeta {
  hash: string | null;
  mime_type: string;
  content_type: string;
  terse: string;
  tiktokens: number;
}

export interface State {
  root: Scru128Id[];
  items: { [id: string]: Item };
  content_meta: { [key: string]: ContentMeta };
  matches: string[];
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

  /*
  isFocusFirst(): boolean {
    return this.type === FocusType.FIRST;
  }
  */

  curr(stack: Stack) {
    if (!this.id || this.type === FocusType.FIRST) {
      return stack.state.value.root[0];
    }
    return this.id;
  }
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

export class Stack {
  filter: {
    curr: Signal<string>;
    content_type: Signal<string>;
    dirty: () => boolean;
    clear: () => void;
  };
  state: Signal<State>;
  selected: Signal<Focus>;
  normalizedSelected: Signal<string>;
  item: Signal<Item | undefined>;

  constructor(initialState: State) {
    this.state = signal(initialState);
    this.filter = createFilter();
    this.selected = signal(Focus.first());
    this.normalizedSelected = signal("");

    this.item = computed((): Item | undefined => {
      return this.state.value.items[this.selected.value.curr(this)];
    });
  }

  getContent(hash: SSRI): Signal<string | undefined> {
    return CAS.getSignal(hash);
  }

  getContentMeta(item: Item): ContentMeta {
    return this.state.value.content_meta[item.hash];
  }

  selectUp(): void {
    const currentItem = this.state.value.items[this.selected.value.curr(this)];
    const peers = this.getPeers(currentItem);
    const currentIndex = peers.indexOf(currentItem.id);
    if (currentIndex > 0) {
      this.selected.value = Focus.id(peers[currentIndex - 1]);
    }
  }

  selectDown(): void {
    const currentItem = this.state.value.items[this.selected.value.curr(this)];
    const peers = this.getPeers(currentItem);
    const currentIndex = peers.indexOf(currentItem.id);
    if (currentIndex < peers.length - 1) {
      this.selected.value = Focus.id(peers[currentIndex + 1]);
    }
  }

  selectRight(): void {
    const currentItem = this.state.value.items[this.selected.value.curr(this)];
    if (currentItem.children.length > 0) {
      this.selected.value = Focus.id(currentItem.children[0]);
    }
  }

  selectLeft(): void {
    const currentItem = this.state.value.items[this.selected.value.curr(this)];
    if (currentItem.stack_id) {
      this.selected.value = Focus.id(currentItem.stack_id);
    }
  }

  getPeers(item: Item): Scru128Id[] {
    return item.stack_id
      ? this.state.value.items[item.stack_id].children
      : this.state.value.root;
  }
}

export interface Action {
  name: string;
  keys?: (string | JSXInternal.Element)[];
  trigger?: (stack: Stack) => void;
  canApply?: (stack: Stack) => boolean;
  matchKeyEvent?: (event: KeyboardEvent) => boolean;
}
