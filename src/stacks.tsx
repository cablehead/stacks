import { effect, Signal, signal } from "@preact/signals";

import { hide } from "tauri-plugin-spotlight-api";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";

import { Stack} from "./types";

export const currStack: Signal<Stack | null> = signal(null);

invoke<string>("store_list_items", {filter: "", contentType: ""}).then((state) => {
  currStack.value = new Stack(JSON.parse(state));
});

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
    // Include the hash of the focused parent tack item, if it exists
    // stack: stack.parent?.item.value?.hash,
  };

  // Get the hash of the currently focused item
  // const currItem = stack.item.peek()?.hash;

  // Set the new list of items from the backend
  stack.state.value = JSON.parse(await invoke<string>("store_list_items", args));
  console.log("store_list_items", stack.state.value);
};

const updateItems = debounce(innerUpdateItems, 50);

let d1: (() => void) | undefined;

async function initRefresh() {
  if (!currStack.value) return;
  const stack = currStack.value;
  d1 = await listen("refresh-items", () => {
    updateItems(stack);
  });
}
initRefresh();

effect(() => {
  if (!currStack.value) return;
  const stack = currStack.value;

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
