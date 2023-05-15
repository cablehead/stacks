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

function ListView() {
  const mainRef = useRef(null);

  useEffect(() => {
    if (mainRef.current) {
      const inputElement = mainRef.current.querySelector("input");
      if (inputElement) {
        inputElement.focus();

        const handleBlur = () => {
          setTimeout(() => {
            inputElement.focus();
          }, 0);
        };
        inputElement.addEventListener("blur", handleBlur);

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

            case event.key === "Enter":
              if (inputElement.value.trim() !== "") {
                const item = {
                  id: scru128String(),
                  command: inputElement.value,
                };
                item.output = await invoke("run_command", {
                  command: item.command,
                });
                items.value = [...items.value, item];
                inputElement.value = "";
                selected.value = items.value.length - 1;
                updateSelected(0);
              }
              break;

            case (event.ctrlKey && event.key === "n") ||
              event.key === "ArrowDown":
              updateSelected(1);
              break;

            case event.ctrlKey && event.key === "p" || event.key === "ArrowUp":
              updateSelected(-1);
              break;
          }
        }
        inputElement.addEventListener("keydown", handleKeys);

        return () => {
          inputElement.removeEventListener("blur", handleBlur);
          inputElement.removeEventListener("keydown", handleKeys);
        };
      }
    }
  }, []);

  return (
    <main ref={mainRef}>
      <div class="container">
        <div class="left-pane">
          <div class="results">
            {items.value
              .map((item, index) => {
                let displayText = item.command;
                return (
                  <div
                    className={index === selected.value ? "selected" : ""}
                    onClick={() => selected.value = index}
                    style={{
                      maxHeight: "3rem",
                      overflow: "hidden",
                      whiteSpace: "nowrap",
                      textOverflow: "ellipsis",
                      width: "100%",
                    }}
                  >
                    {displayText}
                  </div>
                );
              })}
          </div>
        </div>
        <div class="right-pane">
          {selected.value >= 0 && items.value.length > 0 && (
            <div style="flex: 1; padding-bottom: 1rem; border-bottom: 1px solid #aaa; flex:2; overflow-y: auto; ">
              <pre>
              {items.value[selected.value].output.stdout}
              </pre>
            </div>
          )}
        </div>
      </div>
      <div style={{ paddingTop: "0.5ch", borderTop: "solid 1px #aaa" }}>
        <div>
          <input
            type="text"
            placeholder="Type a command..."
          />
        </div>
      </div>
    </main>
  );
}

function App() {
  return <ListView />;
}

render(<App />, document.querySelector("body"));
