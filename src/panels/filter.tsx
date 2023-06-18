import { useEffect, useRef } from "preact/hooks";

import { borderBottom, borderRight, overlay } from "../ui/app.css";
import { Icon, RenderKeys } from "../ui/icons";

import { filter } from "../state";

export function Filter() {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (inputRef.current != null) {
      inputRef.current.focus();
      filter.input = inputRef.current;
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
          }}
        />
      </div>

      <VertDiv />
      <div
        class="hoverable"
        onMouseDown={() =>
          filter.contentType.show.value = !filter.contentType.show.value}
        style={{
          marginRight: "4ch",
          fontSize: "0.9rem",
          display: "flex",
          alignItems: "center",
        }}
      >
        {filter.contentType.curr.value == "All"
          ? "Content Type"
          : filter.contentType.curr.value}&nbsp;
        <RenderKeys keys={[<Icon name="IconCommandKey" />, "P"]} />
      </div>

      {filter.contentType.show.value && <ContentType />}
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

function ContentType() {
  const inputRef = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (inputRef.current != null) {
      inputRef.current.focus();
    }
  }, []);

  const { options, normalizedSelected, selected, show, curr } =
    filter.contentType;

  return (
    <div
      className={overlay}
      style={{
        position: "absolute",
        width: "20ch",
        overflow: "auto",
        top: "7.5ch",
        fontSize: "0.9rem",
        padding: "1ch",
        right: "8.2ch",
        borderRadius: "0.5rem",
        zIndex: 100,
      }}
    >
      <div style="
      width: 0;
      height: 0;
      overflow: hidden;
       ">
        <input
          ref={inputRef}
          onKeyDown={(event) => {
            event.stopPropagation();
            switch (true) {
              case event.key === "Escape":
                event.preventDefault();
                show.value = false;
                break;

              case (event.metaKey && event.key === "p"):
                event.preventDefault();
                show.value = false;
                break;

              case (event.ctrlKey && event.key === "n") ||
                event.key === "ArrowDown":
                event.preventDefault();
                selected.value += 1;
                break;

              case event.ctrlKey && event.key === "p" ||
                event.key === "ArrowUp":
                event.preventDefault();
                selected.value -= 1;
                break;

              case event.key === "Enter":
                event.preventDefault();
                curr.value = options[normalizedSelected.value];
                show.value = false;
                break;
            }
          }}
          onBlur={() => show.value = false}
        />
      </div>
      {options
        .map((option, index) => (
          <div
            style="
            border-radius: 6px;
            cursor: pointer;
            padding: 0.5ch 0.75ch;
            "
            className={"terserow" + (
              normalizedSelected.value == index ? " hover" : ""
            )}
            onMouseDown={() => {
              selected.value = index;
              curr.value = options[index];
              show.value = false;
            }}
          >
            {option}
          </div>
        ))}
    </div>
  );
}
