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
const mode = signal("list");

function NewItemUI({ onSubmit }) {
  const inputRef = useRef(null);

  useEffect(() => {
    // Set focus on the input field when the component is mounted
    inputRef.current.focus();
  }, []);

  return (
    <form onSubmit={onSubmit}>
      <textarea
        ref={inputRef}
        type="text"
        name="item"
        value=""
        placeholder="Type a new item..."
      />
    </form>
  );
}

function App() {
  const mainRef = useRef(null);

  useEffect(() => {
    function handleDataFromRust(event) {
      console.log("Data pushed from Rust:", event);
      items.value = [...items.value, JSON.parse(event.payload.message)];
      if (selected.value > 0) selected.value += 1;
    }

    async function fetchData() {
      try {
        const initialData = await invoke("init_process");
        items.value = initialData.map(JSON.parse);
      } catch (error) {
        console.error("Error in init_process:", error);
      }

      listen("item", handleDataFromRust);
    }

    fetchData();

    mainRef.current.focus();

    return () => {
      rustLog("Component is unmounted!");
    };
  }, []);

  function updateSelected(n) {
    selected.value = (selected.value + n) % items.value.length;
    if (selected.value < 0) {
      selected.value = items.value.length + selected.value;
    }

    // Scroll the selected item into view
    const selectedItem = mainRef.current.querySelector(
      `.results > div:nth-child(${selected.value + 1})`,
    );
    selectedItem.scrollIntoView({ behavior: "smooth", block: "nearest" });
  }

  function handleKeyDown(event) {
    switch (true) {
      case event.ctrlKey && event.key === "n":
        updateSelected(1);
        break;

      case event.ctrlKey && event.key === "p":
        updateSelected(-1);
        break;

      case event.metaKey && event.key === "n":
        mode.value = "new-item";
        break;

      case event.key === "Escape":
        const currentWindow = getCurrent();
        currentWindow.hide();
        break;
    }
  }

  function handleItemClick(index) {
    selected.value = index;
  }

  async function handleFormSubmit(event) {
    event.preventDefault();
    console.log(event.target);
    const formData = new FormData(event.target);
    const inputValue = formData.get("item");
    console.log(inputValue);
    if (inputValue.trim() !== "") {
      // Send the new item to the Rust backend
      try {
        await invoke("add_item", { item: inputValue });
        mainRef.current.focus();
      } catch (error) {
        console.error("Error adding item:", error);
      }
    }
    mode.value = "list";
  }

  return (
    <main ref={mainRef} onKeyDown={handleKeyDown} tabIndex="0">
      {mode.value == "list" &&
        (
          <div class="container">
            <div class="left-pane">
              <div class="results">
                {items.value
                  .sort((a, b) => cmp(b.id, a.id))
                  .map((item, index) => {
                    let displayText = item.data;

                    return (
                      <div
                        className={index === selected.value ? "selected" : ""}
                        style={{
                          maxHeight: "3rem",
                          overflow: "hidden",
                          whiteSpace: "nowrap",
                          textOverflow: "ellipsis",
                          width: "100%",
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
                      {scru128ToDate(items.value[selected.value].id)
                        .toLocaleString(
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
                    <div>
                      Topic
                    </div>
                    <div>
                      {items.value[selected.value].topic}
                    </div>
                  </div>
                </>
              )}
            </div>
          </div>
        )}

      {mode.value === "new-item" && (
        <NewItemUI
          onSubmit={handleFormSubmit}
        />
      )}
    </main>
  );
}

render(<App />, document.querySelector("body"));
