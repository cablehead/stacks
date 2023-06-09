import { Scru128Id } from "scru128";

import { Item } from "./types.tsx";

import { overlay } from "./app.css.ts";

interface MetaValue {
  name: string;
  value?: string;
  timestamp?: number;
}

const getMeta = (item: Item): MetaValue[] => {
  const toTimestamp = (id: string) => {
    return Scru128Id.fromString(id).timestamp;
  };

  if (item.ids.length === 0) return [];

  let meta = [
    { name: "ID", value: item.ids[0] },
  ];

  if (item.link) {
    meta.push(...[
      { name: "Content Type", value: "Link" },
      { name: "Url", value: item.link.url },
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
  } else {
    meta.push(
      { name: "Content Type", value: item.mime_type },
    );
  }

  if (item.ids.length === 1) {
    return [
      ...meta,
      { name: "Copied", timestamp: toTimestamp(item.ids[0]) },
    ];
  }

  return [
    ...meta,
    { name: "Times copied", value: item.ids.length.toString() },
    {
      name: "Last Copied",
      timestamp: toTimestamp(item.ids[item.ids.length - 1]),
    },
    { name: "First Copied", timestamp: toTimestamp(item.ids[0]) },
  ];
};

function MetaInfoRow(meta: MetaValue) {
  let displayValue: string;
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
    <div style="display:flex;">
      <div
        style={{
          flexShrink: 0,
          width: "16ch",
        }}
      >
        {meta.name}
      </div>
      <div>{displayValue}</div>
    </div>
  );
}

export function MetaPanel({ item }: { item: Item }) {
  return (
    <div
      className={overlay}
      style={{
        position: "absolute",
        width: "50ch",
        overflow: "auto",
        bottom: "0",
        fontSize: "0.9rem",
        right: "0",
        paddingTop: "0.5lh",
        paddingRight: "2ch",
        paddingLeft: "2ch",
        paddingBottom: "0.5lh",
        borderRadius: "0.5rem 0 0 0",
        zIndex: 100,
      }}
    >
      {getMeta(item).map((info) => <MetaInfoRow {...info} />)}
    </div>
  );
}
