import { computed, effect, Signal, signal } from "@preact/signals";

import { hide } from "tauri-plugin-spotlight-api";
import { writeText } from "@tauri-apps/api/clipboard";
import { invoke } from "@tauri-apps/api/tauri";

import { Item, LoadedItem, Stack } from "./types";

// TODO: cap size of CAS, with MRU eviction
const CAS: Map<string, string> = new Map();

export async function getContent(hash: string): Promise<string> {
  const cachedItem = CAS.get(hash);
  if (cachedItem !== undefined) {
    return cachedItem;
  }
  console.log("CACHE MISS", hash);
  const content: string = await invoke("store_get_content", { hash: hash });
  CAS.set(hash, content);
  return content;
}

export const createStack = (items: Signal<Item[]>, parent?: Stack): Stack => {
  const selected = signal(0);

  const loadedHash: Signal<string> = signal("");
  const loadedContent: Signal<string> = signal("");

  const selectedContent = computed((): string | undefined => {
    const item = items.value[normalizedSelected.value];
    if (item === undefined) return undefined;
    if (item.hash !== loadedHash.value) return undefined;
    return loadedContent.value;
  });

  const loaded = computed((): LoadedItem | undefined => {
    const item = items.value[normalizedSelected.value];
    if (item === undefined) return undefined;
    const content = selectedContent.value;
    if (content === undefined) return undefined;
    return {
      item,
      content,
    };
  });

  const normalizedSelected = computed(() => {
    let val = selected.value % (items.value.length);
    if (val < 0) val = items.value.length + val;
    return val;
  });

  async function updateLoaded(hash: string) {
    loadedContent.value = await getContent(hash);
    loadedHash.value = hash;
  }

  effect(() => {
    const item = items.value[normalizedSelected.value];
    if (item === undefined) return undefined;
    if (item.hash != loadedHash.value) {
      updateLoaded(item.hash);
    }
  });

  const parents = parent ? [parent, ...parent.parents] : [];

  return {
    items,
    selected,
    normalizedSelected,
    loaded,
    parents,
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
