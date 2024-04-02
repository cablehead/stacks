import { Signal, signal, useSignal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

import { invoke } from "@tauri-apps/api/tauri";

import { overlay } from "../ui/app.css";
import { Icon } from "../ui/icons";
import { Modes } from "./types";
import { Item, Stack } from "../types";
import { b64ToUtf8 } from "../utils";

const state = (() => {
  const curr = signal("");

  const focused_clip: Signal<Item | null> = signal(null);

  return {
    curr,
    focused_clip,
    accept_meta: async (_stack: Stack, modes: Modes) => {
      if (!focused_clip.value) return;

      if (!curr.value) {
        modes.deactivate();
        return;
      }

      const args = {
        sourceId: focused_clip.value.id,
        content: curr.value,
      };

      await invoke("store_edit_note", args);
      modes.deactivate();
    },
  };
})();

export default {
  name: (_: Stack) => "Rename stack",
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

  activate: (stack: Stack) => {
    const selected = stack.selected_stack();
    if (!selected) {
      return;
    }
    state.focused_clip.value = selected;
  },

  Modal: ({ stack, modes }: { stack: Stack; modes: Modes }) => {
    const inputRef = useRef<HTMLTextAreaElement>(null);
    useEffect(() => {
      if (inputRef.current != null) {
        inputRef.current.focus();
      }
    }, []);

    const content: Signal<string> = useSignal("");

    if (state.focused_clip.value) {
      const { hash } = state.focused_clip.value;
      (async () => {
        content.value = await invoke("store_get_raw_content", { hash });
        if (inputRef.current != null) {
          inputRef.current.select();
        }
      })();
    }

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
            placeholder="..."
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
            {b64ToUtf8(content.value)}
          </textarea>
        </div>
      </div>
    );
  },
};
