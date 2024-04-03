import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/shell";

import { dn, b64ToUtf8, matchKeyEvent } from "./utils";

import {
  addToStackMode,
  editorMode,
  modes,
  pipeStackToShell,
  pipeToCommand,
  renameStackMode,
  setContentTypeAction,
} from "./modals";

import { Icon } from "./ui/icons";
import { Action, getContent, Stack } from "./types";

export const actions: Action[] = [
  {
    name: "Open link in browser",
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
    name: "Set clip content type",
    canApply: (stack: Stack) => stack.selected()?.stack_id != null,
    keys: [<Icon name="IconCommandKey" />, <Icon name="IconShiftKey" />, "U"],
    matchKeyEvent: (event: KeyboardEvent) =>
      matchKeyEvent(event, { meta: true, shift: true, key: "u" }),
    trigger: (stack: Stack) => {
      modes.activate(stack, setContentTypeAction);
    },
  },

  {
    name: "Focus clip",
    keys: ["TAB"],
    matchKeyEvent: (event: KeyboardEvent) =>
      matchKeyEvent(event, { code: "Tab" }),
    canApply: (stack: Stack) => !!stack.selected_item(),
    trigger: (stack: Stack) => {
      const item = stack.selected_item();
      if (item) {
        (async () => {
          await invoke("store_add_to_new_stack", {
            name: dn(),
            sourceId: item.id,
            focus: true,
          });
        })();
      }
    },
  },

  {
    name: "Stash clip",
    keys: [<Icon name="IconCommandKey" />, "S"],
    matchKeyEvent: (event: KeyboardEvent) =>
      matchKeyEvent(event, { meta: true, code: "KeyS" }),
    canApply: (stack: Stack) => stack.selected()?.stack_id != null,
    trigger: (stack: Stack) => {
      modes.activate(stack, addToStackMode);
    },
  },

  {
    name: "Edit clip",
    keys: [<Icon name="IconCommandKey" />, <Icon name="IconReturnKey" />],
    matchKeyEvent: (event: KeyboardEvent) =>
      matchKeyEvent(event, { meta: true, code: "Enter" }),
    canApply: (stack: Stack) => {
      const item = stack.selected_item();
      if (!item) return false;
      return getContent(item).value?.mime_type == "text/plain";
    },
    trigger: (stack: Stack) => modes.activate(stack, editorMode),
  },

  {
    name: "Pipe clip",
    keys: [<Icon name="IconCommandKey" />, "P"],
    matchKeyEvent: (event: KeyboardEvent) =>
      matchKeyEvent(event, { meta: true, code: "KeyP" }),
    trigger: (stack: Stack) => modes.activate(stack, pipeToCommand),
    canApply: (stack: Stack) => !!stack.selected_item(),
  },

  {
    name: "Delete clip",
    keys: [<Icon name="IconCommandKey" />, "DEL"],
    matchKeyEvent: (event: KeyboardEvent) =>
      matchKeyEvent(event, { meta: true, code: "Backspace" }),
    canApply: (stack: Stack) => !!stack.selected_item(),
    trigger: (stack: Stack) => {
      const item = stack.selected_item();
      if (item) {
        invoke("store_delete", { id: item.id });
      }
    },
  },

  {
    name: "Rename stack",
    keys: [
      <Icon name="IconAltKey" />,
      <Icon name="IconCommandKey" />,
      <Icon name="IconReturnKey" />,
    ],
    matchKeyEvent: (event: KeyboardEvent) =>
      matchKeyEvent(event, { meta: true, alt: true, code: "Enter" }),
    canApply: (stack: Stack) => {
      const item = stack.selected_stack();
      if (!item) return false;
      return getContent(item).value?.mime_type == "text/plain";
    },
    trigger: (stack: Stack) => modes.activate(stack, renameStackMode),
  },

  {
    name: "Pipe stack",
    keys: [<Icon name="IconAltKey" />, <Icon name="IconCommandKey" />, "P"],
    matchKeyEvent: (event: KeyboardEvent) =>
      matchKeyEvent(event, { meta: true, alt: true, code: "KeyP" }),
    trigger: (stack: Stack) => modes.activate(stack, pipeStackToShell),
    canApply: (stack: Stack) => !!stack.selected_item(),
  },

  {
    name: "Delete stack",
    keys: [<Icon name="IconAltKey" />, <Icon name="IconCommandKey" />, "DEL"],
    matchKeyEvent: (event: KeyboardEvent) =>
      matchKeyEvent(event, { meta: true, alt: true, code: "Backspace" }),
    canApply: (stack: Stack) => {
      return !!stack.selected_stack();
    },
    trigger: (stack: Stack) => {
      const item = stack.selected_stack();
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

export const attemptActionByName = (name: string, stack: Stack): boolean => {
  for (const action of actions) {
    if (!action.trigger) continue;
    if (action.name != name) continue;
    if (action.canApply && !action.canApply(stack)) continue;
    action.trigger(stack);
    return true;
  }
  return false;
};
