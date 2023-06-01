import { render } from "preact";
import { computed, signal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";
import { writeText } from "@tauri-apps/api/clipboard";

import { hide } from "tauri-plugin-spotlight-api";

import { Icon } from "./icons.tsx";

import {
  borderBottom,
  borderRight,
  darkThemeClass,
  footer,
  iconStyle,
  lightThemeClass,
} from "./app.css.ts";

import { addItem, Item } from "./items.tsx";

let focusSelectedTimeout: number | undefined;

function focusSelected(delay: number) {
  if (focusSelectedTimeout !== undefined) {
    clearTimeout(focusSelectedTimeout);
    focusSelectedTimeout = undefined;
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

function updateSelected(n: number) {
  if (availableItems.value.length === 0) return;
  selected.value = (selected.value + n) % availableItems.value.length;
  if (selected.value < 0) {
    selected.value = availableItems.value.length + selected.value;
  }
  focusSelected(5);
}

const selected = signal(0);
const items = signal(new Map());

const showFilter = signal(false);
const currentFilter = signal("");

const themeMode = signal("light");

function cmp(a: any, b: any) {
  if (a < b) {
    return -1;
  } else if (a > b) {
    return 1;
  } else {
    return 0;
  }
}

const availableItems = computed(() => {
  const ret = Array.from(items.value.values())
    .filter((item) => {
      const filter = currentFilter.value.trim().toLowerCase();
      if (filter === "") return true;
      return item.terse.toLowerCase().includes(filter);
    })
    .sort((a, b) => cmp(b.id, a.id));
  return ret;
});

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
            updateSelected(0);
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
        <Icon name={item.icon} />
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

function RightPane({ item }: { item: Item }) {
  if (!item) {
    return <div />;
  }

  const MetaInfoRow = ({ name, value }: { name: string; value: any }) => (
    <div style="display:flex;">
      <div
        style={{
          flexShrink: 0,
          width: "20ch",
        }}
      >
        {name}
      </div>
      <div>{value}</div>
    </div>
  );

  return (
    <div style=" flex: 3; overflow: auto; display: flex; flex-direction: column;">
      <div
        className={borderBottom}
        style="
				padding-bottom: 0.5rem;
				flex:2;
				overflow: auto;
				"
      >
        <pre style="margin: 0; white-space: pre-wrap; overflow-x: hidden">
        {item.preview}
        </pre>
      </div>
      <div style="height: 3.5lh;  font-size: 0.8rem; overflow-y: auto;">
        {item.meta.map((info) => (
          <MetaInfoRow name={info.name} value={info.value} />
        ))}
      </div>
    </div>
  );
}

async function triggerCopy() {
  const item = availableItems.value[selected.value];
  if (item) {
    await writeText(item.preview);
  }
  clearShowFilter();
  hide();
}

function triggerShowFilter() {
  showFilter.value = true;
}

function clearShowFilter() {
  currentFilter.value = "";
  showFilter.value = false;
}

function Main() {
  useEffect(() => {
    async function handleKeys(event: KeyboardEvent) {
      switch (true) {
        case event.key === "Enter":
          await triggerCopy();
          break;

        case event.key === "Escape":
          event.preventDefault();

          if (showFilter.value) {
            clearShowFilter();
            return;
          }
          hide();
          return;

        /*
            case event.key === "Enter":
              if (inputElement.value.trim() !== "") {
                await invoke("run_command", {
                  command: inputElement.value,
                });
                inputElement.value = "";
              }
              break;
                  */

        case ((!showFilter.value) && event.key === "/"):
          event.preventDefault();
          triggerShowFilter();
          break;

        case (event.ctrlKey && event.key === "n") || event.key === "ArrowDown":
          event.preventDefault();
          updateSelected(1);
          break;

        case event.ctrlKey && event.key === "p" || event.key === "ArrowUp":
          event.preventDefault();
          updateSelected(-1);
          break;
      }
    }
    window.addEventListener("keydown", handleKeys);

    return () => {
      window.removeEventListener("keydown", handleKeys);
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
        ">
        <div style="display: flex; height: 100%; overflow: hidden; gap: 0.5ch;">
          <LeftPane />
          <RightPane item={availableItems.value[selected.value]} />
        </div>
      </section>

      <footer className={footer}>
        <div style="">
          Clipboard
        </div>

        <div style="
    display: flex;
        align-items: center;
    gap: 0.5ch;
    ">
          {!showFilter.value &&
            (
              <div onClick={triggerShowFilter} class="hoverable">
                Filter&nbsp;
                <span className={iconStyle}>
                  /
                </span>
              </div>
            )}

          {showFilter.value &&
            (
              <div onClick={clearShowFilter} class="hoverable">
                Clear Filter&nbsp;
                <span className={iconStyle}>
                  ESC
                </span>
              </div>
            )}

          <div
            className={borderRight}
            style={{
              width: "1px",
              height: "1.5em",
            }}
          />

          <div onClick={async (e) => await triggerCopy()} class="hoverable">
            Copy&nbsp;
            <span className={iconStyle}>
              <Icon name="IconReturnKey" />
            </span>
          </div>

          <div
            className={borderRight}
            style={{
              width: "1px",
              height: "1.5em",
            }}
          />

          <div
            onClick={() => {
              themeMode.value = themeMode.value === "light" ? "dark" : "light";
            }}
            class="hoverable"
          >
            <span style="
            display: inline-block;
            width: 1.5em;
            height: 1.5em;
            text-align: center;
            border-radius: 5px;
            ">
              {themeMode.value == "light"
                ? <Icon name="IconMoon" />
                : <Icon name="IconSun" />}
            </span>
          </div>

          <div
            className={borderRight}
            style={{
              width: "1px",
              height: "1.5em",
            }}
          />

          <div>
            <span style="
            display: inline-block;
            width: 8ch;
            height: 1.5em;
            text-align: center;
            border-radius: 5px;
            ">
            # {items.value.size}
            </span>
          </div>
        </div>
      </footer>
    </main>
  );
}

function App() {
  useEffect(() => {
    interface DataFromRustEvent {
      payload: {
        message: string;
      };
    }

    function handleDataFromRust(event: DataFromRustEvent) {
      console.log("Data pushed from Rust:", event);
      addItem(
        event.payload.message,
        items,
        availableItems,
        selected,
        focusSelected,
        updateSelected,
      );
    }

    async function fetchData() {
      try {
        const data: string[] = await invoke("init_process");
        data.forEach((raw) => {
          addItem(
            raw,
            items,
            availableItems,
            selected,
            focusSelected,
            updateSelected,
          );
        });
      } catch (error) {
        console.error("Error in init_process:", error);
      }
      listen("item", handleDataFromRust);
    }
    fetchData();

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
