import { render } from "preact";
import { signal } from "@preact/signals";

const options = signal([
  "Option 1",
  "Option 2",
  "Option 3",
  "Option 4",
]);

function App() {
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
