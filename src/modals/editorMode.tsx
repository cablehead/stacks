import { signal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

import { writeText } from "@tauri-apps/api/clipboard";
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
      const source_ids = stack.item.value?.ids;
      const args = {
        item: curr.value,
        sourceId: (source_ids && source_ids.length > 0)
          ? source_ids[source_ids.length - 1]
          : undefined,
        stackName: stack.parent?.item.value?.terse,
      };

      console.log("CAPTURE", args);
      invoke("store_capture", args);

      writeText(curr.value);
      modes.deactivate();
    },
  };
})();

export default {
  name: "Editor",
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

    const content = stack.content?.value || "";

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
          placeholder="..."
          onChange={(event) => {
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
          {content}
        </textarea>
      </div>
    );
  },
};
