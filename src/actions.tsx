import { invoke } from "@tauri-apps/api/tauri";
// import { open } from "@tauri-apps/api/shell";
// import { hide } from "tauri-plugin-spotlight-api";

// import { b64ToUtf8 } from "./utils";

import { editorMode, modes, pipeMode } from "./modals";

import { Icon } from "./ui/icons";
import { Action, Stack } from "./types";

export const actions: Action[] = [
  {
    name: "Copy entire stack",
    keys: [
      <Icon name="IconCommandKey" />,
      <Icon name="IconReturnKey" />,
    ],
    matchKeyEvent: (event: KeyboardEvent) =>
      event.metaKey && event.key === "Enter",
    canApply: (_: Stack) => false,
    // stack.item.value?.content_type === "Stack" || !!stack.parent?.item.value,
    trigger: (_: Stack) => {
      /*
      let item = stack.item.value?.content_type === "Stack"
        ? stack.item.value
        : stack.parent?.item.value;
      if (item) {
        invoke("store_copy_entire_stack_to_clipboard", {
          stackHash: item.hash,
        });
        hide();
      }
        */
    },
  },
  {
    name: "Edit",
    keys: [<Icon name="IconCommandKey" />, "E"],
    matchKeyEvent: (event: KeyboardEvent) =>
      event.metaKey && event.key.toLowerCase() === "e",
    trigger: (stack: Stack) => modes.activate(stack, editorMode),
    canApply: (_: Stack) => false,
  },
  {
    name: "Pipe to command",
    keys: [<Icon name="IconCommandKey" />, "|"],
    matchKeyEvent: (event: KeyboardEvent) =>
      event.metaKey && event.shiftKey && event.code == "Backslash",
    trigger: (stack: Stack) => modes.activate(stack, pipeMode),
    canApply: (_: Stack) => false,
  },
  {
    name: "Open",
    keys: [<Icon name="IconCommandKey" />, "O"],
    matchKeyEvent: (event: KeyboardEvent) =>
      event.metaKey && event.key.toLowerCase() === "o",
    trigger: (_: Stack) => {
      // const content = stack.content?.value;
      // console.log("OPEN", content);
      // if (content) open(b64ToUtf8(content));
    },
    canApply: (_: Stack) => false,
  },
  {
    name: "Delete",
    keys: ["Ctrl", "DEL"],
    matchKeyEvent: (event: KeyboardEvent) =>
      event.ctrlKey && event.key === "Backspace",
    canApply: (stack: Stack) => !!stack.item.value,
    trigger: (stack: Stack) => {
      const item = stack.item.value;
      console.log("DELETE", item);
      if (item) {
        invoke("store_delete", { id: item.id });
      }
    },
  },
  {
    name: "Remove from stack",
    keys: ["Ctrl", "DEL"],
    matchKeyEvent: (event: KeyboardEvent) =>
      event.ctrlKey && event.key === "Backspace",
    canApply: (_: Stack) => false,
    trigger: (stack: Stack) => {
      const item = stack.item.value;
      if (item) {
        /*
        invoke("store_delete", {
          hash: item.hash,
          stackHash: stack.parent?.item.value?.hash,
        });
        */
      }
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
