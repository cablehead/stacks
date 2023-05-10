import { render } from "preact";
import { signal } from "@preact/signals";
import { useEffect } from "preact/hooks";

const { listen } = require("@tauri-apps/api/event");
const { invoke } = require("@tauri-apps/api/tauri");

async function rustLog(message) {
  try {
    await invoke("js_log", { message });
  } catch (error) {
    console.error("Error logging message in Rust:", error);
  }
}

const options = signal([
  "Option 1",
  "Option 2",
  "Option 3",
  "Option 4",
  "Option Duck Goose 2",
]);

function App() {
  useEffect(() => {
    function handleDataFromRust(event) {
      console.log("Data pushed from Rust:", event);
    }

    listen("item", handleDataFromRust);

    invoke("init_process");

    return () => {
      rustLog("Component is unmounted!");
    };
  }, []);

  return (
    <main>
      <div style={{ paddingBottom: "0.5rem", borderBottom: "solid 1px #333" }}>
        <div>
          <input type="text" placeholder="Type a command..." />
        </div>
      </div>
      <div class="results">
        {options.value.map((option) => <div>{option}</div>)}
      </div>
    </main>
  );
}

render(<App />, document.querySelector("body"));
