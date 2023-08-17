import { forwardRef } from "preact/compat";
import { useEffect, useRef } from "preact/hooks";

import { b64ToUtf8 } from "../utils";

import { Icon } from "../ui/icons";
import { borderRight } from "../ui/app.css";

import { ContentMeta, Focus, Item, Stack } from "../types";

const renderItems = (
  stack: Stack,
  items: string[],
  maxWidth: string,
  selectedId?: string,
) => (
  <div
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

export function Nav(
  { stack, preview }: {
    stack: Stack;
    preview: string | undefined;
  },
) {
  const theRef = useRef<HTMLDivElement>(null);

  let focusSelectedTimeout: number | undefined;
  function focusSelected(delay: number) {
    clearTimeout(focusSelectedTimeout);
    focusSelectedTimeout = window.setTimeout(() => {
      if (theRef.current) {
        console.log("STACK: SCROLL INTO VIEW");
        theRef.current.scrollIntoView({
          behavior: "smooth",
          block: "nearest",
        });
      }
    }, delay);
  }

  useEffect(() => {
    focusSelected(10);
  }, [theRef.current, stack.selected.value]);

  useEffect(() => {
    const onFocus = () => {
      focusSelected(100);
    };
    window.addEventListener("focus", onFocus);
    return () => {
      window.removeEventListener("focus", onFocus);
    };
  }, []);

  const selectedId = stack.selected.value.curr(stack);
  const selectedItem = stack.state.value.items[selectedId];

  const parentItem = selectedItem.stack_id &&
    stack.state.value.items[selectedItem.stack_id];

  const items = parentItem ? parentItem.children : stack.state.value.root;

  const previewItem = preview && stack.state.value.items[preview];

  return (
    <div style="flex: 3; display: flex; height: 100%; overflow: hidden; gap: 0.5ch;">
      {parentItem &&
        renderItems(stack, stack.state.value.root, "8ch", parentItem.id)}
      {renderItems(stack, items, "20ch")}
      <div style="flex: 3; overflow: auto; height: 100%">
        {previewItem
          ? <Preview stack={stack} item={previewItem} />
          : <i>no matches</i>}
      </div>
    </div>
  );
}

const RowIcon = ({ contentMeta }: { contentMeta: ContentMeta }) => {
  switch (contentMeta.content_type) {
    case "Stack":
      return <Icon name="IconStack" />;

    case "Image":
      return <Icon name="IconImage" />;

    case "Link":
      return <Icon name="IconLink" />;

    case "Text":
      return <Icon name="IconClipboard" />;
  }

  return <Icon name="IconBell" />;
};

const TerseRow = forwardRef<
  HTMLDivElement,
  { stack: Stack; item: Item; selectedId?: string }
>(
  ({ stack, item, selectedId }, ref) => {
    const meta = stack.getContentMeta(item);

    return (
      <div
        ref={ref}
        className={"terserow" +
          (stack.selected.value.curr(stack) === item.id ? " highlight" : "") +
          (item.id === selectedId ? " selected" : "")}
        onMouseDown={() => {
          stack.selected.value = Focus.id(item.id);
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
        <div
          style={{
            flexShrink: 0,
            width: "2ch",
            whiteSpace: "nowrap",
            overflow: "hidden",
          }}
        >
          <RowIcon contentMeta={meta} />
        </div>

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
  },
);

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

  if (!item.stack_id) {
    const childrenItems = item.children.map((childId) =>
      stack.state.value.items[childId]
    );
    const firstChildPreview = childrenItems[0] && (
      <Preview stack={stack} item={childrenItems[0]} />
    );

    return (
      <div style="flex: 3; overflow: auto; height: 100%">
        {renderItems(stack, item.children, "20ch", item.children[0])}
        {firstChildPreview}
      </div>
    );
  }

  return (
    <pre style="margin: 0; white-space: pre-wrap; overflow-x: hidden">
    { b64ToUtf8(content) }
    </pre>
  );
}
