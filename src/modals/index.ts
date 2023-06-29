import { Signal, signal } from "@preact/signals";

import { hide } from "tauri-plugin-spotlight-api";

import { Mode} from "./types";

import { default as mainMode } from './mainMode';
import { default as filterContentTypeMode } from './filterContentTypeMode';
import { default as addToStackMode } from './addToStackMode';
import { default as editorMode } from './editorMode';
import { default as actionsMode } from './actionsMode';

export { mainMode, filterContentTypeMode, addToStackMode, editorMode, actionsMode };

export const modes = {
  modes: [mainMode, actionsMode, editorMode] as Mode[],
  active: signal(mainMode) as Signal<Mode>,
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
    if (this.active.value == mainMode) {
      hide();
      return;
    }
    this.active.value = mainMode;
  },
  get(name: string) {
    return this.modes.find((mode) => mode.name === name) || mainMode;
  },
};
