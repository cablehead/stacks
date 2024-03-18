import { signal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

import { invoke } from "@tauri-apps/api/tauri";

import { overlay } from "../ui/app.css";
import { Icon } from "../ui/icons";
import { Modes } from "./types";
import { Stack } from "../types";

const state = (() => {
  const curr = signal("");
  return {
    curr,
    accept_meta: (stack: Stack, modes: Modes) => {
      const selected = stack.selected();
      if (!selected) return;

      if (!curr.value) {
        modes.deactivate();
        return;
      }

      const args = {
        stackId: selected.stack_id || selected.id,
        content: curr.value,
        shouldFocus: true,
      };

      invoke("store_new_note", args);
      modes.deactivate();
    },
  };
})();

export default {
  name: () => "New clip",
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
        style={{
          width: "100%",
          height: "100%",
          backgroundColor: "transparent",
          zIndex: 1000,
          position: "absolute",
          padding: "1ch",
        }}
      >
        <div
          className={overlay}
          style={{
            padding: "2ch",
            borderRadius: "0.5rem",
            width: "calc(100% - 2ch)",
            height: "calc(100% - 2ch)",
          }}
        >
          <textarea
            ref={inputRef}
            spellcheck={false}
            style={{
              width: "100%",
              height: "100%",
              resize: "none",
              outline: "none",
              border: "none",
            }}
            placeholder="Enter clip..."
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

                case event.metaKey && event.key === "Enter":
                  state.accept_meta(stack, modes);
                  break;
              }
            }}
          >
          </textarea>
        </div>
      </div>
    );
  },
};
