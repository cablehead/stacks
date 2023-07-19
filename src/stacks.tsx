import { computed, effect, Signal, signal } from "@preact/signals";

import { hide } from "tauri-plugin-spotlight-api";
import { listen } from "@tauri-apps/api/event";
import { writeText } from "@tauri-apps/api/clipboard";
import { invoke } from "@tauri-apps/api/tauri";

import { Focus, Item, Stack } from "./types";

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
  const selected = signal(Focus.first());

  const normalizedSelected = computed(() => {
    let val = selected.value.currIndex() % (items.value.length);
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
  if (stack.parent) {
    innerUpdateItems(stack.parent);
  }

  const filter = stack.filter.curr.value;
  const contentType = stack.filter.content_type.value;

  const args = {
    filter: filter,
    contentType: contentType,
    // Include the hash of the focused parent stack item, if it exists
    stack: stack.parent?.item.value?.hash,
  };

  // Get the hash of the currently focused item
  const currItem = stack.item.peek()?.hash;

  // Set the new list of items from the backend
  stack.items.value = await invoke<Item[]>("store_list_items", args);

  // If the app doesn't currently have focus, focus the first (newly touched)
  // item in the stack, or if stack.selected.value is the focus first sentinel
  if (!document.hasFocus() || stack.selected.value.isFocusFirst()) {
    stack.selected.value = Focus.index(0);
    return;
  }

  // If the app does have focus, try to find the previously focused item, in
  // order to preserve focus
  const index = stack.items.peek().findIndex((item) => item.hash == currItem);
  console.log("updateItems: Refocus:", currItem, index, stack.selected.value);
  if (index >= 0) stack.selected.value = Focus.index(index);
};

const updateItems = debounce(innerUpdateItems, 50);

let d1: (() => void) | undefined;

async function initRefresh() {
  d1 = await listen("refresh-items", () => {
    updateItems(currStack.value);
  });
}
initRefresh();

effect(() => {
  const stack = currStack.value;
  // Depend on the current filter and content type from the stack, so we react
  // to filter changes
  console.log(
    "currStack: updateItems",
    stack.filter.curr.value,
    stack.filter.content_type.value,
  );
  updateItems(stack);
});
// End items

effect(() => {
  console.log("SELECTED:", currStack.value.selected.value);
});

export async function triggerCopy() {
  const item = currStack.value.item.value;
  if (!item) return;
  await invoke("store_copy_to_clipboard", {
    sourceId: item.ids[0],
    stackHash: currStack.value.parent?.item.value?.hash,
  });
  hide();
}

export async function triggerCopyEntireStack(stack: Stack) {
  const item = stack.item.value;
  if (item) {
    const sortedStackItems = Object.values(item.stack).sort((a, b) => {
      const lastIdA = a.ids[a.ids.length - 1];
      const lastIdB = b.ids[b.ids.length - 1];
      return lastIdB.localeCompare(lastIdA);
    });
    const contents = await Promise.all(
      sortedStackItems
        .filter((item) => item.mime_type === "text/plain")
        .map((item) => CAS.get(item.hash)),
    );
    const entireString = contents.join("\n");
    await writeText(entireString);
  }
}

if (import.meta.hot) {
  import.meta.hot.accept(() => {});
  import.meta.hot.dispose(() => {
    if (d1) d1();
  });
}
