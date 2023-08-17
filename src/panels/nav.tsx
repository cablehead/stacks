import { forwardRef } from "preact/compat";
import { useEffect, useRef } from "preact/hooks";

import { b64ToUtf8 } from "../utils";

import { Icon } from "../ui/icons";
import { borderRight } from "../ui/app.css";

import { ContentMeta, Focus, Item, Stack } from "../types";

export function Parent({ stack }: { stack: Stack }) {
  const theRef = useRef<HTMLDivElement>(null);

  let focusSelectedTimeout: number | undefined;

  function focusSelected(delay: number) {
    if (focusSelectedTimeout !== undefined) {
      return;
    }

    focusSelectedTimeout = window.setTimeout(() => {
      focusSelectedTimeout = undefined;
      if (theRef.current) {
        // console.log("PARENT STACK: SCROLL INTO VIEW: SKIP");
        /*
        theRef.current.scrollIntoView({
          block: "start",
        });
        */
      }
    }, delay);
  }

  useEffect(() => {
    focusSelected(100);
  }, []);

  return (
    <div
      className={borderRight}
      style="
      flex: 1;
      max-width: 8ch;
      overflow-y: auto;
      padding-right: 0.5rem;
    "
    >
      {stack.state.value.root.map((id) => stack.state.value.items[id])
        .map((item) => {
          return (
            <TerseRow
              ref={/* index === stack.normalizedSelected.value ? theRef :*/ null}
              stack={stack}
              item={item}
            />
          );
        })}
    </div>
  );
}

export function Nav({ stack }: { stack: Stack }) {
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

  return (
    <div style="flex: 3; display: flex; height: 100%; overflow: hidden; gap: 0.5ch;">
      {false && <Parent stack={stack} />}
      <div
        className={borderRight}
        style="
      flex: 1;
      max-width: 20ch;
      overflow-y: auto;
      padding-right: 0.5rem;
    "
      >
        {stack.state.value.root.map((id) => stack.state.value.items[id])
          .map((item, index) => {
            return (
              <TerseRow
                stack={stack}
                item={item}
                ref={/* index === stack.normalizedSelected.value ? theRef : */ null}
                key={index}
              />
            );
          })}
      </div>

      <div style="flex: 3; overflow: auto; height: 100%">
        <Preview stack={stack} />

        {
          /*stack.items.value.length > 0
          ? <Preview stack={stack} />
          : <i>no matches</i> */
        }
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
  { stack: Stack; item: Item }
>(
  ({ stack, item }, ref) => {
    const meta = stack.getContentMeta(item);

    return (
      <div
        ref={ref}
        className={"terserow" +
          (stack.selected.value.curr(stack) === item.id
            // ? (currStack.value === stack ? " highlight" : " selected")
            ? " highlight"
            : "")}
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

function Preview({ stack }: { stack: Stack }) {
  const item = stack.item.value;
  const content = stack.content?.value;
  if (!item || !content) return <div>loading...</div>;
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
    return (
      <div>
        {meta.terse}
        <br />
        <br />
        Stack: {item.children.length}

        <div>
          {item.children.map((child_id) => {
            const child = stack.state.value.items[child_id];
            const meta = stack.getContentMeta(child);
            return meta;
          }).map((meta) => <div>{meta.terse}</div>)}
        </div>
      </div>
    );
  }

  return (
    <pre style="margin: 0; white-space: pre-wrap; overflow-x: hidden">
    { b64ToUtf8(content) }
    </pre>
  );
}
