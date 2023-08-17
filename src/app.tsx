import { useEffect } from "preact/hooks";

import {
  actionsMode,
  addToStackMode,
  editorMode,
  filterContentTypeMode,
  modes,
  newNoteMode,
  pipeMode,
} from "./modals";

import { darkThemeClass, lightThemeClass } from "./ui/app.css";

import { Nav } from "./panels/nav";
import { StatusBar } from "./panels/statusbar";
import { MetaPanel } from "./panels/meta";
import { Actions } from "./panels/actions";
import { Filter } from "./panels/filter";

import { attemptAction } from "./actions";

import { currStack, triggerCopy } from "./stacks";

import { default as theme } from "./theme";

async function globalKeyHandler(event: KeyboardEvent) {
  console.log("GLOBAL", event);

  const stack = currStack.value;
  if (!stack) return;

  if (attemptAction(event, stack)) return;

  switch (true) {
    case event.key === "Enter":
      await triggerCopy();
      return;

    case event.key === "Escape":
      event.preventDefault();

      // attempt to clear filter first
      if (stack.filter.dirty()) {
        stack.filter.clear();
        return;
      }

      /*
      // attempt to pop the current stack
      if (stack.parent) {
        stack = stack.parent;
        return;
      }
      */

      // otherwise, hide the window
      // stack.selected.value = Focus.first();
      modes.deactivate();
      return;

    case event.metaKey && event.key === "k":
      event.preventDefault();
      modes.toggle(stack, actionsMode);
      return;

    case (event.shiftKey && event.key === "Tab") ||
      (event.ctrlKey && event.key === "h"): {
      event.preventDefault();
      return;
    }

    case event.ctrlKey && event.key === "l": {
      event.preventDefault();

      /*
      // if this is a stack, open it
      const item = stack.item.value;

      if (!item) return;
      const meta = stack.getContentMeta(item);

      if (meta.content_type == "Stack") {
        // const subStack = createStack(item.stack, stack);
        // stack = subStack;
        return;
      }
      return;
      */
    }

    case event.key === "Tab": {
      event.preventDefault();

      /*
      if (stack.parent) return;

      // if this is a stack, open it
      const item = stack.item.value;
      if (item && item.content_type == "Stack") {
        // const subStack = createStack(item.stack, stack);
        // stack = subStack;
        return;
      }

      // otherwise, add to stack
      modes.activate(stack, addToStackMode);
      return;
      */
    }

    case (event.metaKey && event.key === "n"):
      event.preventDefault();
      modes.toggle(stack, newNoteMode);
      return;

    case (event.metaKey && event.key === "p"):
      event.preventDefault();
      modes.toggle(stack, filterContentTypeMode);
      return;

    case (event.ctrlKey && event.key === "n") || event.key === "ArrowDown":
      event.preventDefault();
      // stack.selected.value = stack.selected.value.down();
      return;

    case event.ctrlKey && event.key === "p" || event.key === "ArrowUp":
      event.preventDefault();
      // stack.selected.value = stack.selected.value.up();
      return;

    case (event.metaKey && (event.key === "Meta" || event.key === "c")):
      // avoid capturing command-c
      return;

    default:
      // fallback to sending the key stroke to the filter input
      const filterInput = document.getElementById("filter-input");
      if (filterInput) filterInput.focus();
  }
}

export function App() {
  const NAV_TIMEOUT = 30 * 1000; // 30 seconds

  let blurTime: number | null = null;

  const onBlurHandler = () => {
    blurTime = Date.now();
  };

  const onFocusHandler = () => {
    if (blurTime && Date.now() - blurTime > NAV_TIMEOUT) {
      console.log("NAV_TIMEOUT: reset");
      // modes.activate(stack, mainMode);
      // stack.filter.clear();
      // stack.selected.value = Focus.first();
    }
  };

  useEffect(() => {
    window.addEventListener("keydown", globalKeyHandler);
    window.addEventListener("blur", onBlurHandler);
    window.addEventListener("focus", onFocusHandler);

    return () => {
      window.removeEventListener("keydown", globalKeyHandler);
      window.removeEventListener("blur", onBlurHandler);
      window.removeEventListener("focus", onFocusHandler);
    };
  }, []);

  const stack = currStack.value;

  return (
    <main
      className={theme.value === "light" ? lightThemeClass : darkThemeClass}
    >
      {stack
        ? (
          <>
            <Filter stack={stack} />
            <div style="
            display: flex;
            flex-direction: column;
            height: 100%;
            width: 100%;
            overflow: hidden;
            padding-top:1ch;
            padding-left:1ch;
            padding-right:1ch;
            position: relative;
        ">
              <Nav stack={stack} />

              <MetaPanel stack={stack} />

              {modes.isActive(addToStackMode) && (
                <addToStackMode.Modal stack={stack} modes={modes} />
              )}
              {modes.isActive(actionsMode) && <Actions stack={stack} />}
              {modes.isActive(editorMode) && (
                <editorMode.Modal stack={stack} modes={modes} />
              )}
              {modes.isActive(newNoteMode) && (
                <newNoteMode.Modal stack={stack} modes={modes} />
              )}
              {modes.isActive(pipeMode) && (
                <pipeMode.Modal stack={stack} modes={modes} />
              )}
              {modes.isActive(filterContentTypeMode) &&
                <filterContentTypeMode.Modal stack={stack} modes={modes} />}
            </div>
            <StatusBar stack={stack} />
          </>
        )
        : "loading"}
    </main>
  );
}
