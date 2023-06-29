import { signal } from "@preact/signals";


import { Icon } from "../ui/icons";

import { Modes } from "./types";

import { default as actionsMode } from "./actionsMode";
import { default as filterContentTypeMode } from "./filterContentTypeMode";

export const themeMode = signal("light");

let focusSelectedTimeout: number | undefined;

export function focusSelected(delay: number) {
  if (focusSelectedTimeout !== undefined) {
    return;
  }

  focusSelectedTimeout = window.setTimeout(() => {
    focusSelectedTimeout = undefined;
    const selectedItem = document.querySelector(
      `.terserow.selected`,
    );
    if (selectedItem) {
      selectedItem.scrollIntoView({
        behavior: "smooth",
        block: "nearest",
      });
    }
  }, delay);
}

export const state = (() => {
  const curr = signal("");
  let inputRef: HTMLInputElement | null = null;

  /*
   * todo:
  effect(() => {
    invoke<Item[]>("store_set_filter", {
      curr: curr.value,
      contentType: filterContentTypeMode.curr.value,
    });
  });
  */

  return {
    curr,
    dirty: () => curr.value != "" || filterContentTypeMode.curr.value != "All",
    clear: () => {
      if (inputRef) inputRef.value = "";
      curr.value = "";
      filterContentTypeMode.curr.value = "All";
    },
    get input(): HTMLInputElement | null {
      return inputRef;
    },
    set input(ref: HTMLInputElement | null) {
      inputRef = ref;
    },
  };
})();

export default {
  name: "Clipboard",
  state: state,
  hotKeys: (modes: Modes) => [
    {
      name: "Copy",
      keys: [<Icon name="IconReturnKey" />],
      onMouseDown: () => {
      },
    },

    {
      name: "Actions",
      keys: [<Icon name="IconCommandKey" />, "K"],
      onMouseDown: () => {
        modes.toggle(actionsMode);
      },
    },

    ...(state.dirty()
      ? [
        {
          name: "Clear filter",
          keys: ["ESC"],
          onMouseDown: () => {
            state.clear();
          },
        },
      ]
      : []),
  ],
};
