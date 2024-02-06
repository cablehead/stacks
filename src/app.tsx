import { useEffect } from "preact/hooks";

import {
  actionsMode,
  editorMode,
  mainMode,
  modes,
  newNoteMode,
  pipeToCommand,
  settingsMode,
} from "./modals";

import { darkThemeClass, lightThemeClass } from "./ui/app.css";

import { Nav } from "./panels/nav";
import { StatusBar } from "./panels/statusbar";
import { MetaPanel } from "./panels/meta";
import { Actions } from "./panels/actions";
import { Filter } from "./panels/filter";

import { attemptAction } from "./actions";

import { Stack } from "./types";

import { invoke } from "@tauri-apps/api/tauri";

import { default as theme } from "./theme";

const stack = new Stack({});

async function globalKeyHandler(event: KeyboardEvent) {
  if (!stack) return;

  function adjustFontSize(delta: number) {
    const currentFontSize =
      window.getComputedStyle(document.documentElement).fontSize;
    const newFontSize = parseFloat(currentFontSize) + delta + "px";
    document.documentElement.style.fontSize = newFontSize;
  }

  switch (true) {
    case event.metaKey && event.shiftKey && event.key === "=":
      adjustFontSize(1);
      return;

    case event.metaKey && event.key === "-":
      adjustFontSize(-1);
      return;
  }

  if (modes.attemptAction(event, stack)) return;

  if (attemptAction(event, stack)) return;

  switch (true) {
    case !event.metaKey && event.key === "Enter":
      await stack.triggerCopy();
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

    case (event.metaKey && event.key === "z"):
      event.preventDefault();
      stack.undo();
      return;

    case (event.metaKey && event.key === "t"):
      event.preventDefault();
      stack.touch();
      return;

    case event.metaKey && event.key === "l":
      event.preventDefault();
      console.log("store_win_move");
      invoke("store_win_move", {});
      return;

    case (event.ctrlKey && event.key === "h") || event.key === "ArrowLeft": {
      event.preventDefault();
      stack.selectLeft();
      return;
    }

    case event.ctrlKey && event.key === "l" || event.key === "ArrowRight": {
      event.preventDefault();
      stack.selectRight();
      return;
    }

    case event.metaKey && event.key === "0":
      event.preventDefault();
      stack.reset();
      return;

    case event.metaKey &&
      ((event.ctrlKey && event.key === "n") || event.key === "ArrowDown"):
      event.preventDefault();
      stack.moveDown();
      return;

    case event.metaKey &&
      (event.ctrlKey && event.key === "p" || event.key === "ArrowUp"):
      event.preventDefault();
      stack.moveUp();
      return;

    case !event.metaKey && !event.altKey &&
      ((event.ctrlKey && event.key === "n") || event.key === "ArrowDown"):
      event.preventDefault();
      stack.selectDown();
      return;

    case !event.metaKey && !event.altKey &&
      (event.ctrlKey && event.key === "p" || event.key === "ArrowUp"):
      event.preventDefault();
      stack.selectUp();
      return;

    case !event.metaKey && event.altKey &&
      ((event.ctrlKey && event.key === "n") || event.key === "ArrowDown"):
      event.preventDefault();
      stack.selectDownStack();
      return;

    case !event.metaKey && event.altKey &&
      (event.ctrlKey && event.key === "p" || event.key === "ArrowUp"):
      event.preventDefault();
      stack.selectUpStack();
      return;

    case (event.metaKey && (event.key === "Meta" || event.key === "c")):
      // avoid capturing command-c
      return;

    default:
      if (modes.active.value == mainMode) {
        // fallback to sending the key stroke to the filter input
        const filterInput = document.getElementById("filter-input");
        if (filterInput) filterInput.focus();
      }
  }
}

export function App() {
  const NAV_TIMEOUT = 30 * 1000; // 30 seconds

  let blurTime: number | null = null;

  const onBlurHandler = () => {
    blurTime = Date.now();
  };

  const onFocusHandler = () => {
    if (!stack) return;
    if (blurTime && Date.now() - blurTime > NAV_TIMEOUT) {
      console.log("NAV_TIMEOUT: reset");
      modes.activate(stack, mainMode);
      stack.reset();
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
      {stack
        ? (
          <>
            {!modes.isActive(editorMode) &&
              !modes.isActive(newNoteMode) &&
              !modes.isActive(pipeToCommand) &&
              <Filter stack={stack} />}
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
              {modes.isActive(actionsMode) && <Actions stack={stack} />}
              {modes.showActiveOverlay(stack)}
            </div>
            <StatusBar stack={stack} />
          </>
        )
        : "loading"}
    </main>
  );
}
