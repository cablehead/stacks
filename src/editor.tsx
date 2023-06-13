import { style } from "@vanilla-extract/css";

import { Signal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

import { vars } from "./app.css.ts";

import { Item } from "./types.tsx";

export function Editor({ showEditor, item }: {
  showEditor: Signal<boolean>;
  item?: Item;
}) {
  const inputRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (inputRef.current != null) {
      inputRef.current.focus();
    }
  }, []);

  return (
    <div
      className={style({
  boxShadow: "0 0 6px " + vars.shadowColor})}
      style={{
        position: "absolute",
        overflow: "auto",
        fontSize: "0.9rem",
        bottom: "2ch",
        right: "2ch",
        left: "2ch",
        top: "2ch",
        borderRadius: "0.5rem",
        zIndex: 1000,
      }}
    >
      <textarea
        ref={inputRef}
        style={{
            width: "100%",
            height: "100%",
            padding: "1ch",
            margin: "1ch",
            }}
        onBlur={() => {
            console.log("peace");
            showEditor.value = false;
            }}
        placeholder="Search..."
        onInput={() => {
          if (inputRef.current == null) return;
        }}
        onKeyDown={(event) => {
          event.stopPropagation();
          console.log("Editor:", event);
        }}
      >
      </textarea>
    </div>
  );
}
