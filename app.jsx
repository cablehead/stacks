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
      <div class="container">
        <div class="left-pane">
          <table class="results">
            <tbody>
              {items.value
                .sort((a, b) => cmp(b.id, a.id))
                .map((item, index) => {
                  return (
                    <tr
                      className={index === selected.value ? "selected" : ""}
                      onClick={() => handleItemClick(index)}
                    >
                      <td>{item.topic}</td>
                      <td
                        style={{
                          whiteSpace: "nowrap",
                          textOverflow: "ellipsis",
                          width: "100%",
                        }}
                      >
                        {item.data}
                      </td>
                    </tr>
                  );
                })}
            </tbody>
          </table>
        </div>
        <div class="right-pane">
          {selected.value >= 0 && items.value.length > 0 && (
            <>
              <div style="flex: 1; padding-bottom: 1rem; border-bottom: 1px solid #aaa; flex:2; overflow-y: auto; ">
                <pre>
              {items.value[selected.value].data}
                </pre>
              </div>
              <div style="max-height: 5lh; font-size: 0.8rem; font-weight: 500; display: grid; grid-template-columns: min-content 1fr; overflow-y: auto; padding:1ch; align-content: start;">
                <div>
                  ID
                </div>
                <div>
                  {items.value[selected.value].id}
                </div>
                <div>
                  Created
                </div>
                <div>
                  {scru128ToDate(items.value[selected.value].id).toLocaleString(
                    "en-US",
                    {
                      weekday: "short",
                      year: "numeric",
                      month: "short",
                      day: "numeric",
                      hour: "numeric",
                      minute: "numeric",
                      hour12: true,
                    },
                  )}
                </div>
              </div>
            </>
          )}
        </div>
      </div>
    </main>
  );
}

render(<App />, document.querySelector("body"));
