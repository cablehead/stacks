import { useEffect } from "preact/hooks";

import {
  actionsMode,
  addToStackMode,
  editorMode,
  filterContentTypeMode,
  mainMode,
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

import { currStack, triggerCopy } from "./stacks";

import { themeMode } from "./modals/mainMode";

async function globalKeyHandler(event: KeyboardEvent) {
  console.log("GLOBAL", event);
  switch (true) {
    case event.key === "Enter":
      await triggerCopy();
      break;

    case event.key === "Escape":
      event.preventDefault();

      // attempt to clear filter first
      if (mainMode.state.dirty()) {
        mainMode.state.clear();
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
      /* todo:
      const parents = currStack.value.parents;
      if (parents.length > 0) {
          console.log("switch", currStack.value);
        currStack.value = parents[0];
      }
      */
      return;
    }

    case event.key === "Tab": {
      event.preventDefault();

      /* todo:
      const loaded = currStack.loaded.value;
      if (!loaded) return;
      if (loaded.item.content_type == "Stack") {
        const subStack = createStack(
          signal(loaded.item.stack),
          currStack,
        );
        currStack.value = subStack;

        return;
      }
      */

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
      if (mainMode.state.input !== null) {
        mainMode.state.input.focus();
      }
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
      className={themeMode.value === "light" ? lightThemeClass : darkThemeClass}
    >
      <Filter />
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

