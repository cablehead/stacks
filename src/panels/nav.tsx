import { useSignal } from "@preact/signals";

import { Icon } from "../ui/icons";
import { borderRight } from "../ui/app.css";

import { Item, Stack, LoadedItem } from "../types";
import { createStack } from "../stacks";

export function Nav({ stack, parent }: { stack: Stack; parent?: boolean }) {
  return (
    <>
      <div
        className={borderRight}
        style="
      flex: 1;
      max-width: 20ch;
      overflow-y: auto;
      padding-right: 0.5rem;
    "
      >
        {stack.items.value
          .map((item, index) => {
            return <TerseRow stack={stack} item={item} index={index} />;
          })}
      </div>

      <RightPane
        loaded={stack.loaded.value}
        parent={parent}
      />
    </>
  );
}

const RowIcon = ({ item }: { item: Item }) => {
  switch (item.content_type) {
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

const TerseRow = (
  { stack, item, index }: { stack: Stack; item: Item; index: number },
) => (
  <div
    className={"terserow" +
      (index === stack.selected.value ? " selected" : "")}
    onClick={() => stack.selected.value = index}
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
      <RowIcon item={item} />
    </div>

    <div
      style={{
        flexGrow: 1,
        whiteSpace: "nowrap",
        overflow: "hidden",
        textOverflow: "ellipsis",
      }}
    >
      {item.terse}
    </div>
  </div>
);

function RightPane(
  { loaded, parent }: {
    loaded: LoadedItem | undefined;
    parent?: boolean;
  },
) {
  return (
    <div style="flex: 3; overflow: auto; height: 100%">
      {loaded
        ? <Preview item={loaded.item} content={loaded.content} parent={parent} />
        : "loading..."}
    </div>
  );
}

function Preview(
  { item, content }: { item: Item; content: string; parent?: boolean; },
) {
  if (item.mime_type === "image/png") {
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

  if (!parent && item.content_type == "Stack") {
    return <Nav stack={createStack(useSignal(item.stack))} parent={true} />;
  }

  if (item.link) {
    return (
      <img
        src={item.link.screenshot}
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
    { content !== undefined ? content : "loading..." }
    </pre>
  );
}
