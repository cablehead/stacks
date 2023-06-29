import { useEffect } from "preact/hooks";
import { useSignal } from "@preact/signals";

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

import { Item } from "./types";
import {
  focusSelected,
  getContent,
  loadedItem,
  selectedContent,
  selectedItem,
  stack,
  themeMode,
  triggerCopy,
  updateSelected,
} from "./modals/mainMode";

function RightPane(
  { item, content }: {
    item: Item | undefined;
    content: string | undefined;
  },
) {
  if (!item) {
    return <div />;
  }

  function SubItem({ item }: {
    item: Item;
  }) {
    const content = useSignal("");
    useEffect(() => {
      const fetchData = async () => {
        const result = await getContent(item.hash);
        content.value = result;
      };
      fetchData();
    });
    return <div>{content}</div>;
  }

  function Preview(
    { item, content }: { item: Item; content: string },
  ) {
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
      return (
        <div>
          <h1>{content}</h1>
          {item.stack.map((item) => <SubItem item={item} />)}
        </div>
      );
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
    { content !== undefined ? content : "loading..." }
      </pre>
    );
  }

  return (
    <div style="flex: 3; overflow: auto; height: 100%">
      {content ? <Preview item={item} content={content} /> : "loading..."}
    </div>
  );
}

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
      updateSelected(1);
      break;

    case event.ctrlKey && event.key === "p" || event.key === "ArrowUp":
      event.preventDefault();
      updateSelected(-1);
      break;

    case (event.metaKey && (event.key === "Meta" || event.key === "c")):
      // avoid capturing command-c
      return;

    default:
      if (loadedItem.value) {
        if (attemptAction(event, loadedItem.value)) return;
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
        <div style="display: flex; height: 100%; overflow: hidden; gap: 0.5ch;">
          <Nav />
          <RightPane
            item={selectedItem.value}
            content={selectedContent.value}
          />
        </div>

        {selectedItem.value && selectedContent.value &&
          (
            <MetaPanel
              item={selectedItem.value}
              content={selectedContent.value}
            />
          )}

        {modes.isActive(addToStackMode) &&
          <addToStackMode.Modal modes={modes} />}

        {loadedItem.value && modes.isActive(actionsMode) &&
          <Actions loaded={loadedItem.value} />}

        {selectedContent.value && modes.isActive(editorMode) &&
          <Editor content={selectedContent.value} />}
      </div>
      <StatusBar />
    </main>
  );
}

export function App() {
  useEffect(() => {
    listen("recent-items", (event: Event<Item[]>) => {
      console.log("Data pushed from Rust:", event);
      stack.items.value = event.payload;
      updateSelected(0);
    });

    async function init() {
      stack.items.value = await invoke<Item[]>("init_window");
      updateSelected(0);
    }
    init();

    // set selection back to the top onBlur
    const onBlur = () => {
      stack.selected.value = 0;
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
