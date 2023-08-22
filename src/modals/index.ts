import { Signal, signal } from "@preact/signals";

import { hide } from "tauri-plugin-spotlight-api";

import { Mode } from "./types";

import { default as mainMode } from "./mainMode";
import { default as filterContentTypeMode } from "./filterContentTypeMode";
import { default as addToStackMode } from "./addToStackMode";
import { default as editorMode } from "./editorMode";
import { default as newNoteMode } from "./newNoteMode";
import { default as actionsMode } from "./actionsMode";
import { default as pipeMode } from "./pipeMode";
import { default as settingsMode } from "./settingsMode";

import { Stack } from "../types";

export {
  actionsMode,
  pipeMode,
  addToStackMode,
  settingsMode,
  editorMode,
  newNoteMode,
  filterContentTypeMode,
  mainMode,
};

export const modes = {
  active: signal(settingsMode) as Signal<Mode>,
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
};
