// import { invoke } from "@tauri-apps/api/tauri";

import { overlay, vars } from "../ui/app.css";
import { Modes } from "./types";
import { Stack } from "../types";

export default {
  name: (_: Stack) => "Settings",
  hotKeys: (_: Stack, modes: Modes) => [
    {
      name: "Discard",
      keys: ["ESC"],
      onMouseDown: () => modes.deactivate(),
    },
  ],
  Modal: ({ stack, modes }: { stack: Stack; modes: Modes }) => {
    console.log(stack, modes);

    return (
      <div
        className={overlay}
        style={{
          position: "absolute",
          overflow: "auto",
          fontSize: "0.9rem",
          bottom: "2ch",
          right: "2ch",
          left: "2ch",
          top: "2ch",
          padding: "1ch 2ch 1ch 2ch",
          borderRadius: "0.5rem",
          zIndex: 1000,
        }}
      >
        <p>OpenAI API Access</p>
        <form onSubmit={() => {}}>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              textAlign: "right",
            }}
          >
            <label style={{ width: "40ch" }}>Access Token</label>
            <input
              type="text"
              style={{
                outline: "none",
                borderColor: vars.borderColor,
                borderWidth: "1px",
                borderStyle: "solid",
                borderRadius: "0.25rem",
              }}
              name="accessToken"
              value={""}
              onChange={() => {}}
            />
          </div>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              textAlign: "right",
            }}
          >
            <label style={{ width: "40ch" }}>Preferred Model</label>
            <select
              name="selectedModel"
              value={"davinci"}
              onChange={() => {}}
              style={{
                outline: "none",
                borderColor: vars.borderColor,
                borderWidth: "1px",
                borderStyle: "solid",
                borderRadius: "0.25rem",
                appearance: "none",
              }}
            >
              <option value="davinci">DaVinci</option>
              <option value="curie">Curie</option>
              <option value="babbage">Babbage</option>
              <option value="ada">Ada</option>
            </select>
          </div>
        </form>
      </div>
    );
  },
};
