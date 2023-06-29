import { computed, effect, Signal, signal } from "@preact/signals";
import { hide } from "tauri-plugin-spotlight-api";
import { listen } from "@tauri-apps/api/event";
import { writeText } from "@tauri-apps/api/clipboard";
import { invoke } from "@tauri-apps/api/tauri";
import { Item, LoadedItem } from "./types";
import { filterContentTypeMode, mainMode } from "./modals";

const CAS: Map<string, string> = new Map();

async function getContent(hash: string): Promise<string> {
  const cachedItem = CAS.get(hash);
  if (cachedItem !== undefined) {
    return cachedItem;
  }
  console.log("CACHE MISS", hash);
  const content: string = await invoke("store_get_content", { hash: hash });
  CAS.set(hash, content);
  return content;
}

const items = signal<Item[]>([]);
const selected = signal(0);
const loadedHash: Signal<string> = signal("");
const loadedContent: Signal<string> = signal("");

const normalizedSelected = computed(() => {
  let val = selected.value % (items.value.length);
  if (val < 0) val = items.value.length + val;
  return val;
});

const selectedContent = computed((): string | undefined => {
  const item = items.value[normalizedSelected.value];
  if (item === undefined || item.hash !== loadedHash.value) return undefined;
  return loadedContent.value;
});

const loaded = computed((): LoadedItem | undefined => {
  const item = items.value[normalizedSelected.value];
  const content = selectedContent.value;
  if (item === undefined || content === undefined) return undefined;
  return { item, content };
});

async function updateLoaded(hash: string) {
  loadedContent.value = await getContent(hash);
  loadedHash.value = hash;
}

effect(() => {
  const item = items.value[normalizedSelected.value];
  if (item && item.hash != loadedHash.value) {
  console.log("EFFECT", "updateLoaded", "proceed");
    updateLoaded(item.hash);
  }
});

const updateItems = async (filter: string, contentType: string) => {
  items.value = await invoke<Item[]>("store_list_items", {
    filter: filter,
    contentType: contentType,
  });
};

const d1 = await listen("refresh-items", () => {
  console.log("Data pushed from Rust");
  updateItems(mainMode.state.curr.value, filterContentTypeMode.curr.value);
});
console.log("init my listen", d1);

effect(() => {
  console.log("EFFECT", "updateItems");
  updateItems(mainMode.state.curr.value, filterContentTypeMode.curr.value);
});

export const currStack = {
  items,
  selected,
  normalizedSelected,
  loaded,
};

export async function triggerCopy() {
  const loaded = currStack.loaded.value;
  if (!loaded) return;

  if (loaded.item.mime_type != "text/plain") {
    console.log("MIEM", loaded.item.mime_type);
  } else {
    await writeText(loaded.content);
  }
  currStack.selected.value = 0;
  hide();
}

if (import.meta.hot) {
  console.log("HOT");
  import.meta.hot.accept(() => {
    console.log("ACCEPT");
  });
  import.meta.hot.dispose(() => {
    console.log("DISPOSE4");
    d1();
  });
}
