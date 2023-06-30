import { forwardRef } from "preact/compat";
import { useEffect, useRef } from "preact/hooks";

import { Icon } from "../ui/icons";
import { borderRight } from "../ui/app.css";

import { Item, Stack } from "../types";
import { createStack } from "../stacks";

/*
export function RenderStack({ stack }: { stack: Stack }) {
  const parent = stack.parents[0];
  return (
    <div style="display: flex; height: 100%; overflow: hidden; gap: 0.5ch;">
      {parent &&
        (
          <div
            className={borderRight}
            style="
      flex: 1;
      max-width: 10ch;
      overflow-y: auto;
      padding-right: 0.5rem;
    "
          >
            {parent.items.value
              .map((item, index) => {
                return <TerseRow stack={parent} item={item} index={index} />;
              })}
          </div>
        )}
    <Nav stack={stack} />
    </div>
  );
}
*/

export function Nav({ stack }: { stack: Stack }) {
  const theRef = useRef<HTMLDivElement>(null);

  let focusSelectedTimeout: number | undefined;

  function focusSelected(delay: number) {
    if (focusSelectedTimeout !== undefined) {
      return;
    }

    focusSelectedTimeout = window.setTimeout(() => {
      focusSelectedTimeout = undefined;
      if (theRef.current) {
        console.log("SCROLL INTO VIEW");
        theRef.current.scrollIntoView({
          behavior: "smooth",
          block: "nearest",
        });
      }
    }, delay);
  }

  useEffect(() => {
    focusSelected(5);
  }, [theRef.current]);

  useEffect(() => {
    const onFocus = () => {
      focusSelected(100);
    };
    window.addEventListener("focus", onFocus);
    return () => {
      window.removeEventListener("focus", onFocus);
    };
  }, []);

  return (
    <div style="flex: 3; display: flex; height: 100%; overflow: hidden; gap: 0.5ch;">
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
            return (
              <TerseRow
                stack={stack}
                item={item}
                index={index}
                ref={index === stack.normalizedSelected.value ? theRef : null}
                key={index}
              />
            );
          })}
      </div>

      <div style="flex: 3; overflow: auto; height: 100%">
        <Preview stack={stack} />
      </div>
    </div>
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

const TerseRow = forwardRef<
  HTMLDivElement,
  { stack: Stack; item: Item; index: number }
>(
  ({ stack, item, index }, ref) => (
    <div
      ref={ref}
      className={"terserow" +
        (index === stack.normalizedSelected.value ? " selected" : "")}
      onClick={() => {
        /* todo:
        if (currStack.value != stack) {
          console.log("Switcheroo");
          currStack.value = stack;
        }
        */
        stack.selected.value = index;
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
  ),
);

function Preview({ stack }: { stack: Stack }) {
  const item = stack.item.value;
  const content = stack.content?.value;
  if (!item || !content) return <div>loading...</div>;

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

  if (item.content_type == "Stack") {
    const subStack = createStack(item.stack);
    return <Nav stack={subStack} />;
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
    { content }
    </pre>
  );
}
