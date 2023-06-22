import { signal, useSignal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

import { borderBottom, overlay } from "../ui/app.css";

export const state = {
  show: signal(false),
};

/*
function ActionRow(
  { action, isSelected, item }: {
    action: Action;
    isSelected: boolean;
    item: Item;
  },
) {
  return (
    <div
      className={"terserow" + (isSelected ? " hover" : "")}
      style="
        display: flex;
        width: 100%;
        overflow: hidden;
        padding: 0.5ch 0.75ch;
        justify-content: space-between;
        border-radius: 6px;
        cursor: pointer;
        "
      onMouseDown={() => {
        if (action.trigger) action.trigger(item);
      }}
    >
      <div>
        {action.name}
      </div>
      <div>
        {action.keys ? <RenderKeys keys={action.keys} /> : ""}
      </div>
    </div>
  );
}
*/

export function AddToStack() {
  const inputRef = useRef<HTMLInputElement>(null);

  const selected = useSignal(0);
  const currFilter = useSignal("");

  useEffect(() => {
    selected.value = 0;
    if (inputRef.current != null) {
      inputRef.current.focus();
    }
  }, []);

  /*

  const normalizedSelected = useComputed(() => {
    let val = selected.value % (actionsAvailable.value.length);
    if (val < 0) val = actionsAvailable.value.length + val;
    return val;
  });
  */

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
            placeholder="Stack name..."
            onInput={() => {
              if (inputRef.current == null) return;
              currFilter.value = inputRef.current.value;
            }}
          />
        </div>
      </div>

      <div style="
        padding:1ch;
        ">
        {
          /*actionsAvailable.value
          .map((action, index) => (
            <ActionRow
              action={action}
              isSelected={normalizedSelected.value == index}
              item={item}
            />
          )) */
        }
      </div>
    </div>
  );
}
