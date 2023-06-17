import { useEffect, useRef } from "preact/hooks";

import { borderBottom, borderRight } from "../ui/app.css";
import { Icon, RenderKeys } from "../ui/icons";

import { filter } from "../state";

export function Filter() {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (inputRef.current != null) {
      inputRef.current.focus();
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
      <div>/</div>
      <div
        style={{
          flexGrow: "1",
        }}
      >
        <input
          type="text"
          placeholder="Type a filter..."
          ref={inputRef}
          onInput={() => {
            if (inputRef.current == null) return;
            filter.curr.value = inputRef.current.value;
            // updateSelected(0);
          }}
        />
      </div>

      <VertDiv />
      <div
        class="hoverable"
        onMouseDown={() => null}
        style={{
          marginRight: "4ch",
          fontSize: "0.9rem",
          display: "flex",
          alignItems: "center",
        }}
      >
        Content Type&nbsp;
        <RenderKeys keys={[<Icon name="IconCommandKey" />, "P"]} />
      </div>
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
