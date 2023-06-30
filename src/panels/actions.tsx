import { Signal, useComputed, useSignal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

import { borderBottom, overlay } from "../ui/app.css";
import { RenderKeys } from "../ui/icons";

import { Action, Stack } from "../types";

import { actionsMode, modes } from "../modals";

import { actions, attemptAction } from "../actions";

function ActionRow(
  { action, isSelected, stack }: {
    action: Action;
    isSelected: boolean;
    stack: Stack;
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
        if (action.trigger) action.trigger(stack);
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

export function Actions({ stack }: {
  stack: Stack;
}) {
  const inputRef = useRef<HTMLInputElement>(null);

  const selected = useSignal(0);
  const currFilter = useSignal("");

  useEffect(() => {
    if (inputRef.current != null) {
      inputRef.current.focus();
    }
  }, []);

  const actionsAvailable: Signal<Action[]> = useComputed(() => {
    return actions
      .filter((action) => {
        if (currFilter.value === "") {
          return true;
        }
        return action.name.toLowerCase().includes(
          currFilter.value.toLowerCase(),
        );
      })
      .filter((action) => !action.canApply || action.canApply(stack));
  });

  const normalizedSelected = useComputed(() => {
    let val = selected.value % (actionsAvailable.value.length);
    if (val < 0) val = actionsAvailable.value.length + val;
    return val;
  });

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
            onBlur={() => modes.deactivate()}
            placeholder="Search..."
            onInput={() => {
              if (inputRef.current == null) return;
              currFilter.value = inputRef.current.value;
            }}
            onKeyDown={(event) => {
              console.log("ACTIONS", event);
              event.stopPropagation();
              switch (true) {
                case event.key === "Escape":
                  event.preventDefault();
                  modes.deactivate();
                  break;

                case event.key === "Enter":
                  event.preventDefault();
                  modes.deactivate();
                  const action =
                    actionsAvailable.value[normalizedSelected.value];
                  if (!action || !action.trigger) return;
                  action.trigger(stack);
                  break;

                case event.metaKey && event.key === "k":
                  event.preventDefault();
                  modes.toggle(actionsMode);
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

                default:
                  attemptAction(event, stack);
              }
            }}
          />
        </div>
      </div>

      <div style="
        padding:1ch;
        ">
        {actionsAvailable.value
          .map((action, index) => (
            <ActionRow
              action={action}
              isSelected={normalizedSelected.value == index}
              stack={stack}
            />
          ))}
      </div>
    </div>
  );
}
