import { Signal, signal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

import { invoke } from "@tauri-apps/api/tauri";

import { overlay, vars } from "../ui/app.css";
import { b64ToUtf8 } from "../utils";
import { Icon } from "../ui/icons";
import { Modes } from "./types";
import { Stack } from "../types";

interface CommandOutput {
  out: string;
  err: string;
  code: number;
  mime_type?: string;
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
      const selected = stack.selected_item();
      if (!selected) return;
      const args = {
        sourceId: selected.id,
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
    `Pipe clip to shell${
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
                height: "6lh",
                minHeight: "6lh",
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
                placeholder="Shell command ..."
                onInput={(event) => {
                  state.curr.value =
                    (event.target as HTMLTextAreaElement).value;
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

            <div
              style={{
                whiteSpace: "pre",
                overflow: "auto",
                width: "100%",
                flex: "1",
                padding: "1ch 3ch",
                boxShadow: `0 -2px 3px ${vars.shadowColor}`,
                backgroundColor: vars.backgroundColor,
                color: vars.textColor,
                borderColor: vars.borderColor,
              }}
            >
              {state.res.value.mime_type?.startsWith("image/")
                ? (
                  <img
                    src={`data:${state.res.value.mime_type};base64,${state.res.value.out}`}
                  />
                )
                : b64ToUtf8(state.res.value.out)}
            </div>
            {state.res.value.err != "" &&
              (
                <div
                  style={{
                    whiteSpace: "pre",
                    width: "100%",
                    flex: "1",
                    overflow: "auto",
                    padding: "1ch 3ch",
                    boxShadow: `0 -2px 3px ${vars.shadowColor}`,
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
      </div>
    );
  },
};
