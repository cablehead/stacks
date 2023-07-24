import { useEffect } from "preact/hooks";

import {
  actionsMode,
  addToStackMode,
  editorMode,
  filterContentTypeMode,
  mainMode,
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

import { createStack, currStack, triggerCopy } from "./stacks";

import { Focus } from "./types";

import { default as theme } from "./theme";

async function globalKeyHandler(event: KeyboardEvent) {
  console.log("GLOBAL", event);

  if (attemptAction(event, currStack.value)) return;

  switch (true) {
    case event.key === "Enter":
      await triggerCopy();
      return;

    case event.key === "Escape":
      event.preventDefault();

      // attempt to clear filter first
      if (currStack.value.filter.dirty()) {
        currStack.value.filter.clear();
        return;
      }

      /*
      // attempt to pop the current stack
      if (currStack.value.parent) {
        currStack.value = currStack.value.parent;
        return;
      }
      */

      // otherwise, hide the window
      currStack.value.selected.value = Focus.first();
      modes.deactivate();
      return;

    case event.metaKey && event.key === "k":
      event.preventDefault();
      modes.toggle(currStack.value, actionsMode);
      return;

    case (event.shiftKey && event.key === "Tab") ||
      (event.ctrlKey && event.key === "h"): {
      if (currStack.value.parent) {
        currStack.value = currStack.value.parent;
        return;
      }
      return;
    }

    case event.ctrlKey && event.key === "l": {
      event.preventDefault();

      // if this is a stack, open it
      const item = currStack.value.item.value;
      if (item && item.content_type == "Stack") {
        const subStack = createStack(item.stack, currStack.value);
        currStack.value = subStack;
        return;
      }
      return;
    }

    case event.key === "Tab": {
      event.preventDefault();

      if (currStack.value.parent) return;

      // if this is a stack, open it
      const item = currStack.value.item.value;
      if (item && item.content_type == "Stack") {
        const subStack = createStack(item.stack, currStack.value);
        currStack.value = subStack;
        return;
      }

      // otherwise, add to stack
      modes.activate(currStack.value, addToStackMode);
      return;
    }

    case (event.metaKey && event.key === "n"):
      event.preventDefault();
      modes.toggle(currStack.value, newNoteMode);
      return;

    case (event.metaKey && event.key === "p"):
      event.preventDefault();
      modes.toggle(currStack.value, filterContentTypeMode);
      return;

    case (event.ctrlKey && event.key === "n") || event.key === "ArrowDown":
      event.preventDefault();
      currStack.value.selected.value = currStack.value.selected.value.down();
      return;

    case event.ctrlKey && event.key === "p" || event.key === "ArrowUp":
      event.preventDefault();
      currStack.value.selected.value = currStack.value.selected.value.up();
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
      modes.activate(currStack.value, mainMode);
      currStack.value.filter.clear();
      currStack.value.selected.value = Focus.first();
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

  return (
    <main
      className={theme.value === "light" ? lightThemeClass : darkThemeClass}
    >
      <Filter stack={currStack.value} />
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
        <Nav stack={currStack.value} />
        <MetaPanel stack={currStack.value} />
        {modes.isActive(addToStackMode) && (
          <addToStackMode.Modal stack={currStack.value} modes={modes} />
        )}
        {modes.isActive(actionsMode) && <Actions stack={currStack.value} />}
        {modes.isActive(editorMode) && (
          <editorMode.Modal stack={currStack.value} modes={modes} />
        )}
        {modes.isActive(newNoteMode) && (
          <newNoteMode.Modal stack={currStack.value} modes={modes} />
        )}
        {modes.isActive(pipeMode) && (
          <pipeMode.Modal stack={currStack.value} modes={modes} />
        )}
        {modes.isActive(filterContentTypeMode) &&
          <filterContentTypeMode.Modal stack={currStack.value} modes={modes} />}
      </div>
      <StatusBar stack={currStack.value} />
    </main>
  );
}
