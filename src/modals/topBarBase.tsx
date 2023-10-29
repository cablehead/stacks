import { useEffect, useRef } from "preact/hooks";
import { computed, signal } from "@preact/signals";

import { overlay } from "../ui/app.css";
import { Icon } from "../ui/icons";

import { Modes } from "./types";
import { Stack } from "../types";

type Options = {
  name: () => string;
  options: string[];
  rightOffset: () => number;
  accept: (stack: Stack, modes: Modes, chosen: string) => void;
  // activate: (stack: Stack, selected: typeof signal, normalizedSelected: typeof computed) => void,
  // hotKeys: (stack: Stack, modes: Modes) => any,
  // positioning: () => number,
};

export function createModal(opt: Options) {
  const state = (() => {
    const options = opt.options;
    const selected = signal(0);
    const normalizedSelected = computed(() => {
      let val = selected.value % (options.length);
      if (val < 0) val = options.length + val;
      return val;
    });
    return {
      options,
      selected,
      normalizedSelected,
      accept: (stack: Stack, modes: Modes) => {
        opt.accept(stack, modes, options[normalizedSelected.value]);
      },
    };
  })();

  const modal = {
    name: opt.name,

    hotKeys: (stack: Stack, modes: Modes) => [
      {
        name: "Select",
        keys: [<Icon name="IconReturnKey" />],
        onMouseDown: () => {
          state.accept(stack, modes);
        },
      },
      {
        name: "Back",
        keys: ["ESC"],
        onMouseDown: () => modes.deactivate(),
      },
    ],

    activate: (stack: Stack) => {
      const idx = state.options.indexOf(stack.filter.content_type.value);
      state.selected.value = idx == -1 ? 0 : idx;
    },

    Modal: ({ stack, modes }: { stack: Stack; modes: Modes }) => {
      const { options, normalizedSelected, selected } = state;
      const inputRef = useRef<HTMLInputElement>(null);

      useEffect(() => {
        if (inputRef.current != null) {
          inputRef.current.focus();
        }
      }, []);

      return (
        <div
          className={overlay}
          style={{
            position: "absolute",
            width: "20ch",
            overflow: "hidden",
            top: "0",
            fontSize: "0.9rem",
            padding: "1ch",
            right: opt.rightOffset(),
            borderRadius: "0 0 0.5rem 0.5rem",
            zIndex: 100,
          }}
        >
          <div style="
      width: 0;
      height: 0;
      overflow: hidden;
       ">
            <input
              ref={inputRef}
              onKeyDown={(event) => {
                event.stopPropagation();
                switch (true) {
                  case event.key === "Escape":
                    event.preventDefault();
                    modes.deactivate();
                    break;

                  case (event.metaKey && event.key === "u"):
                    event.preventDefault();
                    modes.deactivate();
                    break;

                  case (event.ctrlKey && event.key === "n") ||
                    event.key === "ArrowDown":
                    event.preventDefault();
                    selected.value += 1;
                    break;

                  case event.ctrlKey && event.key === "p" ||
                    event.key === "ArrowUp":
                    event.preventDefault();
                    selected.value -= 1;
                    break;

                  case event.key === "Enter":
                    event.preventDefault();
                    state.accept(stack, modes);
                    break;
                }
              }}
              onBlur={() => modes.deactivate()}
            />
          </div>
          {options
            .map((option, index) => (
              <div
                style="
            border-radius: 6px;
            cursor: pointer;
            padding: 0.5ch 0.75ch;
            "
                className={"terserow" + (
                  normalizedSelected.value == index ? " hover" : ""
                )}
                onMouseDown={() => {
                  selected.value = index;
                  state.accept(stack, modes);
                }}
              >
                {option}
              </div>
            ))}
        </div>
      );
    },
  };

  return modal;
}
