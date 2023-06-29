import { JSXInternal } from "preact/src/jsx";

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
