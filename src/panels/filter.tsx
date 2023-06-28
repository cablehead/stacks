import { useEffect, useRef } from "preact/hooks";
import { effect, signal } from "@preact/signals";

import { invoke } from "@tauri-apps/api/tauri";

import { borderBottom, borderRight } from "../ui/app.css";
import { Icon, RenderKeys } from "../ui/icons";

import { Item } from "../state";
import { modes } from "../modes";
import { filterContentTypeMode } from "../modals";

export const state = (() => {
  const curr = signal("");
  let inputRef: HTMLInputElement | null = null;

  effect(() => {
    invoke<Item[]>("store_set_filter", {
      curr: curr.value,
      contentType: filterContentTypeMode.value(),
    });
  });

  return {
    curr,
    dirty: () => curr.value != "", /* || contentType.curr.value != "All" */
    clear: () => {
      if (inputRef) inputRef.value = "";
      curr.value = "";
      // contentType.selected.value = 0;
      // contentType.curr.value = "All";
    },
    get input(): HTMLInputElement | null {
      return inputRef;
    },
    set input(ref: HTMLInputElement | null) {
      inputRef = ref;
    },
  };
})();

export function Filter() {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (inputRef.current != null) {
      inputRef.current.focus();
      state.input = inputRef.current;
    }
  }, []);

  return (
    <div
      className={borderBottom}
      style={{
        padding: "1ch",
        paddingLeft: "2ch",
        height: "5ch",
        paddingBottom: "0.25ch",
        display: "flex",
        width: "100%",
        gap: "0.5ch",
        alignItems: "center",
      }}
    >
      <div>&gt;</div>
      <div
        style={{
          flexGrow: "1",
        }}
      >
        <input
          type="text"
          placeholder="Type to filter..."
          ref={inputRef}
          onInput={() => {
            if (inputRef.current == null) return;
            state.curr.value = inputRef.current.value;
          }}
        />
      </div>

      <VertDiv />
      <div
        class="hoverable"
        onMouseDown={() => modes.toggle(filterContentTypeMode)}
        style={{
          fontSize: "0.9rem",
          display: "flex",
          alignItems: "center",
        }}
      >
        {filterContentTypeMode.value() == "All"
          ? "Content type"
          : filterContentTypeMode.value()}&nbsp;
        <RenderKeys keys={[<Icon name="IconCommandKey" />, "P"]} />
      </div>

      {modes.isActive(filterContentTypeMode) &&
        filterContentTypeMode.Model(modes)}
    </div>
  );
}

const VertDiv = () => (
  <div
    className={borderRight}
    style={{
      width: "1px",
      height: "1.5em",
    }}
  />
);
