import { useEffect, useRef } from "preact/hooks";

import { b64ToUtf8 } from "../utils";

import { Icon } from "../ui/icons";
import { borderRight } from "../ui/app.css";

import { Item, itemGetContent, Layer, Stack } from "../types";

/*
import Prism from "prismjs";
window.Prism = Prism;
import "prismjs/components/prism-rust";
import "prism-themes/themes/prism-one-dark.css";
*/

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

  useEffect(() => {
    console.log("component: mounted.");
    return () => {
      console.log("component: will unmount.");
    };
  }, []);

  const preRef = useRef<HTMLPreElement>(null);

  useEffect(() => {
    console.log("<pre> inserted", preRef.current);
  }, [preRef.current]);

  useEffect(() => {
    if (!preRef.current) return;

    const item = nav.sub ? nav.sub.selected : null;
    if (!item) return;

    const content = itemGetContent(item);
    if (!content) return;

    preRef.current.textContent = b64ToUtf8(content);
    /*
    if (item.hash == "sha256-0UDbFR5u3lzm+mrjUy5ZLgVbU57It1YMUbX5CN11gYs=") {
      Prism.highlightElement(preRef.current);
    }
    */
    if (item.ephemeral) {
      preRef.current.scrollIntoView({ block: "end", behavior: "auto" });
    }
  }, [nav.sub?.selected.hash]);

  const anchorRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    console.log("1", anchorRef.current);
    if (anchorRef.current) {
      const scrollMeElem = anchorRef.current.querySelector(".scroll-me");
      console.log("2", scrollMeElem);
      if (scrollMeElem) {
        console.log("scroll scroll");
        // scrollMeElem.scrollTop = scrollMeElem.scrollHeight;
        scrollMeElem.scrollIntoView({ block: "end", behavior: "auto" });
      }
    }
  }, [nav.sub?.preview]);

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
                    style="flex: 3; overflow: auto; height: 100%"
                    ref={anchorRef}
                    dangerouslySetInnerHTML={{
                      __html: nav.sub?.preview || "<i>loading</i>",
                    }}
                  >
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

    case "Markdown":
      return <Icon name="IconDocument" />;

    case "Rust":
    case "JSON":
    case "Python":
    case "JavaScript":
    case "Rust":
    case "HTML":
    case "Shell":
    case "Go":
    case "Ruby":
    case "SQL":
    case "XML":
    case "YAML":
      return <Icon name="IconCode" />;
  }

  return <Icon name="IconBell" />;
};
