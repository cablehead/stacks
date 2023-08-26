import { useEffect, useRef } from "preact/hooks";

import { b64ToUtf8 } from "../utils";

import { Icon } from "../ui/icons";
import { borderRight } from "../ui/app.css";

import { ItemMeta, Layer, Scru128Id, Stack } from "../types";

const TerseRow = (
  { stack, item, isSelected, isFocused }: {
    stack: Stack;
    item: ItemMeta;
    isSelected: boolean;
    isFocused: boolean;
  },
) => {
  const theRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (isSelected && theRef.current) {
      theRef.current.scrollIntoView({
        behavior: "smooth",
        block: "nearest",
      });
    }
  }, [isSelected, theRef.current]);

  return (
    <div
      ref={theRef}
      className={"terserow" +
        (isSelected ? (isFocused ? " highlight" : " selected") : "")}
      onMouseDown={() => {
        stack.select(item.o.id);
      }}
      style="
          display: flex;
          width: 100%;
          gap: 0.5ch;
          overflow: hidden;
          padding: 0.5ch 0.75ch;
          border-radius: 6px;
          cursor: pointer;
          "
    >
      {false &&
        (
          <div
            style={{
              flexShrink: 0,
              width: "2ch",
              whiteSpace: "nowrap",
              overflow: "hidden",
            }}
          >
            <RowIcon item={item} />
          </div>
        )}

      <div
        style={{
          flexGrow: 1,
          whiteSpace: "nowrap",
          overflow: "hidden",
          textOverflow: "ellipsis",
        }}
      >
        {item.meta.terse}
      </div>
    </div>
  );
};

const renderItems = (
  stack: Stack,
  key: string,
  layer: Layer,
  maxWidth: string,
  focusedId: Scru128Id,
) => {
  const { items, selected } = layer;

  if (items.length == 0) return <i>no items</i>;
  return (
    <div
      key={key}
      className={borderRight}
      style={`
      flex: 1;
      max-width: ${maxWidth};
      overflow-y: auto;
      padding-right: 0.5rem;
    `}
    >
      {items
        .map((item) => (
          <TerseRow
            stack={stack}
            item={item}
            key={item.o.id}
            isSelected={item.o.id == selected.o.id}
            isFocused={item.o.id == focusedId}
          />
        ))}
    </div>
  );
};

export function Nav({ stack }: { stack: Stack }) {
  const neo = stack.neo.value;
  return (
    <div style="flex: 3; display: flex; height: 100%; overflow: hidden; gap: 0.5ch;">
      {renderItems(stack, "root", neo.root, "20ch", neo.focusedId)}

      {neo.sub
        ? (
          <>
            {renderItems(
              stack,
              neo.root.selected.o.id,
              neo.sub,
              "20ch",
              neo.focusedId,
            )}
            <div style="flex: 3; overflow: auto; height: 100%">
              <Preview stack={stack} item={neo.sub.selected} />
            </div>
          </>
        )
        : <i>no items</i>}
    </div>
  );
}

const RowIcon = ({ item }: { item: ItemMeta }) => {
  if (!item.o.stack_id) return <Icon name="IconStack" />;

  switch (item.meta.content_type) {
    case "Image":
      return <Icon name="IconImage" />;

    case "Link":
      return <Icon name="IconLink" />;

    case "Text":
      return <Icon name="IconClipboard" />;
  }

  return <Icon name="IconBell" />;
};

function Preview({ stack, item }: { stack: Stack; item: ItemMeta }) {
  const content = stack.getContent(item.o.hash).value;
  if (!content) return <div>loading...</div>;

  if (item.meta.mime_type === "image/png") {
    return (
      <img
        src={"data:image/png;base64," + content}
        style={{
          opacity: 0.95,
          borderRadius: "0.5rem",
          maxHeight: "100%",
          height: "auto",
          width: "auto",
          objectFit: "contain",
        }}
      />
    );
  }

  return (
    <pre style="margin: 0; white-space: pre-wrap; overflow-x: hidden">
    { b64ToUtf8(content) }
    </pre>
  );
}
