import { Scru128Id } from "scru128";
import { Signal } from "@preact/signals";

const CryptoJS = require("crypto-js");

function b64ToUtf8(str: string) {
    return decodeURIComponent(escape(window.atob(str)));
}

export type Item = {
  id: string;
  topic: string;
  data: string;
  o?: any;
  icon: string;
  terse: string;
  preview: any;
  key: string;
  created_at: string;
  count: number;
  updated_at?: string;
  meta: Array<{ name: string; value: any }>;
};

function scru128ToDate(id: string) {
  const scruId = Scru128Id.fromString(id);
  const timestampMillis = scruId.timestamp;
  const date = new Date(timestampMillis);
  return date;
}

function parseItem(raw: string): Item | false {
  let item = JSON.parse(raw);
  item.meta = new Array();

  switch (item.topic) {
    case "clipboard":
      let data = JSON.parse(item.data);
      if ("public.utf8-plain-text" in data.types) {
        if (data.types["public.utf8-plain-text"] == "") {
          return false;
        }
        item.icon = "IconClipboard";
        item.terse = b64ToUtf8(data.types["public.utf8-plain-text"]);
        item.preview = item.terse;
        item.key = CryptoJS.MD5(item.preview).toString();
        break;
      }

      if ("public.png" in data.types) {
        item.icon = "IconImage";
        item.terse = data.source;
        item.preview = (
          <img src={"data:image/png;base64," + data["types"]["public.png"]} />
        );
        item.key = CryptoJS.MD5(data["types"]["public.png"]).toString();
        break;
      }

      item.icon = "IconBell";
      item.terse = item.data;
      item.preview = item.data;
      item.key = CryptoJS.MD5(item.preview).toString();
      break;

    default:
      item.icon = "IconBell";
      item.terse = item.data;
      item.preview = item.data;
      item.key = CryptoJS.MD5(item.preview).toString();
  }

  return item;
}

export function addItem(
  raw: string,
  items: Signal<Map<string, Item>>,
  availableItems: Signal<Array<Item>>,
  selected: Signal<number>,
  focusSelected: (delay: number) => void,
  updateSelected: (n: number) => void,
) {
  const item = parseItem(raw);
  if (!item) return;

  item.created_at = scru128ToDate(item.id)
    .toLocaleString(
      "en-US",
      {
        weekday: "short",
        year: "numeric",
        month: "short",
        day: "numeric",
        hour: "numeric",
        minute: "numeric",
        hour12: true,
      },
    );

  const base = new Map(items.value);

  const prev = base.get(item.key);

  if (prev !== undefined) {
    item.count = prev.count + 1;
    item.updated_at = item.created_at;
    item.created_at = prev.created_at;
    item.meta = [
      { name: "ID", value: item.id },
      { name: "Topic", value: item.topic },
      { name: "Times copied", value: item.count },
      { name: "Last Copied", value: item.updated_at },
      { name: "First Copied", value: item.created_at },
    ];

    // If focused item matches the item being added, set focus to the start
    if (availableItems.value[selected.value].key == item.key) {
      selected.value = 0;
      focusSelected(5);
    }
  } else {
    item.count = 1;
    item.meta = [
      { name: "ID", value: item.id },
      { name: "Topic", value: item.topic },
      { name: "Copied", value: item.created_at },
    ];
    // Move cursor down for new item, to preserve focus
    if (selected.value > 0) updateSelected(1);
  }
  focusSelected(5);
  base.set(item.key, item);
  items.value = base;
}
