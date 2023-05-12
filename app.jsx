import { render } from "preact";
import { signal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

const { listen } = require("@tauri-apps/api/event");
const { invoke } = require("@tauri-apps/api/tauri");
const { getCurrent } = require("@tauri-apps/api/window");

import { Scru128Id } from "scru128";

function scru128ToDate(id) {
  const scruId = Scru128Id.fromString(id);
  const timestampMillis = scruId.timestamp;
  const date = new Date(timestampMillis);
  return date;
}

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

        function handleKeys(event) {
          switch (true) {
            case event.key === "Escape":
              const currentWindow = getCurrent();
              currentWindow.hide();
              break;

            case event.key === "Enter":
              console.log(inputElement.value);
              inputElement.value = "";
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
          <div class="results"></div>
        </div>
        <div class="right-pane"></div>
      </div>
      <div style={{ paddingTop: "0.5rem", borderTop: "solid 1px #aaa" }}>
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
