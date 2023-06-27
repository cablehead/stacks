import { useEffect, useRef } from "preact/hooks";

import { overlay } from "../ui/app.css";

import { getContent, Item } from "../state";

import { modes, editorMode } from "../modes";

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
        editorMode.content = content;
      }
    }
    editorMode.content = "";
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
            modes.deactivate();
        }}
        placeholder="..."
        onInput={() => {
          if (inputRef.current !== null) {
            editorMode.content = inputRef.current.value;
          }
        }}
        onKeyDown={(event) => {
          event.stopPropagation();
          switch (true) {
            case event.key === "Escape":
              event.preventDefault();
            modes.deactivate();
              break;

            case event.metaKey && event.key === "e":
              event.preventDefault();
            modes.deactivate();
              break;

            case event.metaKey && event.key === "Enter":
              editorMode.save();
              break;
          }
        }}
      >
      </textarea>
    </div>
  );
}
