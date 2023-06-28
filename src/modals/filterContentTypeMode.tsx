import { useEffect, useRef } from "preact/hooks";
import { computed, signal } from "@preact/signals";

import { overlay } from "../ui/app.css";
import { Icon } from "../ui/icons";

import { Modes } from "./types";

const state = (() => {
  const options = ["All", "Stacks", "Links", "Images"];
  const curr = signal("All");
  const selected = signal(0);
  const normalizedSelected = computed(() => {
    let val = selected.value % (options.length);
    if (val < 0) val = options.length + val;
    return val;
  });
  return {
    options,
    curr,
    selected,
    normalizedSelected,
    accept: (modes: Modes) => {
      state.curr.value = state.options[state.normalizedSelected.value];
      modes.deactivate();
    },
  };
})();

export default {
  name: "Filter by content type",

  curr: state.curr,

  hotKeys: (modes: Modes) => [
    {
      name: "Select",
      keys: [<Icon name="IconReturnKey" />],
      onMouseDown: () => {
        state.accept(modes);
      },
    },
    {
      name: "Back",
      keys: ["ESC"],
      onMouseDown: () => modes.deactivate(),
    },
  ],

  activate: () => {
    const idx = state.options.indexOf(state.curr.value);
    state.selected.value = idx == -1 ? 0 : idx;
  },

  Modal: ({ modes }: { modes: Modes }) => {
    const { options, normalizedSelected, selected, curr } = state;
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
          overflow: "auto",
          top: "7.5ch",
          fontSize: "0.9rem",
          padding: "1ch",
          right: "4.2ch",
          borderRadius: "0.5rem",
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

                case (event.metaKey && event.key === "p"):
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
                  state.accept(modes);
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
                curr.value = options[index];
                modes.deactivate();
              }}
            >
              {option}
            </div>
          ))}
      </div>
    );
  },
};
