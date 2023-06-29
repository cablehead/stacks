import { computed, Signal, signal } from "@preact/signals";

import { hide } from "tauri-plugin-spotlight-api";
import { writeText } from "@tauri-apps/api/clipboard";

import { Item, Stack } from "./types";

export const createStack = (items: Signal<Item[]>): Stack => {
  const selected = signal(0);
  const loaded = signal(undefined);

  const normalizedSelected = computed(() => {
    let val = selected.value % (items.value.length);
    if (val < 0) val = items.value.length + val;
    return val;
  });

  return {
    items,
    selected,
    normalizedSelected,
    loaded,
    parents: [],
  };
};

const items = signal<Item[]>([]);
const root = createStack(items);

export const stack = signal(root);

export async function triggerCopy() {
  const loaded = stack.value.loaded.value;
  if (!loaded) return;

  if (loaded.item.mime_type != "text/plain") {
    console.log("MIEM", loaded.item.mime_type);
  } else {
    await writeText(loaded.content);
  }
  hide();
}

/*
export const selectedItem = computed((): Item | undefined => {
  return stack.items.value[stack.selected.value];
});


const loadedContent: Signal<string> = signal("");
const loadedHash: Signal<string> = signal("");

export const selectedContent = computed((): string | undefined => {
  const item = selectedItem.value;
  if (item === undefined) return undefined;
  if (item.hash !== loadedHash.value) return undefined;
  return loadedContent.value;
});

export const loadedItem = computed((): LoadedItem | undefined => {
  const item = selectedItem.value;
  if (item === undefined) return undefined;
  const content = selectedContent.value;
  if (content === undefined) return undefined;
  return {
    item,
    content,
  };
});

async function updateLoaded(hash: string) {
  loadedContent.value = await getContent(hash);
  loadedHash.value = hash;
}

effect(() => {
  const item = selectedItem.value;
  if (item === undefined) return;
  if (item.hash != loadedHash.value) {
    updateLoaded(item.hash);
  }
});

export async function updateSelected(n: number) {
  if (stack.items.value.length === 0) return;
  stack.selected.value = (stack.selected.value + n) % stack.items.value.length;
  if (stack.selected.value < 0) {
    stack.selected.value = stack.items.value.length + stack.selected.value;
  }
  focusSelected(5);
}
*/
