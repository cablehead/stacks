import { render } from "preact";

function App() {
  return (
    <main>
      <div style={{paddingBottom: "0.5rem", borderBottom: "solid 1px #333"}}>
        <div>
          <input type="text" placeholder="Type a command..." />
        </div>
      </div>
      <div class="results">
        <div>Command 1</div>
        <div>Command 2</div>
        <div>Command 3</div>
        <div>Command 4</div>
      </div>
    </main>
  );
}

render(<App />, document.querySelector("body"));
