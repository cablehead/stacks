import { Signal, signal } from "@preact/signals";

import { hide } from "tauri-plugin-spotlight-api";

import { Icon } from "./ui/icons";

import { state } from "./panels/filter";

import { Mode } from "./modals/types";

const defaultMode = {
  name: "Clipboard",
  hotKeys: () => [
    {
      name: "Copy",
      keys: [<Icon name="IconReturnKey" />],
      onMouseDown: () => {
      },
    },

    {
      name: "Actions",
      keys: [<Icon name="IconCommandKey" />, "K"],
      onMouseDown: () => {
        modes.toggle(actionsMode);
      },
    },

    !state.dirty()
      ? {
        name: "Close",
        keys: ["ESC"],
        onMouseDown: () => {
        },
      }
      : {
        name: "Clear filter",
        keys: ["ESC"],
        onMouseDown: () => {
        },
      },
  ],
};

export const actionsMode = {
  name: "Actions",
  hotKeys: () => [
    {
      name: "Trigger",
      keys: [<Icon name="IconReturnKey" />],
      onMouseDown: () => {
      },
    },
    {
      name: "Back",
      keys: ["ESC"],
      onMouseDown: () => modes.deactivate(),
    },
  ],
};

export const editorMode = {
  name: "Editor",
  hotKeys: () => [
    {
      name: "Capture",
      keys: [
        <Icon name="IconCommandKey" />,
        <Icon name="IconReturnKey" />,
      ],
      onMouseDown: () => {
        // onMouseDown={editor.save}
      },
    },
    {
      name: "Discard",
      keys: ["ESC"],
      onMouseDown: () => modes.deactivate(),
    },
  ],
};

export const modes = {
  modes: [defaultMode, actionsMode, editorMode] as Mode[],
  active: signal(defaultMode) as Signal<Mode>,
  isActive(mode: Mode) {
    return mode == this.active.value;
  },
  activate(mode: Mode) {
    mode.activate && mode.activate();
    this.active.value = mode;
  },
  toggle(mode: Mode) {
    if (mode == this.active.value) {
      this.deactivate();
      return;
    }
    this.activate(mode);
  },
  deactivate() {
    if (this.active.value == defaultMode) {
      hide();
      return;
    }
    this.active.value = defaultMode;
  },
  get(name: string) {
    return this.modes.find((mode) => mode.name === name) || defaultMode;
  },
};
