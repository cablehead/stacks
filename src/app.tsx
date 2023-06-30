import { useEffect } from "preact/hooks";

import {
  actionsMode,
  addToStackMode,
  editorMode,
  filterContentTypeMode,
  modes,
} from "./modals";

import { darkThemeClass, lightThemeClass } from "./ui/app.css";

import { Nav } from "./panels/nav";
import { StatusBar } from "./panels/statusbar";
import { MetaPanel } from "./panels/meta";
import { Actions } from "./panels/actions";
import { Editor } from "./panels/editor";
import { Filter } from "./panels/filter";

import { attemptAction } from "./actions";

import { createStack, currStack, triggerCopy } from "./stacks";

import { default as state } from "./state";

async function globalKeyHandler(event: KeyboardEvent) {
  console.log("GLOBAL", event);
  switch (true) {
    case event.key === "Enter":
      await triggerCopy();
      break;

    case event.key === "Escape":
      event.preventDefault();

      // attempt to clear filter first
      if (currStack.value.filter.dirty()) {
        currStack.value.filter.clear();
        return;
      }

      // attempt to pop the current stack
      if (currStack.value.parent) {
        currStack.value = currStack.value.parent;
        return;
      }

      // otherwise, hide the window
      currStack.value.selected.value = 0;
      modes.deactivate();
      return;

    case event.metaKey && event.key === "k":
      event.preventDefault();
      modes.toggle(actionsMode);
      break;

    case event.shiftKey && event.key === "Tab": {
      if (currStack.value.parent) {
        currStack.value = currStack.value.parent;
        return;
      }
      return;
    }

    case event.key === "Tab": {
      event.preventDefault();

      // if this is a stack, open it
      const item = currStack.value.item.value;
      if (item && item.content_type == "Stack") {
        const subStack = createStack(item.stack, currStack.value);
        currStack.value = subStack;
        return;
      }

      // otherwise, add to stack
      modes.activate(addToStackMode);
      return;
    }

    case (event.metaKey && event.key === "p"):
      event.preventDefault();
      modes.toggle(filterContentTypeMode);
      break;

    case (event.ctrlKey && event.key === "n") || event.key === "ArrowDown":
      event.preventDefault();
      currStack.value.selected.value += 1;
      break;

    case event.ctrlKey && event.key === "p" || event.key === "ArrowUp":
      event.preventDefault();
      currStack.value.selected.value -= 1;
      break;

    case (event.metaKey && (event.key === "Meta" || event.key === "c")):
      // avoid capturing command-c
      return;

    default:
      if (attemptAction(event, currStack.value)) return;
      /*
      if (currStack.value.filter.input !== null) {
        currStack.value.filter.input.focus();
      }
      */
  }
}

export function App() {
  useEffect(() => {
    window.addEventListener("keydown", globalKeyHandler);
    return () => {
      window.removeEventListener("keydown", globalKeyHandler);
    };
  }, []);

  return (
    <main
      className={state.themeMode.value === "light"
        ? lightThemeClass
        : darkThemeClass}
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
          <addToStackMode.Modal modes={modes} />
        )}
        {modes.isActive(actionsMode) && <Actions stack={currStack.value} />}
        {modes.isActive(editorMode) && <Editor stack={currStack.value} />}
      </div>
      <StatusBar />
    </main>
  );
}
