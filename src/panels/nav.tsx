import { useSignal } from "@preact/signals";

import { Icon } from "../ui/icons";
import { borderRight } from "../ui/app.css";

import { Item, Stack } from "../types";
import { createStack } from "../stacks";

export function Nav({ stack }: { stack: Stack }) {
  return (
    <>
      <div style="display: flex; height: 100%; overflow: hidden; gap: 0.5ch;">
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

        <div style="flex: 3; overflow: auto; height: 100%">
          <Preview stack={stack} />
        </div>
      </div>
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
      (index === stack.normalizedSelected.value ? " selected" : "")}
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

function Preview({ stack }: { stack: Stack }) {
  const loaded = stack.loaded.value;
  if (!loaded) return <div>loading..."</div>;

  if (loaded.item.mime_type === "image/png") {
    return (
      <img
        src={"data:image/png;base64," + loaded.content}
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

  if (loaded && loaded.item.content_type == "Stack") {
    const subStack = createStack(useSignal(loaded.item.stack), stack);
    if (subStack.parents.length <= 1) return <Nav stack={subStack} />;
  }

  if (loaded.item.link) {
    return (
      <img
        src={loaded.item.link.screenshot}
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
    { loaded.content }
    </pre>
  );
}
