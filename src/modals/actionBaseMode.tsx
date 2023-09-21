import { JSXInternal } from "preact/src/jsx";
import { computed, effect, Signal, signal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

import { borderBottom, overlay } from "../ui/app.css";
import { Icon } from "../ui/icons";

import { Modes } from "./types";
import { Stack } from "../types";

export default function createActionMode(
  name: (stack: Stack) => string | JSXInternal.Element,
  availOptionsFn: (stack: Stack, availOptions: Signal<string[]>) => void,
  acceptFn: (stack: Stack, modes: Modes, selected: string) => void,
) {
  const state = (() => {
    const selected = signal("");
    const currFilter = signal("");

    const availOptions: Signal<string[]> = signal([]);

    const options = computed(() =>
      availOptions.value
        .filter((item) =>
          currFilter.value == "" ||
          item.toLowerCase().includes(currFilter.value.toLowerCase())
        )
    );

    effect(() => {
      if (options.value.length <= 0) return;
      const chosen = options.value.find((item) => item === selected.peek());
      if (!chosen) selected.value = options.value[0];
    });

    return {
      selected,
      currFilter,
      availOptions,
      options,

      accept: (stack: Stack, modes: Modes) => {
        acceptFn(stack, modes, state.selected.value);
      },

      selectDown: () => {
        const idx = options.value.findIndex((item) => item == selected.peek());
        if (idx < options.value.length - 1) {
          state.selected.value = options.value[idx + 1];
        }
      },

      selectUp: () => {
        const idx = options.value.findIndex((item) => item == selected.peek());
        if (idx > 0) {
          selected.value = options.value[idx - 1];
        }
      },
    };
  })();

  return {
    name: (stack: Stack) => name(stack),

    hotKeys: (stack: Stack, modes: Modes) => {
      const ret = [];

      if (state.options.value.length > 0) {
        ret.push({
          name: "Select",
          keys: [<Icon name="IconReturnKey" />],
          onMouseDown: () => {
            state.accept(stack, modes);
          },
        });
      }

      ret.push(
        {
          name: "Back",
          keys: ["ESC"],
          onMouseDown: () => modes.deactivate(),
        },
      );

      return ret;
    },

    activate: (stack: Stack) => {
      state.currFilter.value = "";
      availOptionsFn(stack, state.availOptions);
      state.selected.value = state.availOptions.value[0];
    },

    Modal: ({ stack, modes }: { stack: Stack; modes: Modes }) => {
      const inputRef = useRef<HTMLInputElement>(null);
      const selectedRef = useRef<HTMLDivElement>(null);

      useEffect(() => {
        if (inputRef.current != null) {
          inputRef.current.focus();
        }
      }, []);

      useEffect(() => {
        if (selectedRef.current) {
          selectedRef.current.scrollIntoView({
            behavior: "smooth",
            block: "nearest",
          });
        }
      }, [selectedRef.current]);

      return (
        <div
          className={overlay}
          style={{
            position: "absolute",
            width: "40ch",
            // overflow: "auto",
            maxHeight: "10.5lh",
            //bottom: "0.25lh",
            bottom: "0",
            fontSize: "0.9rem",
            right: "4ch",
            // borderRadius: "0.5rem",
            borderRadius: "0.5rem 0.5rem 0 0",
            zIndex: 100,
            display: "flex",
            flexDirection: "column",
          }}
        >
          <div
            className={borderBottom}
            style="
        padding:1ch;
        width: 100%;
        "
          >
            <div style="width: 100%">
              <input
                type="text"
                ref={inputRef}
                onBlur={() => modes.deactivate()}
                placeholder="Search..."
                onInput={() => {
                  if (inputRef.current == null) return;
                  state.currFilter.value = inputRef.current.value;
                }}
                onKeyDown={(event) => {
                  event.stopPropagation();
                  switch (true) {
                    case event.key === "Enter":
                      event.preventDefault();
                      state.accept(stack, modes);
                      return;

                    case event.key === "Escape":
                      event.preventDefault();
                      modes.deactivate();
                      return;

                    case (event.ctrlKey && event.key === "n") ||
                      event.key === "ArrowDown":
                      event.preventDefault();
                      state.selectDown();
                      break;

                    case event.ctrlKey && event.key === "p" ||
                      event.key === "ArrowUp":
                      event.preventDefault();
                      state.selectUp();
                      break;
                  }
                }}
              />
            </div>
          </div>

          <div
            style={{
              overflow: "auto",
              flex: 1,
              padding: "1ch",
            }}
          >
            {state.options.value.map(
              (item) => {
                return (
                  <div
                    ref={item == state.selected.value ? selectedRef : null}
                    className={"terserow" +
                      (item == state.selected.value ? " hover" : "")}
                    style={{
                      display: "flex",
                      width: "100%",
                      overflow: "hidden",
                      padding: "0.5ch 0.75ch",
                      justifyContent: "space-between",
                      borderRadius: "6px",
                      cursor: "pointer",
                    }}
                    onMouseDown={() => {
                      state.selected.value = item;
                      state.accept(stack, modes);
                    }}
                  >
                    <div>
                      {item}
                    </div>
                  </div>
                );
              },
            )}
          </div>
        </div>
      );
    },
  };
}
