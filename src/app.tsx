import { render } from "preact";
import { computed, effect, Signal, signal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

import { Event, listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";
import { writeText } from "@tauri-apps/api/clipboard";

import { hide } from "tauri-plugin-spotlight-api";

import { Icon } from "./icons.tsx";
import { StatusBar } from "./statusbar.tsx";
import { MetaPanel } from "./meta.tsx";

import { Item } from "./types.tsx";

import {
  borderBottom,
  borderRight,
  darkThemeClass,
  lightThemeClass,
} from "./app.css.ts";

//
// Global State

const themeMode = signal("light");

const items = signal<Item[]>([]);
const selected = signal(0);

const availableItems = computed(() => {
  return items.value;
  /*
  const ret = Array.from(items.value.values())
    .filter((item) => {
      const filter = currentFilter.value.trim().toLowerCase();
      if (filter === "") return true;
      return item.terse.toLowerCase().includes(filter);
    })
    .sort((a, b) => cmp(b.id, a.id));
  return ret;
  */
});

const selectedItem = computed((): Item | undefined => {
  return availableItems.value[selected.value];
});

const loadedContent: Signal<string> = signal("");
const loadedHash: Signal<string> = signal("");

const selectedContent = computed((): string | undefined => {
  const item = selectedItem.value;
  if (item === undefined) return undefined;
  if (item.hash !== loadedHash.value) return undefined;
  return loadedContent.value;
});

// TODO: cap size of CAS, with MRU eviction
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

async function updateFilter(curr: string) {
  items.value = await invoke<Item[]>("store_set_filter", { curr: curr });
}

const showFilter = signal(false);
const currentFilter = signal("");

effect(() => {
  if (!showFilter.value) currentFilter.value = "";
});

effect(() => {
  const curr = currentFilter.value;
  updateFilter(curr);
});

//

let focusSelectedTimeout: number | undefined;

function focusSelected(delay: number) {
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

async function updateSelected(n: number) {
  if (availableItems.value.length === 0) return;
  selected.value = (selected.value + n) % availableItems.value.length;
  if (selected.value < 0) {
    selected.value = availableItems.value.length + selected.value;
  }
  focusSelected(5);
}

function FilterInput() {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (inputRef.current != null) {
      inputRef.current.focus();
    }
  }, []);

  return (
    <div
      className={borderBottom}
      style="
        padding:1ch;
        padding-left:2ch;
        padding-right:2ch;
        padding-bottom:0.5ch;
        display: flex;
    width: 100%;
        align-items: center;
        "
    >
      <div>/</div>
      <div style="width: 100%">
        <input
          type="text"
          placeholder="Type a filter..."
          ref={inputRef}
          onInput={() => {
            if (inputRef.current == null) return;
            currentFilter.value = inputRef.current.value;
            // updateSelected(0);
          }}
        />
      </div>
    </div>
  );
}

function LeftPane() {
  const TerseRow = ({ item, index }: { item: Item; index: number }) => (
    <div
      className={"terserow" + (index === selected.value ? " selected" : "")}
      onClick={() => selected.value = index}
      style="
        display: flex;
        width: 100%;
        gap: 0.5ch;
        overflow: hidden;
        padding: 0.5ch 0.75ch;
        border-radius: 6px;
        cursor: pointer;
        "
    >
      <div
        style={{
          flexShrink: 0,
          width: "2ch",
          whiteSpace: "nowrap",
          overflow: "hidden",
        }}
      >
        {item.link ? <img src={item.link.icon} /> : (
          <Icon
            name={item.mime_type == "image/png" ? "IconImage" : "IconClipboard"}
          />
        )}
      </div>

      <div
        style={{
          flexGrow: 1,
          whiteSpace: "nowrap",
          overflow: "hidden",
          textOverflow: "ellipsis",
        }}
      >
        {item.terse}
      </div>
    </div>
  );
  return (
    <div
      className={borderRight}
      style="
      flex: 1;
      max-width: 20ch;
      overflow-y: auto;
      padding-right: 0.5rem;
    "
    >
      {availableItems.value
        .map((item, index) => {
          return <TerseRow item={item} index={index} />;
        })}
    </div>
  );
}

function RightPane(
  { item, content }: {
    item: Item | undefined;
    content: string | undefined;
  },
) {
  if (!item) {
    return <div />;
  }

  function Preview(
    { item, content }: { item: Item; content: string | undefined },
  ) {
    if (content !== undefined && item.mime_type === "image/png") {
      return (
        <img
          src={"data:image/png;base64," + content}
          style={{
            opacity: 0.95,
            borderRadius: "0.5rem",
            maxHeight: "100%",
            height: "auto",
            width: "auto",
            objectFit: "contain",
          }}
        />
      );
    }

    if (item.link) {
      return (
        <img
          src={item.link.screenshot}
          style={{
            opacity: 0.95,
            borderRadius: "0.5rem",
            maxHeight: "100%",
            height: "auto",
            width: "auto",
            objectFit: "contain",
          }}
        />
      );
    }

    return (
      <pre style="margin: 0; white-space: pre-wrap; overflow-x: hidden">
    { content !== undefined ? content : "loading..." }
      </pre>
    );
  }

  return (
    <div style="flex: 3; overflow: auto; height: 100%">
      <Preview item={item} content={content} />
    </div>
  );
}

async function triggerCopy() {
  const item = selectedItem.value;
  if (item) {
    if (item.mime_type != "text/plain") {
      console.log("MIEM", item.mime_type);
    } else {
      let content = await getContent(item.hash);
      await writeText(content);
    }
  }
  showFilter.value = false;
  hide();
}

async function triggerDelete() {
  const item = selectedItem.value;
  if (item) {
    items.value = await invoke<Item[]>("store_delete", { hash: item.hash });
  }
}

async function globalKeyHandler(event: KeyboardEvent) {
  switch (true) {
    case event.key === "Enter":
      await triggerCopy();
      break;

    case event.key === "Escape":
      event.preventDefault();

      if (showFilter.value) {
        showFilter.value = false;
        return;
      }
      hide();
      return;

    case event.key === "Tab":
      event.preventDefault();
      // await invoke("open_docs");
      break;

    case ((!showFilter.value) && event.key === "/"):
      event.preventDefault();
      showFilter.value = true;
      break;

    case (event.ctrlKey && event.key === "n") || event.key === "ArrowDown":
      event.preventDefault();
      updateSelected(1);
      break;

    case event.ctrlKey && event.key === "p" || event.key === "ArrowUp":
      event.preventDefault();
      updateSelected(-1);
      break;

    case (event.ctrlKey && event.key === "Backspace"):
      event.preventDefault();
      await triggerDelete();
      break;
  }
}

function Main() {
  useEffect(() => {
    window.addEventListener("keydown", globalKeyHandler);
    return () => {
      window.removeEventListener("keydown", globalKeyHandler);
    };
  }, []);

  return (
    <main
      className={themeMode.value === "light" ? lightThemeClass : darkThemeClass}
    >
      {showFilter.value && <FilterInput />}
      <section style="
            display: flex;
            flex-direction: column;
            height: 100%;
            width: 100%;
            overflow: hidden;
            padding-top:1ch;
            padding-left:1ch;
            padding-right:1ch;
            position: relative;
        ">
        <div style="display: flex; height: 100%; overflow: hidden; gap: 0.5ch;">
          <LeftPane />
          <RightPane
            item={selectedItem.value}
            content={selectedContent.value}
          />
        </div>

        {selectedItem.value &&
          <MetaPanel item={selectedItem.value} />}
      </section>
      <StatusBar
        themeMode={themeMode}
        showFilter={showFilter}
        triggerCopy={triggerCopy}
        triggerDelete={triggerDelete}
      />
    </main>
  );
}

function App() {
  useEffect(() => {
    listen("recent-items", (event: Event<Item[]>) => {
      console.log("Data pushed from Rust:", event);
      items.value = event.payload;
      updateSelected(0);
    });

    async function init() {
      items.value = await invoke<Item[]>("init_window");
      updateSelected(0);
    }
    init();

    // set selection back to the top onBlur
    const onBlur = () => {
      selected.value = 0;
    };
    const onFocus = () => {
      focusSelected(100);
    };

    window.addEventListener("blur", onBlur);
    window.addEventListener("focus", onFocus);

    return () => {
      window.removeEventListener("blur", onBlur);
      window.removeEventListener("focus", onFocus);
    };
  }, []);

  return <Main />;
}

render(<App />, document.body);
