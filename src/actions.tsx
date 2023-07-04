import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/shell";

import { editorMode, modes } from "./modals";

import { Icon } from "./ui/icons";
import { Action, Stack } from "./types";

export const actions: Action[] = [
  {
    name: "Edit",
    keys: [<Icon name="IconCommandKey" />, "E"],
    matchKeyEvent: (event: KeyboardEvent) =>
      event.metaKey && event.key.toLowerCase() === "e",
    trigger: (stack: Stack) => modes.activate(stack, editorMode),
    canApply: (stack: Stack) => stack.item.value?.mime_type === "text/plain",
  },
  {
    name: "Open",
    keys: [<Icon name="IconCommandKey" />, "O"],
    matchKeyEvent: (event: KeyboardEvent) =>
      event.metaKey && event.key.toLowerCase() === "o",
    trigger: (stack: Stack) => {
      const content = stack.content?.value;
      if (content) open(content);
    },
    canApply: (stack: Stack) => stack.item.value?.content_type === "Link",
  },
  {
    name: "Delete",
    keys: ["Ctrl", "DEL"],
    matchKeyEvent: (event: KeyboardEvent) =>
      event.ctrlKey && event.key === "Backspace",
    canApply: (stack: Stack) => !stack.parent,
    trigger: (stack: Stack) => {
      const item = stack.item.value;
      if (item) invoke("store_delete", { hash: item.hash });
    },
  },
  {
    name: "Remove from stack",
    keys: ["Ctrl", "DEL"],
    matchKeyEvent: (event: KeyboardEvent) =>
      event.ctrlKey && event.key === "Backspace",
    canApply: (stack: Stack) => !!stack.parent,
    trigger: (stack: Stack) => {
      const name = stack.parent?.item.value?.terse;
      if (!name) return;
      const item = stack.item.value;
      if (!item) return;
      const id = item.ids[item.ids.length - 1];
      if (!id) return;
      invoke("store_delete_from_stack", { name: name, id: id });
    },
  },
];

export const attemptAction = (event: KeyboardEvent, stack: Stack): boolean => {
  for (const action of actions) {
    if (action.canApply && !action.canApply(stack)) continue;
    if (
      action.trigger && action.matchKeyEvent &&
      action.matchKeyEvent(event)
    ) {
      event.preventDefault();
      action.trigger(stack);
      return true;
    }
  }
  return false;
};
