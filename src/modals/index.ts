import { Signal, signal } from "@preact/signals";

import { hide } from "tauri-plugin-spotlight-api";

import { Mode } from "./types";

import { default as actionsMode } from "./actionsMode";
import { default as addToStackMode } from "./addToStackMode";
import { default as editorMode } from "./editorMode";
import { default as filterContentTypeMode } from "./filterContentTypeMode";
import { default as mainMode } from "./mainMode";
import { default as newMode } from "./newMode";
import { default as newNoteMode } from "./newNoteMode";
import { default as pipeMode } from "./pipeMode";
import { default as pipeToCommand } from "./pipeToCommand";
import { default as setContentTypeAction } from "./setContentTypeAction";
import { default as settingsMode } from "./settingsMode";

import { Stack } from "../types";

export {
  actionsMode,
  addToStackMode,
  editorMode,
  filterContentTypeMode,
  mainMode,
  newMode,
  newNoteMode,
  pipeMode,
  pipeToCommand,
  setContentTypeAction,
  settingsMode,
};

export const modes = {
  active: signal(mainMode) as Signal<Mode>,

  isActive(mode: Mode) {
    return mode == this.active.value;
  },

  activate(stack: Stack, mode: Mode) {
    mode.activate && mode.activate(stack);
    this.active.value = mode;
  },

  toggle(stack: Stack, mode: Mode) {
    if (mode == this.active.value) {
      this.deactivate();
      return;
    }
    this.activate(stack, mode);
  },

  deactivate() {
    if (this.active.value == mainMode) {
      hide();
      return;
    }
    this.active.value = mainMode;
  },

  attemptAction(event: KeyboardEvent, stack: Stack): boolean {
    switch (true) {
      case event.metaKey && event.key === "k":
        event.preventDefault();
        modes.toggle(stack, actionsMode);
        return true;

      case (event.metaKey && event.key === "n"):
        event.preventDefault();
        modes.toggle(stack, newMode);
        return true;

      case (event.metaKey && event.key === "u"):
        event.preventDefault();
        modes.toggle(stack, filterContentTypeMode);
        return true;
    }

    const mode = this.active.value;
    for (const hotKey of mode.hotKeys(stack, this)) {
      if (
        hotKey.matchKeyEvent &&
        hotKey.matchKeyEvent(event)
      ) {
        event.preventDefault();
        hotKey.onMouseDown(event);
        return true;
      }
    }
    return false;
  },
};
