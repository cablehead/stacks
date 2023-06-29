import { useEffect } from "preact/hooks";

import {
  actionsMode,
  addToStackMode,
  editorMode,
  filterContentTypeMode,
  mainMode,
  modes,
} from "./modals";

import { Event, listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";

import { darkThemeClass, lightThemeClass } from "./ui/app.css";

import { Nav } from "./panels/nav";
import { StatusBar } from "./panels/statusbar";
import { MetaPanel } from "./panels/meta";
import { Actions } from "./panels/actions";
import { Editor } from "./panels/editor";
import { Filter } from "./panels/filter";

import { attemptAction } from "./actions";

import { stack, triggerCopy } from "./stacks";

import { Item } from "./types";
import { focusSelected, themeMode } from "./modals/mainMode";

async function globalKeyHandler(event: KeyboardEvent) {
  console.log("GLOBAL", event);
  switch (true) {
    case event.key === "Enter":
      await triggerCopy();
      break;

    case event.key === "Escape":
      event.preventDefault();
      if (mainMode.state.dirty()) {
        mainMode.state.clear();
        return;
      }
      modes.deactivate();
      return;

    case event.metaKey && event.key === "k":
      event.preventDefault();
      modes.toggle(actionsMode);
      break;

    case event.key === "Tab":
      event.preventDefault();
      modes.activate(addToStackMode);
      break;

    case (event.metaKey && event.key === "p"):
      event.preventDefault();
      modes.toggle(filterContentTypeMode);
      break;

    case (event.ctrlKey && event.key === "n") || event.key === "ArrowDown":
      event.preventDefault();
      stack.value.selected.value += 1;
      break;

    case event.ctrlKey && event.key === "p" || event.key === "ArrowUp":
      event.preventDefault();
      stack.value.selected.value -= 1;
      break;

    case (event.metaKey && (event.key === "Meta" || event.key === "c")):
      // avoid capturing command-c
      return;

    default:
      if (stack.value.loaded.value) {
        if (attemptAction(event, stack.value.loaded.value)) return;
      }

      if (mainMode.state.input !== null) {
        mainMode.state.input.focus();
      }
  }
}

function Main() {
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
        <Nav stack={stack.value} />

        {stack.value.loaded.value &&
          (
            <MetaPanel
              loaded={stack.value.loaded.value}
            />
          )}

        {modes.isActive(addToStackMode) &&
          <addToStackMode.Modal modes={modes} />}

        {stack.value.loaded.value && modes.isActive(actionsMode) &&
          <Actions loaded={stack.value.loaded.value} />}

        {stack.value.loaded.value && modes.isActive(editorMode) &&
          <Editor loaded={stack.value.loaded.value} />}
      </div>
      <StatusBar />
    </main>
  );
}

export function App() {
  useEffect(() => {
    listen("recent-items", (event: Event<Item[]>) => {
      console.log("Data pushed from Rust:", event);
      stack.value.items.value = event.payload;
    });

    async function init() {
      stack.value.items.value = await invoke<Item[]>("init_window");
    }
    init();

    // set selection back to the top onBlur
    const onBlur = () => {
      stack.value.selected.value = 0;
    };
    const onFocus = () => {
      focusSelected(100);
    };

    window.addEventListener("blur", onBlur);
    window.addEventListener("focus", onFocus);

    return () => {
      window.removeEventListener("blur", onBlur);
      window.removeEventListener("focus", onFocus);
    };
  }, []);

  return <Main />;
}
