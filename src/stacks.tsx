import { computed, effect, Signal, signal } from "@preact/signals";

import { hide } from "tauri-plugin-spotlight-api";
import { listen } from "@tauri-apps/api/event";
import { writeText } from "@tauri-apps/api/clipboard";
import { invoke } from "@tauri-apps/api/tauri";

import { Item, Stack } from "./types";

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

export const createStack = (
  initItems?: Record<string, Item>,
  parent?: Stack,
): Stack => {
  const filter = createFilter();
  const items = signal(
    initItems
      ? Object.values(initItems).sort((a, b) => {
        const lastIdA = a.ids[a.ids.length - 1];
        const lastIdB = b.ids[b.ids.length - 1];
        if (lastIdA < lastIdB) return 1;
        if (lastIdA > lastIdB) return -1;
        return 0;
      })
      : [],
  );
  const selected = signal(0);

  const normalizedSelected = computed(() => {
    let val = selected.value % (items.value.length);
    if (val < 0) val = items.value.length + val;
    return val;
  });

  const item = computed((): Item | undefined =>
    items.value[normalizedSelected.value]
  );

  return {
    filter,
    items,
    selected,
    normalizedSelected,
    item,
    get content(): undefined | Signal<string | undefined> {
      if (item.value) {
        return CAS.getSignal(item.value.hash);
      }
      return undefined;
    },
    parent,
  };
};

const root = createStack();
export const currStack: Signal<Stack> = signal(root);

//
// Wire filter, and server refresh notifications, to update the current stacks
// items
const updateItems = async (stack: Stack) => {
  const filter = stack.filter.curr.value;
  const contentType = stack.filter.content_type.value;

  const args= {
    filter: filter,
    contentType: contentType,
    stack: stack.parent?.item.value?.hash,
  };

  const curr = stack.item.peek()?.terse;
  stack.items.value = await invoke<Item[]>("store_list_items", args);

  const index = stack.items.peek().findIndex((item) => item.terse == curr);
  console.log("updateItems: Refocus:", curr, index);
  if (index >= 0) stack.selected.value = index;
};

let d1: (() => void) | undefined;

async function initRefresh() {
  d1 = await listen("refresh-items", () => {
    console.log("LISTEN");
    updateItems(currStack.value);
    const parent = currStack.value.parent;
    if (parent) {
      updateItems(parent);
    }
  });
}
initRefresh();

effect(() => {
  console.log("EFFECT");
  updateItems(currStack.value);
});
// End items

export async function triggerCopy() {
  const item = currStack.value.item.value;
  if (!item) return;
  const content = currStack.value.content?.value;
  if (!content) return;

  if (item.mime_type != "text/plain") {
    console.log("MIEM", item.mime_type);
  } else {
    await writeText(content);
  }
  hide();
}

if (import.meta.hot) {
  import.meta.hot.accept(() => {});
  import.meta.hot.dispose(() => {
    if (d1) d1();
  });
}
