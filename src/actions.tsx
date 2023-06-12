import { Signal, useSignal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

import { borderBottom, iconStyle, overlay } from "./app.css.ts";

function ActionRow(
  { name, keys, isSelected }: {
    name: string;
    keys?: string[];
    isSelected: boolean;
  },
) {
  return (
    <div
      className={"terserow" + (isSelected ? " selected" : "")}
      style="
        display: flex;
        width: 100%;
        overflow: hidden;
        padding: 0.5ch 0.75ch;
        justify-content: space-between;
        border-radius: 6px;
        cursor: pointer;
        "
    >
      <div>
        {name}
      </div>
      <div>
        {keys
          ? keys.map((key, index) => (
            <span
              className={iconStyle}
              style={index !== keys.length - 1 ? { marginRight: "0.25ch" } : {}}
            >
              {key}
            </span>
          ))
          : ""}
      </div>
    </div>
  );
}

export function Actions({ showActions }: {
  showActions: Signal<boolean>;
}) {
  const inputRef = useRef<HTMLInputElement>(null);

  const selected = useSignal(0);

  useEffect(() => {
    selected.value = 0;
    if (inputRef.current != null) {
      inputRef.current.focus();
    }
  }, []);

  const actions = [{
    name: "Delete",
    keys: ["Ctrl", "DEL"],
  }, {
    name: "Microlink Screenshot",
  }];

  return (
    <div
      className={overlay}
      style={{
        position: "absolute",
        width: "40ch",
        overflow: "auto",
        //bottom: "0.25lh",
        bottom: "0",
        fontSize: "0.9rem",
        right: "4ch",
        // borderRadius: "0.5rem",
        borderRadius: "0.5rem 0.5rem 0 0",
        zIndex: 100,
      }}
    >
      <div
        className={borderBottom}
        style="
        padding:1ch;
        display: flex;
        width: 100%;
        align-items: center;
        "
      >
        <div style="width: 100%">
          <input
            type="text"
            ref={inputRef}
            onBlur={() => showActions.value = false}
            placeholder="Search..."
            onKeyDown={(event) => {
              event.stopPropagation();
              console.log("ACTIONS:", event);
              switch (true) {
                case event.key === "Escape":
                  event.preventDefault();
                  showActions.value = false;
                  break;

                case event.metaKey && event.key === "k":
                  event.preventDefault();
                  showActions.value = !showActions.value;
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
              }
            }}
          />
        </div>
      </div>

      <div style="
        padding:1ch;
        ">
        {actions.map((action, index) => (
          <ActionRow
            name={action.name}
            keys={action.keys}
            isSelected={selected.value == index}
          />
        ))}
      </div>
    </div>
  );
}
