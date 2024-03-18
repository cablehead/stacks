import { JSXInternal } from "preact/src/jsx";

import { invoke } from "@tauri-apps/api/tauri";

import { Scru128Id } from "scru128";

import { Icon } from "../ui/icons";
import { overlay } from "../ui/app.css";

import { Content, getContent, Item, Stack } from "../types";
import { truncateUrl } from "../utils";
import { attemptActionByName } from "../actions";

interface MetaValue {
  name: string;
  value?: string | JSXInternal.Element;
  timestamp?: number;
}

function getMeta(stack: Stack, item: Item, content: Content): MetaValue[] {
  const toTimestamp = (id: string) => {
    return Scru128Id.fromString(id).timestamp;
  };

  let meta: MetaValue[] = [
    {
      name: item.stack_id ? content.content_type : "Stack",
      value: (
        <a
          onClick={(e) => {
            e.preventDefault;
            invoke("store_new_note", {
              stackId: null,
              content: item.id,
              shouldFocus: false,
            });
          }}
          style="display:flex; align-items: center; cursor: pointer;"
        >
          <span>
            {item.id}
          </span>
          <span
            style={{
              display: "inline-block",
              verticalAlign: "middle",
              width: "2ch",
              paddingBottom: "0.2em",
              whiteSpace: "nowrap",
              overflow: "hidden",
            }}
          >
            <Icon name="IconClipboard" />
          </span>
        </a>
      ),
    },
  ];

  /*
  Todo: restore aggregate tiktoken count for entire stack
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
  */

  if (item.stack_id && content.mime_type == "text/plain") {
    const info = [
      { s: "word", n: content.words },
      { s: "char", n: content.chars },
      { s: "token", n: content.tiktokens },
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

  if (content.content_type == "Link") {
    const url = content.terse;
    meta.push({
      name: "Url",
      value: (
        <a
          onClick={(e) => {
            e.preventDefault();
            attemptActionByName("Open", stack);
          }}
          style="cursor: pointer;"
        >
          <span>{truncateUrl(url, 54)}</span>
          <span
            style={{
              display: "inline-block",
              verticalAlign: "middle",
              width: "2ch",
              whiteSpace: "nowrap",
              overflow: "hidden",
              paddingBottom: "0.4em",
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
  const content = getContent(item).value;
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
