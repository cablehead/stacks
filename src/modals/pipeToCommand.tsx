import { computed, Signal, signal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";

import { overlay, vars } from "../ui/app.css";
import { Icon } from "../ui/icons";
import { Modes } from "./types";
import { getContent, Item, Scru128Id, Stack } from "../types";

interface ExecStatus {
  exec_id: number;
  out?: Scru128Id;
  err?: Scru128Id;
  code?: number;
}

const state = (() => {
  const curr = signal("");

  const exec_id = signal(0);

  const status: Signal<ExecStatus | undefined> = signal(undefined);
  const out: Signal<Item | undefined> = signal(undefined);

  const code = computed((): number | undefined => {
    const res = status.value?.exec_id === exec_id.value
      ? status.value.code
      : undefined;
    console.log("COMPUTERED", res);
    return res;
  });

  (async () => {
    const d1 = await listen(
      "pipe-to-shell",
      (event: { payload: ExecStatus }) => {
        status.value = event.payload;
        console.log("pipe-to-shell", exec_id.value, status.value);
      },
    );
    if (import.meta.hot) {
      import.meta.hot.dispose(() => {
        if (d1) d1();
      });
    }
  })();

  return {
    curr,
    exec_id,
    out,
    code,
    accept_meta: async (stack: Stack, _: Modes) => {
      const selected = stack.selected_item();
      if (!selected) return;
      exec_id.value += 1;
      const args = {
        execId: exec_id.value,
        sourceId: selected.id,
        command: curr.value,
      };
      out.value = undefined;
      invoke("store_pipe_to_command", args);
    },
  };
})();

export default {
  name: () =>
    `Pipe clip to shell${
      state.code.value !== undefined ? ` :: exit code: ${state.code.value}` : ""
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

  activate: (stack: Stack) => {
    state.exec_id.value += 1;
    state.out.value = stack.selected();
  },

  Modal: ({ stack, modes }: { stack: Stack; modes: Modes }) => {
    const inputRef = useRef<HTMLTextAreaElement>(null);
    useEffect(() => {
      if (inputRef.current != null) {
        inputRef.current.focus();
        inputRef.current.value = "cat";
        inputRef.current.select();
        state.curr.value = "cat";
        // state.accept_meta(stack, modes);
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
                display: "flex",
                gap: "1ch",
              }}
            >
              <div>$</div>
              <textarea
                ref={inputRef}
                spellcheck={false}
                style={{
                  width: "100%",
                  outline: "none",
                  resize: "none",
                  boxSizing: "border-box",
                  padding: 0,
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
                padding: "1ch 2ch",
                boxShadow: `0 -2px 3px ${vars.shadowColor}`,
                backgroundColor: vars.backgroundColor,
                color: vars.textColor,
                borderColor: vars.borderColor,
              }}
              dangerouslySetInnerHTML={{
                __html: state.out.value !== undefined &&
                    getContent(state.out.value).value?.preview ||
                  (state.code.value !== undefined && "<i>no output</i>" ||
                    "<i>...</i>"),
              }}
            >
            </div>
            {
              /*state.res.value.err != "" &&
              (
                <div
                  style={{
                    whiteSpace: "pre",
                    width: "100%",
                    flex: "1",
                    overflow: "auto",
                    padding: "1ch 2ch",
                    boxShadow: `0 -2px 3px ${vars.shadowColor}`,
                    backgroundColor: vars.backgroundColor,
                    color: vars.textColor,
                    borderColor: vars.borderColor,
                  }}
                >
                  {state.res.value.err}
                </div>
              )*/
            }
          </div>
        </div>
      </div>
    );
  },
};
