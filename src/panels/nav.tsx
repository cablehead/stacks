import { useEffect, useRef } from "preact/hooks";

import { b64ToUtf8 } from "../utils";

import { Icon } from "../ui/icons";
import { borderRight } from "../ui/app.css";

import { Item, itemGetContent, Layer, Stack } from "../types";

const TerseRow = (
  { stack, item, isSelected, isFocused, showIcons }: {
    stack: Stack;
    item: Item;
    isSelected: boolean;
    isFocused: boolean;
    showIcons: boolean;
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
      {showIcons &&
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
        {item.terse}
      </div>
    </div>
  );
};

const renderItems = (
  stack: Stack,
  key: string,
  layer: Layer,
  showIcons: boolean,
) => {
  if (layer.items.length == 0) return <i>no items</i>;
  return (
    <div
      key={key}
      className={borderRight}
      style={{
        flex: 1,
        maxWidth: layer.is_focus ? "20ch" : "14ch",
        overflowY: "auto",
        paddingRight: "0.5rem",
      }}
    >
      {layer.items
        .map((item) => (
          <TerseRow
            stack={stack}
            item={item}
            key={item.id}
            isSelected={item.id == layer.selected.id}
            isFocused={layer.is_focus}
            showIcons={showIcons}
          />
        ))}
    </div>
  );
};

export function Nav({ stack }: { stack: Stack }) {
  const nav = stack.nav.value;

  return (
    <div style="flex: 3; display: flex; height: 100%; overflow: hidden; gap: 0.5ch;">
      {nav.root
        ? (
          <>
            {renderItems(stack, "root", nav.root, false)}
            {nav.sub
              ? (
                <>
                  {renderItems(
                    stack,
                    nav.root.selected.id,
                    nav.sub,
                    true,
                  )}
                  <div style="flex: 3; overflow: auto; height: 100%">
                    <Preview
                      stack={stack}
                      item={nav.sub.selected}
                    />
                  </div>
                </>
              )
              : <i>no items</i>}
          </>
        )
        : <i>no matches</i>}
    </div>
  );
}

const RowIcon = ({ item }: { item: Item }) => {
  if (!item.stack_id) return <Icon name="IconStack" />;

  switch (item.content_type) {
    case "Image":
      return <Icon name="IconImage" />;

    case "Link":
      return <Icon name="IconLink" />;

    case "Text":
      return <Icon name="IconClipboard" />;
  }

  return <Icon name="IconBell" />;
};

// @ts-ignore
function Preview({ stack, item }: { stack: Stack; item: Item }) {
  const content = itemGetContent(item);
  if (!content) return <div>loading...</div>;

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

  const preRef = useRef<HTMLPreElement>(null);

  useEffect(() => {
    console.log("<pre> inserted", preRef.current);
  }, [preRef.current]);

  console.log("item", item.id, content.length, item.ephemeral);

  useEffect(() => {
    console.log("Preview component mounted.");

    return () => {
      console.log("Preview component will unmount.");
    };
  }, []);

  useEffect(() => {
    if (preRef.current && content) {
      preRef.current.textContent = b64ToUtf8(content);
      preRef.current.scrollIntoView({ block: "end", behavior: "auto" });
    }
  }, [item]);

  return (
    <div>
      <pre
        key={item.id}
        ref={preRef}
        style="margin: 0; white-space: pre-wrap; overflow-x: hidden"
      ></pre>
    </div>
  );
}
