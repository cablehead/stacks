import { JSXInternal } from "preact/src/jsx";

import { Scru128Id } from "scru128";

import { Icon } from "../ui/icons";
import { overlay } from "../ui/app.css";

import { Item, Stack } from "../types";
import { b64ToUtf8, truncateUrl } from "../utils";

function getTextMeta(input: string): { words: number; chars: number } {
  const words = input.trim().split(/\s+/).length;
  const chars = [...input].length;
  return { words, chars };
}

interface MetaValue {
  name: string;
  value?: string | JSXInternal.Element;
  timestamp?: number;
}

function getMeta(item: Item, content: string): MetaValue[] {
  const toTimestamp = (id: string) => {
    return Scru128Id.fromString(id).timestamp;
  };

  if (item.ids.length === 0) return [];

  let meta: MetaValue[] = [
    { name: "ID", value: item.ids[item.ids.length - 1] },
    { name: "Content Type", value: item.content_type },
  ];

  if (item.content_type == "Text") {
    const textMeta = getTextMeta(b64ToUtf8(content));
    meta.push({
      name: "Info",
      value: (
        <span>
          {`${textMeta.words} word${textMeta.words !== 1 ? "s" : ""}, ${textMeta.chars} char${textMeta.chars !== 1 ? "s" : ""}`}
        </span>
      ),
    });
  }

  if (item.content_type == "Link") {
    const url = b64ToUtf8(content);
    meta.push({
      name: "Url",
      value: (
        <a href={url} target="_blank">
          <span>{truncateUrl(url, 54)}</span>
          <span
            style={{
              display: "inline-block",
              verticalAlign: "middle",
              width: "2ch",
              whiteSpace: "nowrap",
              overflow: "hidden",
            }}
          >
            <Icon name="IconExternalLink" />
          </span>
        </a>
      ),
    });
  }

  if (item.link) {
    meta.push(...[
      { name: "Title", value: item.link.title },
      {
        name: "Description",
        value: (
          <div
            style={{
              maxHeight: "3.2lh",
              overflow: "auto",
              textOverflow: "ellipsis",
            }}
          >
            {item.link.description}
          </div>
        ),
      },
    ]);
  }

  if (item.ids.length === 1) {
    return [
      ...meta,
      { name: "Touched", timestamp: toTimestamp(item.ids[0]) },
    ];
  }

  return [
    ...meta,
    { name: "Times Touched", value: item.ids.length.toString() },
    {
      name: "Last Touched",
      timestamp: toTimestamp(item.ids[item.ids.length - 1]),
    },
    { name: "First Touched", timestamp: toTimestamp(item.ids[0]) },
  ];
}

function MetaInfoRow(meta: MetaValue) {
  let displayValue;
  if (meta.timestamp !== undefined) {
    displayValue = new Date(meta.timestamp).toLocaleString("en-US", {
      weekday: "short",
      year: "numeric",
      month: "short",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
      hour12: true,
    });
  } else {
    displayValue = meta.value || "";
  }

  return (
    <div style="display:flex; width: 100%">
      <div
        style={{
          flexShrink: 0,
          width: "16ch",
        }}
      >
        {meta.name}
      </div>
      <div style={{ overflowWrap: "anywhere", wordBreak: "break-all" }}>
        {displayValue}
      </div>
    </div>
  );
}

export function MetaPanel({ stack }: { stack: Stack }) {
  const item = stack.item.value;
  const content = stack.content?.value;
  if (!item || !content) return <></>;

  return (
    <div
      className={overlay}
      style={{
        position: "absolute",
        width: "47ch",
        overflowX: "hidden",
        bottom: "0",
        fontSize: "0.9rem",
        right: "0",
        paddingTop: "0.25lh",
        paddingLeft: "1ch",
        paddingBottom: "0.25lh",
        borderRadius: "0.5em 0 0 0",
        zIndex: 10,
      }}
    >
      {getMeta(item, content).map((info) => <MetaInfoRow {...info} />)}
    </div>
  );
}
