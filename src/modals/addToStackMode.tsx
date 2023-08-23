import { Signal, signal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

// import { invoke } from "@tauri-apps/api/tauri";

import { borderBottom, overlay } from "../ui/app.css";
import { Icon } from "../ui/icons";

import { Modes } from "./types";
import { ContentMeta, Item, Stack } from "../types";

interface ItemMeta {
  item: Item;
  meta: ContentMeta;
}

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

  const options: Signal<ItemMeta[]> = signal([]);

  const dn = signal("");

  return {
    selected,
    currFilter,
    options,
    dn,

    accept: (_stack: Stack, _modes: Modes) => {
      /*
      const item = stack.item.value;
      if (!item) return;
      const id = item.ids[item.ids.length - 1];
      if (!id) return;
      const name = options.value[normalizedSelected.value]?.terse;
      if (!name) return;
      (async () => {
        await invoke("store_add_to_stack", { name: name, id: id });
        stack.selected.value = Focus.first();
        modes.deactivate();
      })();
      */
    },

    accept_meta: (_stack: Stack, _modes: Modes) => {
      /*
      const item = stack.item.value;
      if (!item) return;
      const id = item.ids[item.ids.length - 1];
      if (!id) return;
      let name = currFilter.value;
      if (name === "") name = state.dn.value;
      if (name === "") return;
      (async () => {
        await invoke("store_add_to_stack", { name: name, id: id });
        stack.selected.value = Focus.first();
        modes.deactivate();
      })();
        */
    },

    selectDown: () => {
      const idx = options.value.findIndex((o) => o.item.id == selected.peek());
      if (idx < options.value.length - 1) {
        state.selected.value = options.value[idx + 1].item.id;
      }
    },

    selectUp: () => {
      const idx = options.value.findIndex((o) => o.item.id == selected.peek());
      if (idx > 0) {
        selected.value = options.value[idx - 1].item.id;
      }
    },
  };
})();

export default {
  name: () => "Add to stack",

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
    state.currFilter.value = "";
    state.options.value = stack.state.value.root
      .filter((id) => id != stack.item.value?.stack_id)
      .map((id) => {
        const item = stack.state.value.items[id];
        return { item: item, meta: stack.getContentMeta(item) };
      });
    state.selected.value = state.options.value[0].item.id;
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
            (o) => {
              return (
                <div
                  className={"terserow" +
                    (o.item.id == state.selected.value ? " hover" : "")}
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
                    // state.selected.value = index;
                    // state.accept(stack, modes);
                  }}
                >
                  <div>
                    {o.meta.terse}
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
