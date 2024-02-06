import { Signal, signal } from "@preact/signals";

import { hide } from "tauri-plugin-spotlight-api";
import { invoke } from "@tauri-apps/api/tauri";

import { Mode } from "./types";

import { default as actionsMode } from "./actionsMode";
import { default as addToStackMode } from "./addToStackMode";
import { default as editorMode } from "./editorMode";
import { default as filterContentTypeMode } from "./filterContentTypeMode";
import { default as mainMode } from "./mainMode";
import { default as newMode } from "./newMode";
import { default as newNoteMode } from "./newNoteMode";
import { default as pipeToCommand } from "./pipeToCommand";
import { default as pipeStackToShell } from "./pipeStackToShell";
import { default as setContentTypeAction } from "./setContentTypeAction";
import { default as settingsMode } from "./settingsMode";

import { Stack } from "../types";
import { dn, matchKeyEvent } from "../utils";

export {
  actionsMode,
  addToStackMode,
  editorMode,
  filterContentTypeMode,
  mainMode,
  newMode,
  newNoteMode,
  pipeStackToShell,
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

  showActiveOverlay(stack: Stack) {
    if ("Modal" in this.active.value) {
      const Modal = this.active.value.Modal as preact.ComponentType<any>;
      return <Modal stack={stack} modes={this} />;
    }
  },

  attemptAction(event: KeyboardEvent, stack: Stack): boolean {
    switch (true) {
      case event.metaKey && event.key === "k":
        event.preventDefault();
        modes.toggle(stack, actionsMode);
        return true;

      // https://github.com/cablehead/stacks/issues/40
      case (matchKeyEvent(event, {
        meta: true,
        alt: true,
        shift: true,
        code: "KeyN",
      })):
        event.preventDefault();
        (async () => {
          await invoke("store_new_stack", {
            name: dn(),
          });
          modes.activate(stack, newNoteMode);
        })();
        return true;

      case event.metaKey && event.key === ",":
        event.preventDefault();
        modes.toggle(stack, settingsMode);
        return;

      case (matchKeyEvent(event, { meta: true, shift: true, code: "KeyN" })):
        event.preventDefault();
        modes.toggle(stack, newNoteMode);
        return true;

      case (matchKeyEvent(event, { meta: true, alt: true, code: "KeyN" })):
        event.preventDefault();
        (async () => {
          await invoke("store_new_stack", {
            name: dn(),
          });
        })();
        return true;

      case (matchKeyEvent(event, { meta: true, key: "n" })):
        event.preventDefault();
        modes.toggle(stack, newMode);
        return true;

      case matchKeyEvent(event, { meta: true, key: "u" }):
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
