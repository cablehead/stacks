import { computed, effect, Signal, signal } from "@preact/signals";
import { useEffect, useMemo, useRef } from "preact/hooks";

import { invoke } from "@tauri-apps/api/tauri";

import { borderBottom, overlay } from "../ui/app.css";
import { Icon } from "../ui/icons";

import { Modes } from "./types";
import { Item, Stack } from "../types";

export default {
  name: "Add to stack",

  hotKeys: (_: Stack, modes: Modes) => [
    {
      name: "Select",
      keys: [<Icon name="IconReturnKey" />],
      onMouseDown: () => {
      },
    },
    {
      name: "Create new",
      keys: [
        <Icon name="IconCommandKey" />,
        <Icon name="IconReturnKey" />,
      ],
      onMouseDown: () => {
      },
    },
    {
      name: "Back",
      keys: ["ESC"],
      onMouseDown: () => modes.deactivate(),
    },
  ],

  Modal: ({ modes }: { modes: Modes }) => {
    const inputRef = useRef<HTMLInputElement>(null);

    useEffect(() => {
      if (inputRef.current != null) {
        inputRef.current.focus();
      }
    }, []);

    const state = useMemo(() => {
      const selected = signal(0);
      const currFilter = signal("");
      const options: Signal<Item[]> = signal([]);

      const normalizedSelected = computed(() => {
        let val = selected.value % (options.value.length);
        if (val < 0) val = options.value.length + val;
        return val;
      });

      async function fetchOptions(filter: string) {
        options.value = await invoke("store_list_stacks", { filter: filter });
      }

      effect(() => {
        console.log("EFFECT", currFilter.value);
        fetchOptions(currFilter.value);
      });

      return {
        selected,
        currFilter,
        options,
        normalizedSelected,
      };
    }, []);

    return (
      <div
        className={overlay}
        style={{
          position: "absolute",
          width: "40ch",
          overflow: "auto",
          //bottom: "0.25lh",
          bottom: "0",
          fontSize: "0.9rem",
          right: "4ch",
          // borderRadius: "0.5rem",
          borderRadius: "0.5rem 0.5rem 0 0",
          zIndex: 100,
        }}
      >
        <div
          className={borderBottom}
          style="
        padding:1ch;
        display: flex;
        width: 100%;
        align-items: center;
        "
        >
          <div style="width: 100%">
            <input
              type="text"
              ref={inputRef}
              onBlur={() => modes.deactivate()}
              placeholder="Stack name..."
              onInput={() => {
                if (inputRef.current == null) return;
                state.currFilter.value = inputRef.current.value;
              }}
              onKeyDown={(event) => {
                event.stopPropagation();
                switch (true) {
                  case event.key === "Escape":
                    event.preventDefault();
                    modes.deactivate();
                    break;
                }
              }}
            />
          </div>
        </div>

        <div style="
        padding:1ch;
        ">
          {state.options.value
            .map((item, index) => (
              <Row
                item={item}
                isSelected={state.normalizedSelected.value == index}
              />
            ))}
        </div>
      </div>
    );
  },
};

function Row(
  { item, isSelected }: {
    item: Item;
    isSelected: boolean;
  },
) {
  return (
    <div
      className={"terserow" + (isSelected ? " hover" : "")}
      style="
        display: flex;
        width: 100%;
        overflow: hidden;
        padding: 0.5ch 0.75ch;
        justify-content: space-between;
        border-radius: 6px;
        cursor: pointer;
        "
      onMouseDown={() => {
      }}
    >
      <div>
        {item.terse}
      </div>
    </div>
  );
}
