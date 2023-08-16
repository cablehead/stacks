import { JSXInternal } from "preact/src/jsx";

import { computed, Signal, signal } from "@preact/signals";

import { invoke } from "@tauri-apps/api/tauri";

export interface Item {
  id: string;
  last_touched: string;
  touched: string[];
  hash: string;
  stack_id: string | null;
  children: string[];
}

export interface ContentMeta {
  hash: string | null;
  mime_type: string;
  content_type: string;
  terse: string;
  tiktokens: number;
}

export interface State {
  root: string[];
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
  id: string;

  constructor(type: FocusType, id: string = "") {
    this.type = type;
    this.id = id;
  }

  static first(): Focus {
    return new Focus(FocusType.FIRST);
  }

  static index(id: string): Focus {
    return new Focus(FocusType.ID, id);
  }

  /*
  isFocusFirst(): boolean {
    return this.type === FocusType.FIRST;
  }

  down() {
    if (this.type === FocusType.FIRST) {
      return Focus.index(1);
    } else if (this.type === FocusType.ID) {
      return Focus.index(this.n + 1);
    }
    return this;
  }

  up() {
    if (this.type === FocusType.FIRST) {
      return Focus.index(-1);
    } else if (this.type === FocusType.ID) {
      return Focus.index(this.n - 1);
    }
    return this;
  }
  */

  curr(stack: Stack) {
    if (this.type === FocusType.FIRST) {
      return stack.state.value.root[0];
    }
    return this.id;
  }
}

export const CAS = (() => {
  const cache: Map<string, string> = new Map();
  const signalCache: Map<string, Signal<string>> = new Map();

  async function get(hash: string): Promise<string> {
    const cachedItem = cache.get(hash);
    if (cachedItem !== undefined) {
      return cachedItem;
    }
    const content: string = await invoke("store_get_content", { hash: hash });
    cache.set(hash, content);
    return content;
  }

  function getSignal(hash: string): Signal<string> {
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

  get content(): undefined | Signal<string | undefined> {
    if (this.item.value) {
      return CAS.getSignal(this.item.value.hash);
    }
    return undefined;
  }

  getContentMeta(item: Item): ContentMeta {
    return this.state.value.content_meta[item.hash];
  }
}

export interface Action {
  name: string;
  keys?: (string | JSXInternal.Element)[];
  trigger?: (stack: Stack) => void;
  canApply?: (stack: Stack) => boolean;
  matchKeyEvent?: (event: KeyboardEvent) => boolean;
}
