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
  loaded: Signal<LoadedItem | undefined>;
}

export interface LoadedItem {
  item: Item;
  content: string;
}

export interface Action {
  name: string;
  keys?: (string | JSXInternal.Element)[];
  trigger?: (loaded: LoadedItem) => void;
  canApply?: (item: Item) => boolean;
}
