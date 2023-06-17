import { useEffect } from "preact/hooks";

import { Event, listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";
import { writeText } from "@tauri-apps/api/clipboard";
import { hide } from "tauri-plugin-spotlight-api";

import { Icon } from "./ui/icons";
import {
  borderRight,
  darkThemeClass,
  lightThemeClass,
} from "./ui/app.css";

import { StatusBar } from "./panels/statusbar";
import { MetaPanel } from "./panels/meta";
import { Actions, attemptAction } from "./panels/actions";
import { Editor } from "./panels/editor";
import { Filter } from "./panels/filter";

import {
  actions,
  editor,
  filter,
  focusSelected,
  getContent,
  Item,
  selectedContent,
  selectedItem,
  stack,
  themeMode,
  updateSelected,
} from "./state";

function LeftPane() {
  const RowIcon = ({ item }: { item: Item }) => {
    switch (item.content_type) {
      case "Image":
        return <Icon name="IconImage" />;

      case "Link":
        return <Icon name="IconLink" />;

      case "Text":
        return <Icon name="IconClipboard" />;
    }

    return <Icon name="IconBell" />;
  };

  const TerseRow = ({ item, index }: { item: Item; index: number }) => (
    <div
      className={"terserow" +
        (index === stack.selected.value ? " selected" : "")}
      onClick={() => stack.selected.value = index}
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
  );
  return (
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
          return <TerseRow item={item} index={index} />;
        })}
    </div>
  );
}

function RightPane(
  { item, content }: {
    item: Item | undefined;
    content: string | undefined;
  },
) {
  if (!item) {
    return <div />;
  }

  function Preview(
    { item, content }: { item: Item; content: string | undefined },
  ) {
    if (content !== undefined && item.mime_type === "image/png") {
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
      <Preview item={item} content={content} />
    </div>
  );
}

async function triggerCopy() {
  const item = selectedItem.value;
  if (item) {
    if (item.mime_type != "text/plain") {
      console.log("MIEM", item.mime_type);
    } else {
      let content = await getContent(item.hash);
      await writeText(content);
    }
  }
  filter.show.value = false;
  hide();
}

async function globalKeyHandler(event: KeyboardEvent) {
  switch (true) {
    case event.key === "Enter":
      await triggerCopy();
      break;

    case event.key === "Escape":
      event.preventDefault();

      if (actions.show.value) {
        actions.show.value = false;
        return;
      }

      if (filter.show.value) {
        filter.show.value = false;
        return;
      }
      hide();
      return;

    case event.metaKey && event.key === "k":
      event.preventDefault();
      actions.show.value = !actions.show.value;
      // await invoke("open_docs");
      break;

    case event.key === "Tab":
      event.preventDefault();
      // await invoke("open_docs");
      break;

    case ((!filter.show.value) && event.key === "/"):
      event.preventDefault();
      filter.show.value = true;
      break;

    case (filter.show.value && event.metaKey && event.key === "p"):
      event.preventDefault();
      filter.showContentType.value = !filter.showContentType.value;
      break;

    case (event.ctrlKey && event.key === "n") || event.key === "ArrowDown":
      event.preventDefault();
      updateSelected(1);
      break;

    case event.ctrlKey && event.key === "p" || event.key === "ArrowUp":
      event.preventDefault();
      updateSelected(-1);
      break;

    default:
      if (selectedItem.value) attemptAction(event, selectedItem.value);
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
      {filter.show.value && <Filter />}
      <section style="
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
          <LeftPane />
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

        {selectedItem.value && actions.show.value &&
          <Actions showActions={actions.show} item={selectedItem.value} />}

        {selectedItem.value && editor.show.value &&
          <Editor item={selectedItem.value} />}
      </section>
      <StatusBar
        themeMode={themeMode}
        showFilter={filter.show}
        showActions={actions.show}
        triggerCopy={triggerCopy}
      />
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
      actions.show.value = false;
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
