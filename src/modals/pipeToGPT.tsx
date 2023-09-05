import { Signal, signal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

// import { invoke } from "@tauri-apps/api/tauri";

import { b64ToUtf8 } from "../utils";
import { overlay, vars } from "../ui/app.css";
import { Icon } from "../ui/icons";
import { Modes } from "./types";
import { Stack } from "../types";

interface CommandOutput {
  out: string;
  err: string;
  code: number;
}

const state = (() => {
  const curr = signal("");

  const res: Signal<CommandOutput> = signal(
    {
      out: "",
      err: "",
      code: 0,
    },
  );

  return {
    curr,
    res,
    accept_meta: async (stack: Stack, _: Modes) => {
      const selected = stack.selected();
      if (!selected) return;
      /*
      console.log("FOO", curr.value);
      const args = {
        sourceId: selected.id,
        command: curr.value,
      };
      const res: CommandOutput = await invoke("store_pipe_to_command", args);
      state.res.value = res;
      console.log("RES", res);
      */
    },
  };
})();

export default {
  name: () => `Pipe to GPT`,
  hotKeys: (stack: Stack, modes: Modes) => [
    {
      name: "Submit",
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
        inputRef.current.value = "";
        inputRef.current.select();
        state.curr.value = "";
      }
    }, []);

    const item = stack.selected();
    const content = (item && stack.getContent(item.hash).value) || "";

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
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            height: "100%",
            justifyContent: "space-between",
          }}
        >
          <div
            style={{
              whiteSpace: "pre",
              overflow: "auto",
              width: "100%",
              padding: "1ch 3ch",
              backgroundColor: vars.backgroundColor,
              color: vars.textColor,
            }}
            >
          Context
          </div>

          <div
            style={{
              whiteSpace: "pre",
              overflow: "auto",
              width: "100%",
              flex: "1",
              padding: "1ch 3ch",
              boxShadow: `0 2px 5px ${vars.shadowColor}`,
              backgroundColor: vars.backgroundColor,
              color: vars.textColor,
              borderColor: vars.borderColor,
            }}
          >
            {b64ToUtf8(content)}
          </div>

          <div
            style={{
              height: "4lh",
              minHeight: "4lh",
              overflow: "auto",
            }}
          >
            <textarea
              ref={inputRef}
              spellcheck={false}
              style={{
                width: "100%",
                margin: "2ch",
                outline: "none",
                border: "none",
              }}
              placeholder="Additional prompt ..."
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

          {false &&
            (
              <div
                style={{
                  whiteSpace: "pre",
                  overflow: "auto",
                  width: "100%",
                  flex: "1",
                  padding: "1ch 3ch",
                  boxShadow: `0 -2px 5px ${vars.shadowColor}`,
                  backgroundColor: vars.backgroundColor,
                  color: vars.textColor,
                  borderColor: vars.borderColor,
                }}
              >
                {state.res.value.out}
              </div>
            )}
          {state.res.value.err != "" &&
            (
              <div
                style={{
                  whiteSpace: "pre",
                  width: "100%",
                  flex: "1",
                  overflow: "auto",
                  padding: "1ch 3ch",
                  boxShadow: `0 -2px 5px ${vars.shadowColor}`,
                  backgroundColor: vars.backgroundColor,
                  color: vars.textColor,
                  borderColor: vars.borderColor,
                }}
              >
                {state.res.value.err}
              </div>
            )}
        </div>
      </div>
    );
  },
};
