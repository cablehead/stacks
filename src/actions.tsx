import { Signal, useComputed, useSignal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

import { borderBottom, iconStyle, overlay } from "./app.css.ts";

import { JSXInternal } from "preact/src/jsx";

import { invoke } from "@tauri-apps/api/tauri";
import { open } from '@tauri-apps/api/shell';

import { Item } from "./types.tsx";
import { Icon } from "./icons.tsx";

import { getContent, editor } from "./state.tsx";

interface Action {
  name: string;
  keys?: (string | JSXInternal.Element)[];
  trigger?: (item: Item) => void;
  canApply?: (item: Item) => boolean;
}

/*
async function microlink_screenshot(item: Item): Promise<boolean> {
  console.log("MICROLINK");
  const content = await getContent(item.hash);
  const err = await invoke<string | undefined>("microlink_screenshot", {
    url: content,
  });
  console.log(content, err);
  if (err) {
    alert(err);
    return false;
  }
  return true;
}
*/

async function open_link(item: Item) {
  const url = await getContent(item.hash);
  await open(url);
}

const actions = [
  {
    name: "Edit",
    keys: [<Icon name="IconCommandKey" />, "E"],
    trigger: (item: Item) => editor.show.value = true,
    canApply: (item: Item) => item.mime_type === "text/plain",
  },
  {
    name: "Open",
    keys: [<Icon name="IconCommandKey" />, "O"],
    trigger: open_link,
    canApply: (item: Item) => item.content_type === "Link",
  },
  /*
  {
    name: "Microlink Screenshot",
    trigger: microlink_screenshot,
    canApply: (item: Item) => item.content_type === "Link",
  },
  */
  {
    name: "Delete",
    keys: ["Ctrl", "DEL"],
    trigger: (item: Item) => invoke("store_delete", { hash: item.hash }),
  },
];

const trigger = (name: string, item: Item): void => {
  const action = actions.filter((action) => action.name === name)[0];
  if (action.canApply && !action.canApply(item)) return;
  if (action.trigger) action.trigger(item);
};

export const attemptAction = (event: KeyboardEvent, item: Item): boolean => {
  switch (true) {
    case (event.ctrlKey && event.key === "Backspace"):
      event.preventDefault();
      trigger("Delete", item);
      return true;

    case (event.metaKey && event.key === "e"):
      event.preventDefault();
      trigger("Edit", item);
      return true;

    case (event.metaKey && event.key === "o"):
      event.preventDefault();
      trigger("Open", item);
      return true;
  }

  return false;
};

function RenderKeys({ keys }: { keys: (string | JSXInternal.Element)[] }) {
  return (
    <>
      {keys.map((key, index) => (
        <span
          className={iconStyle}
          style={index !== keys.length - 1 ? { marginRight: "0.25ch" } : {}}
        >
          {key}
        </span>
      ))}
    </>
  );
}

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

export function Actions({ showActions, item }: {
  showActions: Signal<boolean>;
  item: Item;
}) {
  const inputRef = useRef<HTMLInputElement>(null);

  const selected = useSignal(0);
  const currFilter = useSignal("");

  useEffect(() => {
    selected.value = 0;
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
      .filter((action) => !action.canApply || action.canApply(item));
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
            onBlur={() => showActions.value = false}
            placeholder="Search..."
            onInput={() => {
              if (inputRef.current == null) return;
              currFilter.value = inputRef.current.value;
            }}
            onKeyDown={(event) => {
              event.stopPropagation();
              switch (true) {
                case event.key === "Escape":
                  event.preventDefault();
                  showActions.value = false;
                  break;

                case event.key === "Enter":
                  event.preventDefault();
                  showActions.value = false;
                  const action =
                    actionsAvailable.value[normalizedSelected.value];
                  if (!action || !action.trigger) return;
                  action.trigger(item);
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

                default:
                  if (attemptAction(event, item)) showActions.value = false;
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
              item={item}
            />
          ))}
      </div>
    </div>
  );
}
