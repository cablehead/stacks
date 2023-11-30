import { Signal, signal, useSignal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

import { invoke } from "@tauri-apps/api/tauri";

import { overlay } from "../ui/app.css";
import { Icon } from "../ui/icons";
import { Modes } from "./types";
import { getContent, Stack } from "../types";
import { b64ToUtf8 } from "../utils";

const state = (() => {
  const curr = signal("");
  return {
    curr,
    accept_meta: async (stack: Stack, modes: Modes) => {
      const item = stack.selected();
      if (!item) return;

      if (!curr.value) {
        modes.deactivate();
        return;
      }

      const args = {
        sourceId: item.id,
        content: curr.value,
      };

      await invoke("store_edit_note", args);
      modes.deactivate();
    },
  };
})();

export default {
  name: (_: Stack) => "Editor",
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
        inputRef.current.select();
      }
    }, []);

    const item = stack.selected();

    const content: Signal<string> = useSignal("");

    const meta = item && getContent(item).value;

    if (item) {
      (async () => {
        content.value = await invoke("store_get_raw_content", {
          hash: item.hash,
        });
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
            fontSize: "0.9rem",
            padding: "2ch",
            borderRadius: "0.5rem",
            width: "calc(100% - 2ch)",
            height: "calc(100% - 2ch)",
          }}
        >
          {meta &&
            (
              <div>
                {meta.content_type}
              </div>
            )}

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
