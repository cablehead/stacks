import { useEffect, useRef } from "preact/hooks";

import { b64ToUtf8 } from "../utils";

import { Icon } from "../ui/icons";
import { borderRight } from "../ui/app.css";

import { Stack, ItemMeta, Scru128Id } from "../types";

const TerseRow = (
  { stack, item, selectedId }: {
    stack: Stack;
    item: ItemMeta;
    selectedId?: Scru128Id;
  },
) => {
  const theRef = useRef<HTMLDivElement>(null);
  const isSelected = selectedId === item.o.id;

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
        (stack.selected.value.curr(stack) === item.o.id ? " highlight" : "") +
        (isSelected ? " selected" : "")}
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
  items: ItemMeta[],
  maxWidth: string,
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
      {items
        .map((item) => (
          <TerseRow
            stack={stack}
            item={item}
            key={item.o.id}
            selectedId={items[0].o.id}
          />
        ))}
    </div>
  );
};

export function Nav({ stack }: { stack: Stack }) {
  return (
    <div style="flex: 3; display: flex; height: 100%; overflow: hidden; gap: 0.5ch;">
      {renderItems(stack, "root", stack.neo.value.root, "20ch")}
      {renderItems(stack, "", stack.neo.value.sub, "20ch")}
      <div style="flex: 3; overflow: auto; height: 100%">
        <Preview stack={stack} item={stack.neo.value.preview} />
      </div>
    </div>
  );

  /*

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
  */
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
