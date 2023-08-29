import { effect, Signal, signal } from "@preact/signals";

import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";

import { Nav, Stack } from "./types";

export const currStack: Signal<Stack | null> = signal(null);

invoke<Nav>("store_list_items", { filter: "", contentType: "" }).then(
  (nav) => {
    currStack.value = new Stack(nav);
  },
);

let d1: (() => void) | undefined;

async function initRefresh() {
    console.log("CREATE D1");
  d1 = await listen("refresh-items", () => {
    if (!currStack.value) return;
    const stack = currStack.value;
    stack.refreshNav();
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
  stack.refreshNav();
});

if (import.meta.hot) {
  import.meta.hot.accept(() => {});
  import.meta.hot.dispose(() => {
    console.log("DISPOSE D1");
    if (d1) d1();
  });
}
