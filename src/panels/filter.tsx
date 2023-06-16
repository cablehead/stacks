import { useEffect, useRef } from "preact/hooks";

import { borderBottom } from "../ui/app.css";

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
      style="
        padding:1ch;
        padding-left:2ch;
        padding-right:2ch;
        padding-bottom:0.5ch;
        display: flex;
    width: 100%;
        align-items: center;
        "
    >
      <div>/</div>
      <div style="width: 100%">
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
    </div>
  );
}
