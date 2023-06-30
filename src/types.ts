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
  stack: Item[];
}

export interface Stack {
  items: Signal<Item[]>;
  selected: Signal<number>;
  normalizedSelected: Signal<number>;
  item: Signal<Item | undefined>;
  get content(): undefined | Signal<string | undefined>;
}

export interface Action {
  name: string;
  keys?: (string | JSXInternal.Element)[];
  trigger?: (stack: Stack) => void;
  canApply?: (stack: Stack) => boolean;
}
