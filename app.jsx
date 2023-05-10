import { render } from "preact";
import { signal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

const { listen } = require("@tauri-apps/api/event");
const { invoke } = require("@tauri-apps/api/tauri");
const { getCurrent } = require("@tauri-apps/api/window");

import { Scru128Id } from "scru128";

function cmp(a, b) {
  if (a < b) {
    return -1;
  } else if (a > b) {
    return 1;
  } else {
    return 0;
  }
}

function scru128ToDate(id) {
  const scruId = Scru128Id.fromString(id);
  const timestampMillis = scruId.timestamp;
  const date = new Date(timestampMillis);
  return date;
}

async function rustLog(message) {
  try {
    await invoke("js_log", { message });
  } catch (error) {
    console.error("Error logging message in Rust:", error);
  }
}

const items = signal([]);
const selected = signal(0);

function App() {
  const mainRef = useRef(null);

  useEffect(() => {
    function handleDataFromRust(event) {
      console.log("Data pushed from Rust:", event);
      items.value = [...items.value, JSON.parse(event.payload.message)];
    }

    async function fetchData() {
      try {
        const initialData = await invoke("init_process");
        items.value = initialData.map(JSON.parse);
      } catch (error) {
        console.error("Error in init_process:", error);
      }
    }

    listen("item", handleDataFromRust);

    fetchData();

    mainRef.current.focus();

    return () => {
      rustLog("Component is unmounted!");
    };
  }, []);

  function handleKeyDown(event) {
    if (event.ctrlKey && event.key === "n") {
      selected.value = (selected.value + 1) % items.value.length;
    } else if (event.ctrlKey && event.key === "p") {
      selected.value = selected.value === 0
        ? items.value.length - 1
        : selected.value - 1;
    } else if (event.key === "Escape") {
      const currentWindow = getCurrent();
      currentWindow.hide();
    }

    // Scroll the selected item into view
    const selectedItem = mainRef.current.querySelector(
      `.results > div:nth-child(${selected.value + 1})`,
    );
    selectedItem.scrollIntoView({ behavior: "smooth", block: "nearest" });
  }

  function handleItemClick(index) {
    selected.value = index;
  }

  return (
    <main ref={mainRef} onKeyDown={handleKeyDown} tabIndex="0">
      <div style={{ paddingBottom: "0.5rem", borderBottom: "solid 1px #333" }}>
        <div>
          <input type="text" placeholder="Type a command..." />
        </div>
      </div>
      <div class="container">
        <div class="left-pane">
          <div class="results">
            {items.value
              .sort((a, b) => cmp(b.id, a.id))
              .map((item, index) => {
                let date = scru128ToDate(item.id);
                let itemData = JSON.parse(item.data);

                let displayText = "public.utf8-plain-text" in itemData.types
                  ? atob(itemData.types["public.utf8-plain-text"])
                  : itemData.source;

                return (
                  <div
                    className={index === selected.value ? "selected" : ""}
                    style={{
                      maxHeight: "3rem",
                      overflow: "hidden",
                      whiteSpace: "nowrap",
                    }}
                    onClick={() => handleItemClick(index)}
                  >
                    {displayText}
                  </div>
                );
              })}
          </div>
        </div>
        <div class="right-pane">
          <pre>{selected.value >= 0 && items.value.length > 0 &&
      JSON.stringify(JSON.parse(items.value[selected.value].data), null, 2)}</pre>
        </div>
      </div>
    </main>
  );
}

render(<App />, document.querySelector("body"));
