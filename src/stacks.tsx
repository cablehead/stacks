import { computed, effect, Signal, signal } from "@preact/signals";
import { hide } from "tauri-plugin-spotlight-api";
import { listen } from "@tauri-apps/api/event";
import { writeText } from "@tauri-apps/api/clipboard";
import { invoke } from "@tauri-apps/api/tauri";
import { Item, Stack } from "./types";
import { filterContentTypeMode, mainMode } from "./modals";

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

export const createStack = (initItems?: Item[], parent?: Stack): Stack => {
  const items = signal(initItems || []);
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
const updateItems = async (filter: string, contentType: string) => {
  if (currStack.value != root) {
    return;
  }
  console.log("updateItems", filter, contentType);
  currStack.value.items.value = await invoke<Item[]>("store_list_items", {
    filter: filter,
    contentType: contentType,
  });
};

const d1 = await listen("refresh-items", () => {
  updateItems(mainMode.state.curr.value, filterContentTypeMode.curr.value);
});

effect(() => {
  updateItems(mainMode.state.curr.value, filterContentTypeMode.curr.value);
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
  currStack.value.selected.value = 0;
  hide();
}

if (import.meta.hot) {
  import.meta.hot.accept(() => {});
  import.meta.hot.dispose(() => {
    d1();
  });
}
