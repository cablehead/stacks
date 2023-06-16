import { signal } from "@preact/signals";

import { writeText } from "@tauri-apps/api/clipboard";

import { invoke } from "@tauri-apps/api/tauri";

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
}

// TODO: cap size of CAS, with MRU eviction
const CAS: Map<string, string> = new Map();

export async function getContent(hash: string): Promise<string> {
  const cachedItem = CAS.get(hash);
  if (cachedItem !== undefined) {
    return cachedItem;
  }

  console.log("CACHE MISS", hash);
  const content: string = await invoke("store_get_content", { hash: hash });
  CAS.set(hash, content);
  return content;
}

export const editor = {
  show: signal(false),
  content: "",
  get save() {
    return () => writeText(this.content);
  },
};
