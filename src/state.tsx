import { signal } from "@preact/signals";

import { default as filterContentTypeMode } from "./modals/filterContentTypeMode";

const themeMode = signal("light");

const filter = (() => {
  const curr = signal("");
  let inputRef: HTMLInputElement | null = null;
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
    themeMode,
    filter,
}
