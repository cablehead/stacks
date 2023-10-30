import { JSXInternal } from "preact/src/jsx";
import { Signal } from "@preact/signals";

import { Stack } from "../types";

export interface HotKey {
  name: string;
  keys: (string | JSXInternal.Element)[];
  onMouseDown: (event: any) => void;
  matchKeyEvent?: (event: KeyboardEvent) => boolean;
}

export interface Mode {
  name: (stack: Stack) => (string | JSXInternal.Element);
  hotKeys: (stack: Stack, modes: Modes) => HotKey[];
  activate?: (stack: Stack) => void;
}

export interface Modes {
  active: Signal<Mode>;
  isActive: (mode: Mode) => boolean;
  activate: (stack: Stack, mode: Mode) => void;
  toggle: (stack: Stack, mode: Mode) => void;
  deactivate: () => void;
  attemptAction: (event: KeyboardEvent, stack: Stack) => boolean;
}
