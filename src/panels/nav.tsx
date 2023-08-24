import { useEffect, useRef } from "preact/hooks";

import { b64ToUtf8 } from "../utils";

import { Icon } from "../ui/icons";
import { borderRight } from "../ui/app.css";

import { Item, Stack } from "../types";

const TerseRow = (
  { stack, item, selectedId }: {
    stack: Stack;
    item: Item;
    selectedId?: string;
  },
) => {
  const theRef = useRef<HTMLDivElement>(null);
  const isSelected = stack.selected.value.curr(stack) === item.id;

  useEffect(() => {
    if (isSelected && theRef.current) {
      theRef.current.scrollIntoView({
        behavior: "smooth",
        block: "nearest",
      });
    }
  }, [isSelected, theRef.current]);

  const meta = stack.getContentMeta(item);

  return (
    <div
      ref={theRef}
      className={"terserow" +
        (stack.selected.value.curr(stack) === item.id ? " highlight" : "") +
        (item.id === selectedId ? " selected" : "")}
      onMouseDown={() => {
        stack.select(item.id);
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
      {item.stack_id &&
        (
          <div
            style={{
              flexShrink: 0,
              width: "2ch",
              whiteSpace: "nowrap",
              overflow: "hidden",
            }}
          >
            <RowIcon stack={stack} item={item} />
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
        {meta.terse}
      </div>
    </div>
  );
};

const renderItems = (
  stack: Stack,
  key: string,
  items: string[],
  maxWidth: string,
  selectedId?: string,
) => {
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
      {items.map((id) => stack.state.value.items[id])
        .map((item) => (
          <TerseRow
            stack={stack}
            item={item}
            key={item.id}
            selectedId={selectedId}
          />
        ))}
    </div>
  );
};

export function Nav({ stack }: { stack: Stack }) {
  const selectedId = stack.selected.value.curr(stack);
  const selectedItem = stack.state.value.items[selectedId];

  if (!selectedItem) return <i>no matches</i>;

  const parentItem = selectedItem.stack_id &&
    stack.state.value.items[selectedItem.stack_id];

  if (!parentItem) {
    const selectedItemChildren = stack.getChildren(selectedItem);
    const selectedChildId = stack.lastSelected.get(selectedId) ||
      selectedItemChildren[0];
    const selectedChild = stack.state.value.items[selectedChildId];
    return (
      <div style="flex: 3; display: flex; height: 100%; overflow: hidden; gap: 0.5ch;">
        {renderItems(stack, "root", stack.state.value.root, "20ch", selectedId)}
        {renderItems(
          stack,
          selectedId,
          selectedItemChildren,
          "20ch",
          selectedChildId,
        )}
        {selectedChild &&
          (
            <div style="flex: 3; overflow: auto; height: 100%">
              <Preview stack={stack} item={selectedChild} />
            </div>
          )}
      </div>
    );
  }

  const parentItemChildren = stack.getChildren(parentItem);

  return (
    <div style="flex: 3; display: flex; height: 100%; overflow: hidden; gap: 0.5ch;">
      {renderItems(
        stack,
        "root",
        stack.state.value.root,
        "10ch",
        parentItem.id,
      )}
      {renderItems(
        stack,
        parentItem.id,
        parentItemChildren,
        "20ch",
        selectedId,
      )}
      <div style="flex: 3; overflow: auto; height: 100%">
        <Preview stack={stack} item={selectedItem} />
      </div>
    </div>
  );
}

const RowIcon = ({ stack, item }: { stack: Stack; item: Item }) => {
  if (!item.stack_id) return <Icon name="IconStack" />;

  const contentMeta = stack.getContentMeta(item);

  switch (contentMeta.content_type) {
    case "Image":
      return <Icon name="IconImage" />;

    case "Link":
      return <Icon name="IconLink" />;

    case "Text":
      return <Icon name="IconClipboard" />;
  }

  return <Icon name="IconBell" />;
};

function Preview({ stack, item }: { stack: Stack; item: Item }) {
  const content = stack.getContent(item.hash).value;
  if (!content) return <div>loading...</div>;
  const meta = stack.getContentMeta(item);

  if (meta.mime_type === "image/png") {
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
