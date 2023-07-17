import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/shell";

import { b64ToUtf8 } from "./utils";

import { editorMode, modes } from "./modals";

import { Icon } from "./ui/icons";
import { Action, Stack } from "./types";

import { triggerCopyEntireStack } from "./stacks";

export const actions: Action[] = [
  {
    name: "Copy entire stack",
    keys: [
      <Icon name="IconCommandKey" />,
      <Icon name="IconReturnKey" />,
    ],
    matchKeyEvent: (event: KeyboardEvent) =>
      event.metaKey && event.key === "Enter",
    canApply: (stack: Stack) => stack.item.value?.content_type === "Stack",
    trigger: (stack: Stack) => triggerCopyEntireStack(stack),
  },
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
      console.log("OPEN", content);
      if (content) open(b64ToUtf8(content));
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
