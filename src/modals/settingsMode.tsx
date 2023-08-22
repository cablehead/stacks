// import { invoke } from "@tauri-apps/api/tauri";

import { overlay } from "../ui/app.css";
import { Modes } from "./types";
import { Stack } from "../types";

export default {
  name: (_: Stack) => "Settings",
  hotKeys: (_: Stack, modes: Modes) => [
    {
      name: "Done",
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
          // fontSize: "0.9rem",
          bottom: "2ch",
          right: "2ch",
          left: "2ch",
          top: "2ch",
          padding: "1ch 2ch 1ch 2ch",
          borderRadius: "0.5rem",
          zIndex: 1000,
        }}
      >
        <p>Why can't I see this?</p>
      </div>
    );
  },
};
