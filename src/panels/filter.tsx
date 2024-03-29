import { useEffect, useRef } from "preact/hooks";

import { borderBottom } from "../ui/app.css";
import { Icon, RenderKeys } from "../ui/icons";
import { VertDiv } from "../ui/components";

import { Stack } from "../types";
import { filterContentTypeMode, modes, newMode } from "../modals";

export function Filter({ stack }: { stack: Stack }) {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (inputRef.current != null) {
      inputRef.current.focus();
    }
  }, []);

  useEffect(() => {
    if (inputRef.current != null) {
      if (inputRef.current.value != stack.filter.curr.value) {
        inputRef.current.value = stack.filter.curr.value;
      }
    }
  }, [stack.filter.curr.value]);

  const nav = stack.nav.value;

  return (
    <div
      className={borderBottom}
      style={{
        padding: "1ch",
        paddingLeft: "2ch",
        height: "5ch",
        paddingBottom: "0.25ch",
        display: "flex",
        width: "100%",
        gap: "0.5ch",
        alignItems: "center",
      }}
    >
      <div>&gt;</div>
      <div
        style={{
          flexGrow: "1",
        }}
      >
        <input
          id="filter-input"
          type="text"
          placeholder="Type to filter..."
          ref={inputRef}
          onInput={(event) =>
            stack.filter.curr.value = (event.target as HTMLInputElement).value}
        />
      </div>

      <VertDiv />
      <div
        id="filter-content-type"
        class="hoverable"
        onMouseDown={() => modes.toggle(stack, filterContentTypeMode)}
        style={{
          fontSize: "0.9rem",
          display: "flex",
          alignItems: "center",
        }}
      >
        {stack.filter.content_type.value == "All"
          ? "Filter by type"
          : stack.filter.content_type.value}&nbsp;
        <RenderKeys keys={[<Icon name="IconCommandKey" />, "U"]} />
      </div>

      <VertDiv />
      <div
        id="trigger-new"
        class="hoverable"
        onMouseDown={() => modes.toggle(stack, newMode)}
        style={{
          fontSize: "0.9rem",
          display: "flex",
          alignItems: "center",
        }}
      >
        New&nbsp;
        <RenderKeys keys={[<Icon name="IconCommandKey" />, "N"]} />
      </div>

      {nav.undo &&
        (
          <>
            <VertDiv />
            <div
              class="hoverable"
              onMouseDown={() => stack.undo()}
              style={{
                fontSize: "0.9rem",
                display: "flex",
                alignItems: "center",
              }}
            >
              Undo delete&nbsp;
              <RenderKeys keys={[<Icon name="IconCommandKey" />, "Z"]} />
            </div>
          </>
        )}
    </div>
  );
}
