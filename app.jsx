import { render } from "preact";
import { signal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

const { listen } = require("@tauri-apps/api/event");
const { invoke } = require("@tauri-apps/api/tauri");
const { getCurrent } = require("@tauri-apps/api/window");

import { Scru128Id, scru128String } from "scru128";

function scru128ToDate(id) {
  const scruId = Scru128Id.fromString(id);
  const timestampMillis = scruId.timestamp;
  const date = new Date(timestampMillis);
  return date;
}

function cmp(a, b) {
  if (a < b) {
    return -1;
  } else if (a > b) {
    return 1;
  } else {
    return 0;
  }
}

const selected = signal(0);
const items = signal([]);

function parseItem(raw) {
  let item = JSON.parse(raw);
  item.created_at = scru128ToDate(item.id)
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
    );
  switch (item.topic) {
    case "command":
      item.o = JSON.parse(item.data);
      item.terse = item.o.command;
      item.preview = item.o.output.stdout;
      break;

    case "clipboard":
      let data = JSON.parse(item.data);
      if ("public.utf8-plain-text" in data.types) {
        item.terse = atob(data.types["public.utf8-plain-text"]);
        item.preview = item.terse;
        break;
      }
      item.terse = data.source;
      item.preview = item.data;
      break;

    default:
      item.terse = item.data;
      item.preview = item.data;
  }
  return item;
}

function RightPane({ item }) {
  if (!item) {
    return <div />;
  }

  return (
    <div class="right-pane">
      <div style="flex: 1; padding-bottom: 1rem; border-bottom: 1px solid #aaa; flex:2; overflow-y: auto;">
        <pre style="margin: 0;">
        {item.preview}
        </pre>
      </div>
      <div style="max-height: 5lh; font-size: 0.8rem; font-weight: 500; display: grid; grid-template-columns: min-content 1fr; overflow-y: auto; padding:1ch; align-content: start;">
        <div>
          ID
        </div>
        <div>
          {item.id}
        </div>
        <div>
          Created
        </div>
        <div>
          {item.created_at}
        </div>
        <div>
          Topic
        </div>
        <div>
          {item.topic}
        </div>
      </div>
    </div>
  );
}

function ListView() {
  const mainRef = useRef(null);

  useEffect(() => {
    if (mainRef.current) {
      function updateSelected(n) {
        selected.value = (selected.value + n) % items.value.length;
        if (selected.value < 0) {
          selected.value = items.value.length + selected.value;
        }

        setTimeout(() => {
          // Scroll the selected item into view
          const selectedItem = mainRef.current.querySelector(
            `.results > div:nth-child(${selected.value + 1})`,
          );
          selectedItem.scrollIntoView({
            behavior: "smooth",
            block: "nearest",
          });
        }, 0);
      }

      async function handleKeys(event) {
        switch (true) {
          case event.key === "Escape":
            const currentWindow = getCurrent();
            currentWindow.hide();
            break;

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

          case (event.ctrlKey && event.key === "n") ||
            event.key === "ArrowDown":
            updateSelected(1);
            break;

          case event.ctrlKey && event.key === "p" || event.key === "ArrowUp":
            updateSelected(-1);
            break;
        }
      }
      window.addEventListener("keydown", handleKeys);

      return () => {
        window.removeEventListener("keydown", handleKeys);
      };
    }
  }, []);

  return (
    <main ref={mainRef}>
      <div class="container">
        <div class="left-pane">
          <div class="results">
            {items.value
              .map((item, index) => {
                return (
                  <div
                    className={index === selected.value ? "selected" : ""}
                    onClick={() => selected.value = index}
                    style={{
                      display: "flex",
                      width: "100%",
                      maxHeight: "3rem",
                      gap: "0.5ch",
                      overflow: "hidden",
                    }}
                  >
                    <div
                      style={{
                        flexShrink: 0,
                        width: "4ch",
                        whiteSpace: "nowrap",
                        overflow: "hidden",
                      }}
                    >
                      {item.topic}
                    </div>

                    <div
                      style={{
                        flexShrink: 0,
                        width: "0.5ch",
                        overflow: "hidden",
                        borderLeft: "solid #aaa 1px",
                      }}
                    >
                      &nbsp;
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
              })}
          </div>
        </div>
        <RightPane item={items.value[selected.value]} />
      </div>
    </main>
  );
}

function App() {
  useEffect(() => {
    function handleDataFromRust(event) {
      console.log("Data pushed from Rust:", event);
      items.value = [
        parseItem(event.payload.message),
        ...items.value,
      ];
      if (selected.value > 0) selected.value += 1;
    }

    async function fetchData() {
      try {
        let data = await invoke("init_process");
        data = data.map(parseItem).reverse();
        items.value = data;
      } catch (error) {
        console.error("Error in init_process:", error);
      }
      listen("item", handleDataFromRust);
    }

    fetchData();
  }, []);

  return <ListView />;
}

render(<App />, document.querySelector("body"));
