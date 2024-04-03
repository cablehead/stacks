import { useEffect, useRef } from "preact/hooks";

import { Icon } from "../ui/icons";
import { borderRight, previewItem } from "../ui/app.css";

import { Content, getContent, Item, Layer, Stack } from "../types";

const TerseRow = (
  { stack, item, isSelected, isFocused, content }: {
    stack: Stack;
    item: Item;
    isSelected: boolean;
    isFocused: boolean;
    content: Content | null;
  },
) => {
  const theRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (isSelected && theRef.current) {
      const fontSize = parseFloat(getComputedStyle(theRef.current).fontSize);
      const yOffset = -2.5 * fontSize; // Desired offset from the top
      const parent = theRef.current.parentElement;
      if (parent) {
        // Calculate new scrollTop position with offset
        const topPosition = theRef.current.offsetTop + yOffset;
        // Apply calculated scrollTop position directly
        parent.scrollTop = topPosition;
      }
    }
  }, [item]);

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
      {content &&
        (
          <div
            style={{
              flexShrink: 0,
              width: "2ch",
              whiteSpace: "nowrap",
              overflow: "hidden",
            }}
          >
            <RowIcon item={item} content={content} />
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
        {content?.terse || item.name}
      </div>
    </div>
  );
};

const renderItems = (
  stack: Stack,
  key: string,
  layer: Layer,
  isRoot: boolean,
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
        scrollBehavior: "smooth",
      }}
    >
      {layer.items
        .map((item) => {
          let content = null;
          if (isRoot) {
            content = getContent(item).value;
          }
          return (
            <TerseRow
              stack={stack}
              item={item}
              key={item.id}
              isSelected={item.id == layer.selected.id}
              isFocused={layer.is_focus}
              content={content}
            />
          );
        })}

      <div style={{ height: `calc(100% - 2.5em)` }}></div>
    </div>
  );
};

export function Preview(
  { content, active, ...rest }:
    & { content: string; active: boolean }
    & JSX.HTMLAttributes,
) {
  const anchorRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (active && anchorRef.current) {
      const fontSize = parseFloat(getComputedStyle(anchorRef.current).fontSize);
      const yOffset = -2.5 * fontSize; // Desired offset from the top
      const parent = anchorRef.current.parentElement;
      if (parent) {
        const topPosition = anchorRef.current.offsetTop + yOffset;
        parent.scrollTop = topPosition;
      }
    }
  }, [active, anchorRef]);

  return (
    <div
      className={`${previewItem} ${active ? "active" : "not-active"}`}
      ref={anchorRef as any}
      dangerouslySetInnerHTML={{
        __html: content || "<i>loading</i>",
      }}
      {...rest}
    >
    </div>
  );
}

export function Nav({ stack }: { stack: Stack }) {
  const nav = stack.nav.value;

  useEffect(() => {
    console.log("component: mounted.");
    return () => {
      console.log("component: will unmount.");
    };
  }, []);

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

                  <div
                    style={{
                      flex: 3,
                      overflow: "auto",
                      display: "flex",
                      flexDirection: "column",
                      scrollBehavior: "smooth",
                    }}
                  >
                    {nav.sub.items.map((item) => {
                      return (
                        <Preview
                          onMouseDown={() => {
                            if (!item) return;
                            stack.select(item.id);
                          }}
                          content={getContent(item).value?.preview || ""}
                          active={item?.id == nav.sub?.selected.id}
                        />
                      );
                    })}
                    <div style={{ minHeight: `calc(100% - 2.5em)` }}></div>
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

const RowIcon = (
  { item, content }: { item: Item; content: Content | null },
) => {
  if (!item.stack_id) return <Icon name="IconStack" />;

  if (!content) return <Icon name="IconClipboard" />;

  switch (content.content_type) {
    case "Image":
      return <Icon name="IconImage" />;

    case "Link":
      return <Icon name="IconLink" />;

    case "Text":
      return <Icon name="IconClipboard" />;

    case "Markdown":
      return <Icon name="IconDocument" />;

    // TODO: oh my
    case "C":
    case "C++":
    case "CSS":
    case "Diff":
    case "Erlang":
    case "Go":
    case "Graphviz":
    case "HTML":
    case "Haskell":
    case "Java":
    case "JSON":
    case "JavaScript":
    case "Lisp":
    case "Lua":
    case "Makefile":
    case "MATLAB":
    case "OCaml":
    case "Objective-C":
    case "PHP":
    case "Perl":
    case "Python":
    case "R":
    case "Regular Expression":
    case "reStructuredText":
    case "Ruby":
    case "Rust":
    case "Shell":
    case "SQL":
    case "XML":
    case "YAML":
      return <Icon name="IconCode" />;
  }

  return <Icon name="IconBell" />;
};
