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

const options = signal([]);

function App() {
  useEffect(() => {
    function handleDataFromRust(event) {
      console.log("Data pushed from Rust:", event);
      options.value = [...options.value, event];
    }

    async function fetchData() {
      try {
        const initialData = await invoke("init_process");
        options.value = initialData;
      } catch (error) {
        console.error("Error in init_process:", error);
      }
    }

    listen("item", handleDataFromRust);

    fetchData();

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
        {options.value.map((option) => (
          <div
            style={{
              maxHeight: "3rem",
              overflow: "hidden",
              whiteSpace: "nowrap",
            }}
          >
            {option}
          </div>
        ))}
      </div>
    </main>
  );
}

render(<App />, document.querySelector("body"));
