import { batch, effect, Signal, signal } from "@preact/signals";

import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";

import { Stack, State } from "./types";

export const currStack: Signal<Stack | null> = signal(null);

invoke<State>("store_list_items", { filter: "", contentType: "" }).then(
  (state) => {
    currStack.value = new Stack(state);
  },
);

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

  await batch(async () => {
    // Set the new list of items from the backend
    stack.state.value = await invoke<State>("store_list_items", args);

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
  });
};

const updateItems = debounce(innerUpdateItems, 50);

let d1: (() => void) | undefined;

async function initRefresh() {
  d1 = await listen("refresh-items", () => {
    if (!currStack.value) return;
    const stack = currStack.value;
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

if (import.meta.hot) {
  import.meta.hot.accept(() => {});
  import.meta.hot.dispose(() => {
    if (d1) d1();
  });
}
