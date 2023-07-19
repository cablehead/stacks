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

enum FocusType {
  INDEX,
  FIRST,
}

export class Focus {
  type: FocusType;
  n: number;

  constructor(type: FocusType, n: number = 0) {
    this.type = type;
    this.n = n;
  }

  static first(): Focus {
    return new Focus(FocusType.FIRST);
  }

  static index(n: number): Focus {
    return new Focus(FocusType.INDEX, n);
  }

  isFocusFirst(): boolean {
    return this.type === FocusType.FIRST;
  }

  down() {
    if (this.type === FocusType.FIRST) {
      return Focus.index(1);
    } else if (this.type === FocusType.INDEX) {
      return Focus.index(this.n + 1);
    }
    return this;
  }

  up() {
    if (this.type === FocusType.FIRST) {
      return Focus.index(-1);
    } else if (this.type === FocusType.INDEX) {
      return Focus.index(this.n - 1);
    }
    return this;
  }

  currIndex() {
    if (this.type === FocusType.FIRST) {
      return 0;
    }
    return this.n;
  }
}

export interface Stack {
  filter: {
    curr: Signal<string>;
    content_type: Signal<string>;
    dirty: () => boolean;
    clear: () => void;
  };
  items: Signal<Item[]>;
  selected: Signal<Focus>;
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
  matchKeyEvent?: (event: KeyboardEvent) => boolean;
}
