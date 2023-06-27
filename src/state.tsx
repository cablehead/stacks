import { computed, effect, Signal, signal } from "@preact/signals";

import { writeText } from "@tauri-apps/api/clipboard";
import { invoke } from "@tauri-apps/api/tauri";
import { hide } from "tauri-plugin-spotlight-api";

interface Link {
  provider: string;
  screenshot: string;
  title: string;
  description: string;
  url: string;
  icon: string;
}

export interface Item {
  hash: string;
  ids: string[];
  mime_type: string;
  content_type: string;
  terse: string;
  link?: Link;
  stack: Item[];
}

export const themeMode = signal("light");

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

export const stack = (() => {
  const items = signal<Item[]>([]);
  const selected = signal(0);

  return {
    items,
    selected,
  };
})();

export async function triggerCopy() {
  const item = selectedItem.value;
  if (item) {
    if (item.mime_type != "text/plain") {
      console.log("MIEM", item.mime_type);
    } else {
      let content = await getContent(item.hash);
      await writeText(content);
    }
  }
  hide();
}

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

let focusSelectedTimeout: number | undefined;

export function focusSelected(delay: number) {
  if (focusSelectedTimeout !== undefined) {
    return;
  }

  focusSelectedTimeout = window.setTimeout(() => {
    focusSelectedTimeout = undefined;
    const selectedItem = document.querySelector(
      `.terserow.selected`,
    );
    if (selectedItem) {
      selectedItem.scrollIntoView({
        behavior: "smooth",
        block: "nearest",
      });
    }
  }, delay);
}

export async function updateSelected(n: number) {
  if (stack.items.value.length === 0) return;
  stack.selected.value = (stack.selected.value + n) % stack.items.value.length;
  if (stack.selected.value < 0) {
    stack.selected.value = stack.items.value.length + stack.selected.value;
  }
  focusSelected(5);
}

export const filter = (() => {
  const curr = signal("");
  let inputRef: HTMLInputElement | null = null;

  const contentType = (() => {
    const options = ["All", "Stacks", "Links", "Images"];
    const curr = signal("All");
    const selected = signal(0);
    const normalizedSelected = computed(() => {
      let val = selected.value % (options.length);
      if (val < 0) val = options.length + val;
      return val;
    });
    return {
      options,
      curr,
      selected,
      normalizedSelected,
    };
  })();

  effect(() => {
    invoke<Item[]>("store_set_filter", {
      curr: curr.value,
      contentType: contentType.curr.value,
    });
  });

  return {
    curr,
    contentType,
    get input(): HTMLInputElement | null {
      return inputRef;
    },
    set input(ref: HTMLInputElement | null) {
      inputRef = ref;
    },
  };
})();
