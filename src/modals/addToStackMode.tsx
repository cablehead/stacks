import { computed, effect, Signal, signal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

import { invoke } from "@tauri-apps/api/tauri";

import { borderBottom, overlay } from "../ui/app.css";
import { Icon } from "../ui/icons";

import { Modes } from "./types";
import { Item, Stack } from "../types";

const state = (() => {
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
    fetchOptions(currFilter.value);
  });

  return {
    selected,
    currFilter,
    options,
    normalizedSelected,

    accept: (stack: Stack, modes: Modes) => {
      const item = stack.item.value;
      if (!item) return;
      const id = item.ids[item.ids.length - 1];
      if (!id) return;
      const name = options.value[normalizedSelected.value]?.terse;
      if (!name) return;
      (async () => {
        await invoke("store_add_to_stack", { name: name, id: id });
        stack.selected.value = 0;
        modes.deactivate();
      })();
    },

    accept_meta: (stack: Stack, modes: Modes) => {
      const item = stack.item.value;
      if (!item) return;
      const id = item.ids[item.ids.length - 1];
      if (!id) return;
      const name = currFilter.value;
      if (name === "") return;
      (async () => {
        await invoke("store_add_to_stack", { name: name, id: id });
        stack.selected.value = 0;
        modes.deactivate();
      })();
    },
  };
})();

export default {
  name: "Add to stack",

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

    if (state.currFilter.value !== "") {
      ret.push({
        name: "Create new",
        keys: [
          <Icon name="IconCommandKey" />,
          <Icon name="IconReturnKey" />,
        ],
        onMouseDown: () => {
          state.accept_meta(stack, modes);
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

  activate: (_: Stack) => {
    state.currFilter.value = "";
    state.selected.value = 0;
  },

  Modal: ({ stack, modes }: { stack: Stack; modes: Modes }) => {
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
                  case event.metaKey && event.key === "Enter":
                    state.accept_meta(stack, modes);
                    return;

                  case event.key === "Enter":
                    state.accept(stack, modes);
                    return;

                  case event.key === "Escape":
                    event.preventDefault();
                    modes.deactivate();
                    return;

                  case (event.ctrlKey && event.key === "n") ||
                    event.key === "ArrowDown":
                    event.preventDefault();
                    state.selected.value += 1;
                    break;

                  case event.ctrlKey && event.key === "p" ||
                    event.key === "ArrowUp":
                    event.preventDefault();
                    state.selected.value -= 1;
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
