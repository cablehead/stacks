import { Signal, signal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";

import { overlay, vars } from "../ui/app.css";
import { Icon } from "../ui/icons";
import { Modes } from "./types";
import { Cacheable, getContent, Scru128Id, Stack } from "../types";

interface ExecStatus {
  exec_id: number;
  out?: Cacheable;
  err?: Cacheable;
  code?: number;
}

const state = (() => {
  const curr = signal("");

  let exec_id = 0;
  const clip_id: Signal<Scru128Id> = signal("0" as Scru128Id);
  const status: Signal<ExecStatus | undefined> = signal(undefined);

  (async () => {
    const d1 = await listen(
      "pipe-to-shell",
      (event: { payload: ExecStatus }) => {
        if (event.payload.exec_id === exec_id) {
          console.log("pipe-to-shell", exec_id, status.value, event.payload);
          status.value = { ...status.value, ...event.payload };
        }
      },
    );
    if (import.meta.hot) {
      import.meta.hot.dispose(() => {
        if (d1) d1();
      });
    }
  })();

  return {
    status,
    curr,
    clip_id,
    accept_meta: async (_: Stack, __: Modes) => {
      exec_id += 1;
      status.value = undefined;
      const args = {
        execId: exec_id,
        sourceId: clip_id.value,
        command: curr.value,
      };
      status.value = undefined;
      invoke("store_pipe_to_command", args);
    },
  };
})();

export default {
  name: () =>
    `Pipe clip to shell${
      state.status.value?.code !== undefined
        ? ` :: exit code: ${state.status.value.code}`
        : ""
    }`,
  hotKeys: (stack: Stack, modes: Modes) => [
    {
      name: "Execute",
      keys: [
        <Icon name="IconCommandKey" />,
        <Icon name="IconReturnKey" />,
      ],
      onMouseDown: () => state.accept_meta(stack, modes),
      matchKeyEvent: (event: KeyboardEvent) =>
        event.metaKey && event.key === "Enter",
    },
    {
      name: "Back",
      keys: ["ESC"],
      onMouseDown: () => modes.deactivate(),
      matchKeyEvent: (event: KeyboardEvent) => event.key === "Escape",
    },
  ],

  activate: (stack: Stack) => {
    const selected = stack.selected_item();
    if (!selected) {
      return;
    }
    state.clip_id.value = selected.id;
    state.status.value = {
      exec_id: 0,
      out: selected,
      code: 0,
    };
  },

  Modal: (_props: { stack: Stack; modes: Modes }) => {
    const inputRef = useRef<HTMLTextAreaElement>(null);
    useEffect(() => {
      if (inputRef.current != null) {
        inputRef.current.focus();
        inputRef.current.value = "cat";
        inputRef.current.select();
        state.curr.value = "cat";
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
              >
              </textarea>
            </div>

            <div
              style={{
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
                __html: state.status.value?.out &&
                    getContent(state.status.value.out).value?.preview ||
                  (state.status.value?.code !== undefined &&
                      "<i>no output</i>" ||
                    "<i>...</i>"),
              }}
            >
            </div>

            {(() => {
              const preview = state.status.value?.err &&
                getContent(state.status.value.err).value?.preview;

              return preview && (
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
                  dangerouslySetInnerHTML={{ __html: preview }}
                >
                </div>
              );
            })()}
          </div>
        </div>
      </div>
    );
  },
};
