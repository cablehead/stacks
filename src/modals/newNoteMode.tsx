import { signal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

import { invoke } from "@tauri-apps/api/tauri";

import { overlay } from "../ui/app.css";
import { Icon } from "../ui/icons";
import { Modes } from "./types";
import { Stack, Focus} from "../types";

const state = (() => {
  const curr = signal("");
  return {
    curr,
    accept_meta: (stack: Stack, modes: Modes) => {
      const args = {
        content: curr.value,
      };

      invoke("store_new_note", args);
      stack.selected.value = Focus.first();
      modes.deactivate();
    },
  };
})();

export default {
  name: () => "New note",
  hotKeys: (stack: Stack, modes: Modes) => [
    {
      name: "Capture",
      keys: [
        <Icon name="IconCommandKey" />,
        <Icon name="IconReturnKey" />,
      ],
      onMouseDown: () => state.accept_meta(stack, modes),
    },
    {
      name: "Discard",
      keys: ["ESC"],
      onMouseDown: () => modes.deactivate(),
    },
  ],
  Modal: ({ stack, modes }: { stack: Stack; modes: Modes }) => {
    const inputRef = useRef<HTMLTextAreaElement>(null);
    useEffect(() => {
      if (inputRef.current != null) {
        inputRef.current.focus();
      }
    }, []);

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
          borderRadius: "0.5rem",
          zIndex: 1000,
        }}
      >
        <textarea
          ref={inputRef}
          spellcheck={false}
          style={{
            width: "100%",
            height: "100%",
            margin: "2ch",
            outline: "none",
            border: "none",
          }}
          onBlur={() => {
            modes.deactivate();
          }}
          placeholder="Enter note..."
          onInput={(event) => {
            state.curr.value = (event.target as HTMLTextAreaElement).value;
          }}
          onKeyDown={(event) => {
            event.stopPropagation();
            switch (true) {
              case event.key === "Escape":
                event.preventDefault();
                modes.deactivate();
                break;

              case event.metaKey && event.key === "e":
                event.preventDefault();
                modes.deactivate();
                break;

              case event.metaKey && event.key === "Enter":
                state.accept_meta(stack, modes);
                break;
            }
          }}
        >
        </textarea>
      </div>
    );
  },
};
