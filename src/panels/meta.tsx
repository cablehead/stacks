import { JSXInternal } from "preact/src/jsx";

import { Scru128Id } from "scru128";

import { Icon } from "../ui/icons";
import { overlay } from "../ui/app.css";

import { Item, itemGetContent, Stack } from "../types";
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

function getMeta(stack: Stack, item: Item, content: string): MetaValue[] {
  const toTimestamp = (id: string) => {
    return Scru128Id.fromString(id).timestamp;
  };

  let meta: MetaValue[] = [
    { name: item.stack_id ? item.content_type : "Stack", value: item.id },
  ];

  if (!item.stack_id) {
    meta.push({
      name: "Tiktokens",
      value: (
        <span>
          {stack.nav.value.sub?.items.reduce(
            (sum, item) => sum + item.tiktokens,
            0,
          ) || 0}
        </span>
      ),
    });
  }

  if (item.stack_id && item.content_type == "Text") {
    const textMeta = getTextMeta(b64ToUtf8(content));

    const info = [
      { s: "word", n: textMeta.words },
      { s: "char", n: textMeta.chars },
      { s: "token", n: item.tiktokens },
    ]
      .filter((item) => item.n)
      .map((item) => `${item.n} ${item.s[0]}`);

    meta.push({
      name: "Info",
      value: (
        <span>
          {info.join(" . ")}
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

  if (item.touched.length === 1) {
    return [
      ...meta,
      { name: "Touched", timestamp: toTimestamp(item.id) },
    ];
  }

  return [
    ...meta,
    { name: "Times Touched", value: item.touched.length.toString() },
    {
      name: "Last Touched",
      timestamp: toTimestamp(item.last_touched),
    },
    { name: "First Touched", timestamp: toTimestamp(item.id) },
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
  const item = stack.selected();
  if (!item) return <></>;
  const content = itemGetContent(item);
  if (!content) return <></>;

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
      {getMeta(stack, item, content).map((info) => <MetaInfoRow {...info} />)}
    </div>
  );
}
