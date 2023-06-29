import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/shell";

import { editorMode, modes } from "./modals";

import { Icon } from "./ui/icons";
import { Item, LoadedItem } from "./types";

export const actions = [
  {
    name: "Edit",
    keys: [<Icon name="IconCommandKey" />, "E"],
    trigger: (_: LoadedItem) => modes.activate(editorMode),
    canApply: (item: Item) => item.mime_type === "text/plain",
  },
  {
    name: "Open",
    keys: [<Icon name="IconCommandKey" />, "O"],
    trigger: (loaded: LoadedItem) => open(loaded.content),
    canApply: (item: Item) => item.content_type === "Link",
  },
  {
    name: "Delete",
    keys: ["Ctrl", "DEL"],
    trigger: (loaded: LoadedItem) => invoke("store_delete", { hash: loaded.item.hash }),
  },
];

const trigger = (name: string, loaded: LoadedItem): void => {
  const action = actions.filter((action) => action.name === name)[0];
  if (action.canApply && !action.canApply(loaded.item)) return;
  if (action.trigger) action.trigger(loaded);
};

export const attemptAction = (event: KeyboardEvent, loaded: LoadedItem): boolean => {
  switch (true) {
    case (event.ctrlKey && event.key === "Backspace"):
      event.preventDefault();
      trigger("Delete", loaded);
      return true;

    case (event.metaKey && event.key === "e"):
      event.preventDefault();
      trigger("Edit", loaded);
      return true;

    case (event.metaKey && event.key === "o"):
      event.preventDefault();
      trigger("Open", loaded);
      return true;
  }

  return false;
};
