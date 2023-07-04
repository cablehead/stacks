import { JSXInternal } from "preact/src/jsx";

import { Signal } from "@preact/signals";

interface Link {
  provider: string;
  screenshot: string;
  title: string;
  description: string;
  url: string;
  icon: string;
}

export interface Item {
  hash: string;
  ids: string[];
  mime_type: string;
  content_type: string;
  terse: string;
  link?: Link;
  stack: Record<string, Item>;
}

export interface Stack {
  filter: {
    curr: Signal<string>;
    content_type: Signal<string>;
    dirty: () => boolean;
    clear: () => void;
  };
  items: Signal<Item[]>;
  selected: Signal<number>;
  normalizedSelected: Signal<number>;
  item: Signal<Item | undefined>;
  get content(): undefined | Signal<string | undefined>;
  parent?: Stack;
}

export interface Action {
  name: string;
  keys?: (string | JSXInternal.Element)[];
  trigger?: (stack: Stack) => void;
  canApply?: (stack: Stack) => boolean;
}
