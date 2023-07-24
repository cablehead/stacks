import { Signal, signal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

import { invoke } from "@tauri-apps/api/tauri";

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
      console.log(curr.value, stack?.item.value?.hash);
      const args = {
        hash: stack.item.value?.hash,
        command: curr.value,
      };
      const res: CommandOutput = await invoke("store_pipe_to_command", args);
      state.res.value = res;
      console.log("RES", res);
    },
  };
})();

export default {
  name: () =>
    `Pipe to command${
      state.res.value.code != 0 ? ` :: exit code: ${state.res.value.code}` : ""
    }`,
  hotKeys: (stack: Stack, modes: Modes) => [
    {
      name: "Execute",
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
        inputRef.current.value = "cat";
        inputRef.current.select();
        state.curr.value = "cat";
        state.accept_meta(stack, modes);
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
          display: "flex",
          flexDirection: "column",
        }}
      >
        <textarea
          ref={inputRef}
          spellcheck={false}
          style={{
            width: "100%",
            height: "2lh",
            margin: "2ch",
            outline: "none",
            border: "none",
          }}
          placeholder="Enter command..."
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

              case event.metaKey && event.key === "Enter":
                state.accept_meta(stack, modes);
                break;
            }
          }}
        >
        </textarea>

        <div
          style={{
            flexGrow: 1,
            display: "flex",
            flexDirection: "column",
          }}
        >
          <div
            style={{
              whiteSpace: "pre",
              height: "100%",
              width: "100%",
              overflow: "auto",
              padding: "1ch 3ch",
              boxShadow: `0 -2px 5px ${vars.shadowColor}`,
              backgroundColor: vars.backgroundColor,
              color: vars.textColor,
              borderColor: vars.borderColor,
            }}
          >
            {state.res.value.out}
          </div>
          {state.res.value.err != "" &&
            (
              <div
                style={{
                  whiteSpace: "pre",
                  height: "100%",
                  width: "100%",
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
