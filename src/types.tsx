
interface Link {
  provider: string;
  screenshot: string;
  title: string;
  description: string;
}

export interface Item {
  hash: string;
  ids: string[];
  mime_type: string;
  terse: string;
  link?: Link;
}
