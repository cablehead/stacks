
export interface Item {
  id: string;
  last_touched: string;
  touched: string[];
  hash: string;
  stack_id: string | null;
  children: string[];
}

export interface ContentMeta {
  hash: string | null;
  mime_type: string;
  content_type: string;
  terse: string;
  tiktokens: number;
}

export interface State {
  root: string[];
  items: { [id: string]: Item };
  content_meta: { [key: string]: ContentMeta };
  matches: string[];
}

/*
enum FocusType {
  INDEX,
  FIRST,
}

export class Focus {
  type: FocusType;
  item: string;

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
*/

