import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/shell";
// import { hide } from "tauri-plugin-spotlight-api";

import { b64ToUtf8 } from "./utils";

import {
  addToStackMode,
  editorMode,
  modes,
  pipeMode,
  setContentTypeAction,
} from "./modals";

import { Icon } from "./ui/icons";
import { Action, itemGetContent, Stack } from "./types";

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
    name: "Set content type",
    // keys: ["TAB"],
    // matchKeyEvent: (event: KeyboardEvent) => event.key === "Tab",
    canApply: (stack: Stack) => stack.selected()?.stack_id != null,
    trigger: (stack: Stack) => {
      modes.activate(stack, setContentTypeAction);
    },
  },

  {
    name: "Move item to stack",
    keys: ["TAB"],
    matchKeyEvent: (event: KeyboardEvent) => event.key === "Tab",
    canApply: (stack: Stack) => stack.selected()?.stack_id != null,
    trigger: (stack: Stack) => {
      modes.activate(stack, addToStackMode);
    },
  },

  {
    name: "Edit",
    keys: [<Icon name="IconCommandKey" />, "E"],
    matchKeyEvent: (event: KeyboardEvent) =>
      event.metaKey && event.key.toLowerCase() === "e",
    canApply: (stack: Stack) => {
      const item = stack.selected();
      if (!item) return false;
      return item.mime_type == "text/plain";
    },
    trigger: (stack: Stack) => modes.activate(stack, editorMode),
  },
  {
    name: "Pipe item to ...",
    keys: [<Icon name="IconCommandKey" />, "|"],
    matchKeyEvent: (event: KeyboardEvent) =>
      !event.altKey && event.metaKey && event.shiftKey &&
      event.code == "Backslash",
    trigger: (stack: Stack) => modes.activate(stack, pipeMode),
    canApply: (stack: Stack) => !!stack.selected_item(),
  },
  /*
  {
    name: "Pipe stack to GPT",
    keys: ["OPTION", <Icon name="IconCommandKey" />, "|"],
    matchKeyEvent: (event: KeyboardEvent) =>
      event.altKey && event.metaKey && event.shiftKey &&
      event.code == "Backslash",
    trigger: (stack: Stack) => {
      const item = stack.selected_stack();
      if (item) {
        invoke("store_pipe_to_gpt", { sourceId: item.id })
          .catch((err) => console.error("Error caught:", err));
      }
    },
    canApply: (stack: Stack) => !!stack.selected_item(),
  },
  */
  {
    name: "Open",
    keys: [<Icon name="IconCommandKey" />, "O"],
    matchKeyEvent: (event: KeyboardEvent) =>
      event.metaKey && event.key.toLowerCase() === "o",
    trigger: (stack: Stack) => {
      const item = stack.selected();
      if (!item?.hash) return false;
      const content = itemGetContent(item);
      if (typeof (content) == "undefined") return false;
      const url = b64ToUtf8(content);
      console.log("OPEN", url);
      open(url);
    },
    canApply: (stack: Stack) => {
      const item = stack.selected();
      if (!item?.hash) return false;
      return item.content_type == "Link";
    },
  },
  {
    name: "Delete item",
    keys: [<Icon name="IconCommandKey" />, "DEL"],
    matchKeyEvent: (event: KeyboardEvent) =>
      event.metaKey && event.key === "Backspace",
    canApply: (stack: Stack) => {
      const item = stack.selected();
      if (item) {
        return !!item.stack_id;
      }
      return false;
    },
    trigger: (stack: Stack) => {
      const item = stack.selected();
      if (item) {
        invoke("store_delete", { id: item.id });
      }
    },
  },
  {
    name: "Delete stack",
    keys: ["SHIFT", <Icon name="IconCommandKey" />, "DEL"],
    matchKeyEvent: (event: KeyboardEvent) =>
      event.metaKey && event.shiftKey && event.code == "Backspace",
    canApply: (stack: Stack) => {
      const item = stack.selected();
      if (item) {
        return !item.stack_id;
      }
      return false;
    },
    trigger: (stack: Stack) => {
      const item = stack.selected();
      if (item) {
        invoke("store_delete", { id: item.id });
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
