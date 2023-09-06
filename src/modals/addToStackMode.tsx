import { computed, effect, Signal, signal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

import { invoke } from "@tauri-apps/api/tauri";

import { borderBottom, overlay } from "../ui/app.css";
import { Icon } from "../ui/icons";

import { Modes } from "./types";
import { Item, itemGetTerse, Stack } from "../types";

function dn(): string {
  const date = new Date();
  const options: Intl.DateTimeFormatOptions = {
    weekday: "short",
    year: "numeric",
    month: "short",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: true,
    timeZoneName: "short",
  };
  const formattedDate = new Intl.DateTimeFormat("en-US", options).format(
    date,
  );
  return formattedDate;
}

const state = (() => {
  const selected = signal("");
  const currFilter = signal("");

  const availOptions: Signal<Item[]> = signal([]);

  const options = computed(() =>
    availOptions.value
      .filter((item) =>
        currFilter.value == "" ||
        itemGetTerse(item).toLowerCase().includes(
          currFilter.value.toLowerCase(),
        )
      )
  );

  effect(() => {
    if (options.value.length <= 0) return;
    const chosen = options.value.find((item) => item.id === selected.peek());
    if (!chosen) selected.value = options.value[0].id;
  });

  const dn = signal("");

  return {
    selected,
    currFilter,
    availOptions,
    options,
    dn,

    accept: (stack: Stack, modes: Modes) => {
      const item = stack.selected();
      if (!item) return;

      const chosen = options.value.find((item) => item.id === selected.value);
      if (!chosen) return;
      console.log("Accept", selected.value, chosen);

      (async () => {
        await invoke("store_add_to_stack", {
          stackId: chosen.id,
          sourceId: item.id,
        });
        modes.deactivate();
      })();
    },

    accept_meta: (stack: Stack, modes: Modes) => {
      const item = stack.selected();
      if (!item) return;

      let name = currFilter.value;
      if (name === "") name = state.dn.value;
      if (name === "") return;

      (async () => {
        await invoke("store_add_to_new_stack", {
          name: name,
          sourceId: item.id,
        });
        modes.deactivate();
      })();
    },

    selectDown: () => {
      const idx = options.value.findIndex((item) => item.id == selected.peek());
      if (idx < options.value.length - 1) {
        state.selected.value = options.value[idx + 1].id;
      }
    },

    selectUp: () => {
      const idx = options.value.findIndex((item) => item.id == selected.peek());
      if (idx > 0) {
        selected.value = options.value[idx - 1].id;
      }
    },
  };
})();

export default {
  name: () => "Copy to stack",

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
    if (!stack.nav.value.root) return;
    const selected = stack.selected();
    if (!selected) return;

    state.currFilter.value = "";

    state.availOptions.value = stack.nav.value.root.items
      .filter((item) => item.id != selected.stack_id);

    stack.getRoot().then((items) =>
      state.availOptions.value = items
        .filter((item) => item.id != selected.stack_id)
    );

    state.selected.value = state.options.value[0]?.id || "";
    state.dn.value = dn();
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
          maxHeight: "10lh",
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
              placeholder={state.dn.value}
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

        <div style="
        padding:1ch;
        ">
          {state.options.value.map(
            (item) => {
              return (
                <div
                  className={"terserow" +
                    (item.id == state.selected.value ? " hover" : "")}
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
                    state.selected.value = item.id;
                    state.accept(stack, modes);
                  }}
                >
                  <div>
                    {itemGetTerse(item)}
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
