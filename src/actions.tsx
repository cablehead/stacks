import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/shell";

import { b64ToUtf8 } from "./utils";

import {
  addToStackMode,
  editorMode,
  modes,
  pipeMode,
  setContentTypeAction,
} from "./modals";

import { Icon } from "./ui/icons";
import { Action, getContent, Stack } from "./types";

export const actions: Action[] = [
  {
    name: "Set content type",
    canApply: (stack: Stack) => stack.selected()?.stack_id != null,
    trigger: (stack: Stack) => {
      modes.activate(stack, setContentTypeAction);
    },
  },

  {
    name: "Copy item to stack",
    keys: ["TAB"],
    matchKeyEvent: (event: KeyboardEvent) => event.key === "Tab",
    canApply: (stack: Stack) => stack.selected()?.stack_id != null,
    trigger: (stack: Stack) => {
      modes.activate(stack, addToStackMode);
    },
  },

  {
    name: "Edit",
    keys: [<Icon name="IconCommandKey" />, <Icon name="IconReturnKey" />],
    matchKeyEvent: (event: KeyboardEvent) =>
      event.metaKey && event.key === "Enter",
    canApply: (stack: Stack) => {
      const item = stack.selected();
      if (!item) return false;
      return getContent(item).value?.mime_type == "text/plain";
    },
    trigger: (stack: Stack) => modes.activate(stack, editorMode),
  },
  {
    name: "Pipe item to ...",
    keys: [<Icon name="IconCommandKey" />, "P"],
    matchKeyEvent: (event: KeyboardEvent) =>
      !event.ctrlKey && !event.altKey && event.metaKey &&
      event.key.toLowerCase() === "p",
    trigger: (stack: Stack) => modes.activate(stack, pipeMode),
    canApply: (stack: Stack) => !!stack.selected_item(),
  },
  {
    name: "Open",
    keys: [<Icon name="IconCommandKey" />, "O"],
    matchKeyEvent: (event: KeyboardEvent) =>
      event.metaKey && event.key.toLowerCase() === "o",
    trigger: (stack: Stack) => {
      const item = stack.selected();
      if (!item?.hash) return false;

      (async () => {
        const content = await invoke<string>("store_get_raw_content", {
          hash: item.hash,
        });
        const url = b64ToUtf8(content);
        console.log("OPEN", url);
        open(url);
      })();

      return true;
    },
    canApply: (stack: Stack) => {
      const item = stack.selected();
      if (!item?.hash) return false;
      return getContent(item).value?.content_type == "Link";
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
