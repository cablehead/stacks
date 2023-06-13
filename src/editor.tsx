import { useEffect, useRef } from "preact/hooks";

import { overlay } from "./app.css.ts";

import { Item } from "./types.tsx";

import { writeText } from "@tauri-apps/api/clipboard";

import { getContent, showEditor } from "./state.tsx";

export function Editor({ item }: {
  item: Item;
}) {
  const inputRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (inputRef.current != null) {
      inputRef.current.focus();
    }

    async function fetchContent() {
      let content = await getContent(item.hash);
      if (inputRef.current != null) {
        inputRef.current.value = content;
      }
    }
    fetchContent();
  }, []);

  return (
    <div
      className={overlay}
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
          margin: "2ch",
          outline: "none",
          border: "none",
        }}
        onBlur={() => {
          showEditor.value = false;
        }}
        placeholder="..."
        onInput={() => {
          if (inputRef.current == null) return;
        }}
        onKeyDown={(event) => {
          event.stopPropagation();
          switch (true) {
            case event.key === "Escape":
              event.preventDefault();
              showEditor.value = false;
              break;

            case event.metaKey && event.key === "e":
              event.preventDefault();
              showEditor.value = false;
              break;

            case event.metaKey && event.key === "Enter":
              if (inputRef.current !== null) writeText(inputRef.current.value);
              showEditor.value = false;
              break;
          }
        }}
      >
      </textarea>
    </div>
  );
}
