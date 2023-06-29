import { useEffect, useRef } from "preact/hooks";

import { writeText } from "@tauri-apps/api/clipboard";

import { overlay } from "../ui/app.css";

import { modes } from "../modals";

export function Editor({ content }: {
  content: string;
}) {
  const inputRef = useRef<HTMLTextAreaElement>(null);
  useEffect(() => {
    if (inputRef.current != null) {
      inputRef.current.focus();
    }
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
              if (inputRef.current !== null) {
                writeText(inputRef.current.value);
              modes.deactivate();
              }
              break;
          }
        }}
      >
        {content}
      </textarea>
    </div>
  );
}
