import { JSXInternal } from "preact/src/jsx";

import { effect, Signal, signal } from "@preact/signals";

import { hide } from "tauri-plugin-spotlight-api";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";

import { Item, State, ContentMeta } from "./types";

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
  selected: Signal<string>;
  normalizedSelected: Signal<string>;
  item: Signal<Item | undefined>;

  constructor(initialState: State) {
    this.state = signal(initialState);
    this.filter = createFilter();
    this.selected = signal("");
    this.normalizedSelected = signal("");
    this.item = signal(undefined);
  }

  get content(): undefined | Signal<string | undefined> {
    if (this.item.value) {
      return CAS.getSignal(this.item.value.hash);
    }
    return undefined;
  }

  getContentMeta(item: Item): ContentMeta {
      return this.state.value.content_meta[item.id];
  }
}

export const stack = new Stack(await invoke<State>("store_list_items", {}));

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

// updateItems maintains the provided stack's items: reactively, based on the
// stack's current filter and content type
const innerUpdateItems = async (stack: Stack) => {
  const filter = stack.filter.curr.value;
  const contentType = stack.filter.content_type.value;

  const args = {
    filter: filter,
    contentType: contentType,
    // Include the hash of the focused parent stack item, if it exists
    // stack: stack.parent?.item.value?.hash,
  };

  // Get the hash of the currently focused item
  // const currItem = stack.item.peek()?.hash;

  // Set the new list of items from the backend
  stack.state.value = await invoke<State>("store_list_items", args);
  console.log("store_list_items", stack.state.value);
};

const updateItems = debounce(innerUpdateItems, 50);

let d1: (() => void) | undefined;

async function initRefresh() {
  d1 = await listen("refresh-items", () => {
    updateItems(stack);
  });
}
initRefresh();

effect(() => {
  console.log(
    "currStack: updateItems",
    stack.filter.curr.value,
    stack.filter.content_type.value,
  );
  updateItems(stack);
});

/*
effect(() => {
  invoke("store_set_current_stack", {
    stackHash: currStack.value.parent?.item.value?.hash,
  });
});
*/

export async function triggerCopy() {
  /*
  const item = currStack.value.item.value;
  if (!item) return;
  await invoke("store_copy_to_clipboard", {
    sourceId: item.id,
    stackHash: currStack.value.parent?.item.value?.hash,
  });
  */
  hide();
}

if (import.meta.hot) {
  import.meta.hot.accept(() => {});
  import.meta.hot.dispose(() => {
    if (d1) d1();
  });
}

export interface Action {
  name: string;
  keys?: (string | JSXInternal.Element)[];
  trigger?: (stack: Stack) => void;
  canApply?: (stack: Stack) => boolean;
  matchKeyEvent?: (event: KeyboardEvent) => boolean;
}
