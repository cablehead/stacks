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

/*
export interface Stack {
  items: Signal<Item[]>;
  selected: Signal<number>;
  normalizedSelected: Signal<number>;
  selectedItem: Signal<Item | undefined>;
}
*/

export const createStack = (items: Signal<Item[]>): Stack => {
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
  };
};

//
// Wire items up to the filter, and server refresh notifications
const items = signal<Item[]>([]);

const updateItems = async (filter: string, contentType: string) => {
  items.value = await invoke<Item[]>("store_list_items", {
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

export const currStack: Stack = createStack(items);

export async function triggerCopy() {
  const item = currStack.item.value;
  if (!item) return;
  const content = currStack.content?.value;
  if (!content) return;

  if (item.mime_type != "text/plain") {
    console.log("MIEM", item.mime_type);
  } else {
    await writeText(content);
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
