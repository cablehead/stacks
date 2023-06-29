import { computed, effect, Signal, signal } from "@preact/signals";

import { hide } from "tauri-plugin-spotlight-api";
import { listen } from "@tauri-apps/api/event";
import { writeText } from "@tauri-apps/api/clipboard";
import { invoke } from "@tauri-apps/api/tauri";

import { Item, LoadedItem, Stack } from "./types";

import { filterContentTypeMode, mainMode } from "./modals";

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

  console.log("createStack", parent);
  // const parents = parent ? [parent, ...parent.parents] : [];

  /*
  effect(() => {
    console.log("SLECECTED CHANGED", parents.length > 0 ? parents[0].selected.value : -6000, selected.value);
  });
  */

  return {
    items,
    selected,
    normalizedSelected,
    loaded,
  };
};

export const items = signal<Item[]>([]);

const updateItems = async (filter: string, contentType: string) => {
  items.value = await invoke<Item[]>("store_list_items", {
    filter: filter,
    contentType: contentType,
  });
};

listen("refresh-items", () => {
  console.log("Data pushed from Rust");
  updateItems(
    mainMode.state.curr.value,
    filterContentTypeMode.curr.value,
  );
});

effect(() => {
  updateItems(
    mainMode.state.curr.value,
    filterContentTypeMode.curr.value,
  );
});

const root = createStack(items);
export const currStack = root;

export async function triggerCopy() {
  const loaded = currStack.loaded.value;
  if (!loaded) return;

  if (loaded.item.mime_type != "text/plain") {
    console.log("MIEM", loaded.item.mime_type);
  } else {
    await writeText(loaded.content);
  }
  hide();
}
