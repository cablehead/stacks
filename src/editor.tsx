import { useEffect, useRef } from "preact/hooks";

import { overlay } from "./app.css";

import { Item } from "./types";

import { editor, getContent } from "./state";

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
        editor.content = content;
      }
    }
    editor.content = "";
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
          editor.show.value = false;
        }}
        placeholder="..."
        onInput={() => {
          if (inputRef.current !== null) {
            editor.content = inputRef.current.value;
          }
        }}
        onKeyDown={(event) => {
          event.stopPropagation();
          switch (true) {
            case event.key === "Escape":
              event.preventDefault();
              editor.show.value = false;
              break;

            case event.metaKey && event.key === "e":
              event.preventDefault();
              editor.show.value = false;
              break;

            case event.metaKey && event.key === "Enter":
              editor.save();
              editor.show.value = false;
              break;
          }
        }}
      >
      </textarea>
    </div>
  );
}
