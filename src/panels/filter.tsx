import { useEffect, useRef } from "preact/hooks";


import { borderBottom, borderRight } from "../ui/app.css";
import { Icon, RenderKeys } from "../ui/icons";

import { filterContentTypeMode, modes } from "../modals";
import { default as state } from "../state";


export function Filter() {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (inputRef.current != null) {
      inputRef.current.focus();
      state.filter.input = inputRef.current;
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
            state.filter.curr.value = inputRef.current.value;
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
        {filterContentTypeMode.curr.value == "All"
          ? "Content type"
          : filterContentTypeMode.curr.value}&nbsp;
        <RenderKeys keys={[<Icon name="IconCommandKey" />, "P"]} />
      </div>

      {modes.isActive(filterContentTypeMode) &&
        <filterContentTypeMode.Modal modes={modes} />}
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
