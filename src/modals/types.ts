import { JSXInternal } from "preact/src/jsx";
import { Signal } from "@preact/signals";

import { Stack } from "../types";

interface HotKey {
  name: string;
  keys: (string | JSXInternal.Element)[];
  onMouseDown: (event: any) => void;
}

export interface Mode {
  name: string;
  hotKeys: (stack: Stack, modes: Modes) => HotKey[];
  activate?: () => void;
}

export interface Modes {
  modes: Mode[];
  active: Signal<Mode>;
  isActive: (mode: Mode) => boolean;
  activate: (mode: Mode) => void;
  toggle: (mode: Mode) => void;
  deactivate: () => void;
  get: (name: string) => Mode;
}
