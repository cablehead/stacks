import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/shell";

import { editorMode, modes } from "./modals";

import { Icon } from "./ui/icons";
import { Stack } from "./types";

export const actions = [
  {
    name: "Edit",
    keys: [<Icon name="IconCommandKey" />, "E"],
    trigger: (stack: Stack) => modes.activate(stack, editorMode),
    canApply: (stack: Stack) => stack.item.value?.mime_type === "text/plain",
  },
  {
    name: "Open",
    keys: [<Icon name="IconCommandKey" />, "O"],
    trigger: (stack: Stack) => {
      const content = stack.content?.value;
      if (content) open(content);
    },
    canApply: (stack: Stack) => stack.item.value?.content_type === "Link",
  },
  {
    name: "Delete",
    keys: ["Ctrl", "DEL"],
    trigger: (stack: Stack) => {
      const item = stack.item.value;
      if (item) invoke("store_delete", { hash: item.hash });
    },
  },
];

const trigger = (name: string, stack: Stack): void => {
  const action = actions.filter((action) => action.name === name)[0];
  if (action.canApply && !action.canApply(stack)) return;
  if (action.trigger) action.trigger(stack);
};

export const attemptAction = (event: KeyboardEvent, stack: Stack): boolean => {
  switch (true) {
    case (event.ctrlKey && event.key === "Backspace"):
      event.preventDefault();
      trigger("Delete", stack);
      return true;

    case (event.metaKey && event.key === "e"):
      event.preventDefault();
      trigger("Edit", stack);
      return true;

    case (event.metaKey && event.key === "o"):
      event.preventDefault();
      trigger("Open", stack);
      return true;
  }

  return false;
};
