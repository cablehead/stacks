import { render } from "preact";
import { signal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

const { listen } = require("@tauri-apps/api/event");
const { invoke } = require("@tauri-apps/api/tauri");
const { writeText } = require("@tauri-apps/api/clipboard");

const { hide } = require("tauri-plugin-spotlight-api");

import { Scru128Id, scru128String } from "scru128";

function scru128ToDate(id) {
  const scruId = Scru128Id.fromString(id);
  const timestampMillis = scruId.timestamp;
  const date = new Date(timestampMillis);
  return date;
}

function cmp(a, b) {
  if (a < b) {
    return -1;
  } else if (a > b) {
    return 1;
  } else {
    return 0;
  }
}

const selected = signal(0);
const items = signal([]);

// https://heroicons.com
const IconClipboard = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    fill="none"
    viewBox="0 0 24 24"
    strokeWidth={1.5}
    stroke="currentColor"
    className="w-6 h-6"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      d="M9 12h3.75M9 15h3.75M9 18h3.75m3 .75H18a2.25 2.25 0 002.25-2.25V6.108c0-1.135-.845-2.098-1.976-2.192a48.424 48.424 0 00-1.123-.08m-5.801 0c-.065.21-.1.433-.1.664 0 .414.336.75.75.75h4.5a.75.75 0 00.75-.75 2.25 2.25 0 00-.1-.664m-5.8 0A2.251 2.251 0 0113.5 2.25H15c1.012 0 1.867.668 2.15 1.586m-5.8 0c-.376.023-.75.05-1.124.08C9.095 4.01 8.25 4.973 8.25 6.108V8.25m0 0H4.875c-.621 0-1.125.504-1.125 1.125v11.25c0 .621.504 1.125 1.125 1.125h9.75c.621 0 1.125-.504 1.125-1.125V9.375c0-.621-.504-1.125-1.125-1.125H8.25zM6.75 12h.008v.008H6.75V12zm0 3h.008v.008H6.75V15zm0 3h.008v.008H6.75V18z"
    />
  </svg>
);

const IconCommandLine = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    fill="none"
    viewBox="0 0 24 24"
    strokeWidth={1.5}
    stroke="currentColor"
    className="w-6 h-6"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      d="M6.75 7.5l3 2.25-3 2.25m4.5 0h3m-9 8.25h13.5A2.25 2.25 0 0021 18V6a2.25 2.25 0 00-2.25-2.25H5.25A2.25 2.25 0 003 6v12a2.25 2.25 0 002.25 2.25z"
    />
  </svg>
);

const IconBell = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    fill="none"
    viewBox="0 0 24 24"
    strokeWidth={1.5}
    stroke="currentColor"
    className="w-6 h-6"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      d="M14.857 17.082a23.848 23.848 0 005.454-1.31A8.967 8.967 0 0118 9.75v-.7V9A6 6 0 006 9v.75a8.967 8.967 0 01-2.312 6.022c1.733.64 3.56 1.085 5.455 1.31m5.714 0a24.255 24.255 0 01-5.714 0m5.714 0a3 3 0 11-5.714 0"
    />
  </svg>
);

const IconImage = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    fill="none"
    viewBox="0 0 24 24"
    strokeWidth={1.5}
    stroke="currentColor"
    className="w-6 h-6"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      d="M2.25 15.75l5.159-5.159a2.25 2.25 0 013.182 0l5.159 5.159m-1.5-1.5l1.409-1.409a2.25 2.25 0 013.182 0l2.909 2.909m-18 3.75h16.5a1.5 1.5 0 001.5-1.5V6a1.5 1.5 0 00-1.5-1.5H3.75A1.5 1.5 0 002.25 6v12a1.5 1.5 0 001.5 1.5zm10.5-11.25h.008v.008h-.008V8.25zm.375 0a.375.375 0 11-.75 0 .375.375 0 01.75 0z"
    />
  </svg>
);

function parseItem(raw) {
  let item = JSON.parse(raw);

  item.created_at = scru128ToDate(item.id)
    .toLocaleString(
      "en-US",
      {
        weekday: "short",
        year: "numeric",
        month: "short",
        day: "numeric",
        hour: "numeric",
        minute: "numeric",
        hour12: true,
      },
    );

  item.meta = [
    { name: "ID", value: item.id },
    { name: "Created", value: item.created_at },
    { name: "Topic", value: item.topic },
  ];

  switch (item.topic) {
    case "command":
      item.o = JSON.parse(item.data);
      item.icon = <IconCommandLine />;
      item.terse = item.o.command;
      item.preview = item.o.output.stdout;
      break;

    case "clipboard":
      let data = JSON.parse(item.data);
      if ("public.utf8-plain-text" in data.types) {
        item.icon = <IconClipboard />;
        item.terse = atob(data.types["public.utf8-plain-text"]);
        item.preview = item.terse;
        break;
      }
      item.icon = <IconImage />;
      item.terse = data.source;
      item.preview = item.data;
      break;

    default:
      item.icon = <IconBell />;
      item.terse = item.data;
      item.preview = item.data;
  }
  return item;
}

function LeftPane() {
  const TerseRow = ({ item, index }) => (
    <div
      className={"terserow" + (index === selected.value ? " selected" : "")}
      onClick={() => selected.value = index}
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
        {item.icon}
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
    <div style="
      flex: 1;
      max-width: 20ch;
      overflow-y: auto;
      border-right: 1px solid #aaa;
      padding-right: 0.5rem;
    ">
      {items.value
        .map((item, index) => {
          return <TerseRow item={item} index={index} />;
        })}
    </div>
  );
}

function RightPane({ item }) {
  if (!item) {
    return <div />;
  }

  const MetaInfoRow = ({ name, value }) => (
    <div style="display:flex;">
      <div
        style={{
          flexShrink: 0,
          width: "20ch",
        }}
      >
        {name}
      </div>
      <div>{value}</div>
    </div>
  );

  return (
    <div style=" flex: 3; overflow: auto; display: flex; flex-direction: column;">
      <div style="padding-bottom: 0.5rem; border-bottom: 1px solid #aaa; flex:2; overflow-y: auto;">
        <pre style="margin: 0;">
        {item.preview}
        </pre>
      </div>
      <div style="height: 5lh; ;  overflow-y: auto;">
        {item.meta.map((info) => (
          <MetaInfoRow name={info.name} value={info.value} />
        ))}
      </div>
    </div>
  );
}

function ListView() {
  const mainRef = useRef(null);

  useEffect(() => {
    if (mainRef.current) {
      function updateSelected(n) {
        selected.value = (selected.value + n) % items.value.length;
        if (selected.value < 0) {
          selected.value = items.value.length + selected.value;
        }

        setTimeout(() => {
          // Scroll the selected item into view
          const selectedItem = mainRef.current.querySelector(
            `.terserow:nth-child(${selected.value + 1})`,
          );
          selectedItem.scrollIntoView({
            behavior: "smooth",
            block: "nearest",
          });
        }, 0);
      }

      async function handleKeys(event) {
        switch (true) {
          case event.key === "Enter":
            const item = items.value[selected.value];
            if (item) {
              await writeText(item.preview);
              hide();
            }
            break;

          /*
            case event.key === "Enter":
              if (inputElement.value.trim() !== "") {
                await invoke("run_command", {
                  command: inputElement.value,
                });
                inputElement.value = "";
              }
              break;
                  */

          case (event.ctrlKey && event.key === "n") ||
            event.key === "ArrowDown":
            updateSelected(1);
            break;

          case event.ctrlKey && event.key === "p" || event.key === "ArrowUp":
            updateSelected(-1);
            break;
        }
      }
      window.addEventListener("keydown", handleKeys);

      return () => {
        window.removeEventListener("keydown", handleKeys);
      };
    }
  }, []);

  return (
    <main ref={mainRef}>
      <div style=" display: flex; height: 100%; overflow: hidden; gap: 0.5ch;">
        <LeftPane />
        <RightPane item={items.value[selected.value]} />
      </div>
    </main>
  );
}

function App() {
  useEffect(() => {
    function handleDataFromRust(event) {
      console.log("Data pushed from Rust:", event);
      items.value = [
        parseItem(event.payload.message),
        ...items.value,
      ];
      if (selected.value > 0) selected.value += 1;
    }

    async function fetchData() {
      try {
        let data = await invoke("init_process");
        data = data.map(parseItem).reverse();
        items.value = data;
      } catch (error) {
        console.error("Error in init_process:", error);
      }
      listen("item", handleDataFromRust);
    }
    fetchData();

    // set selection back to the top onBlur
    const onBlur = () => {
      selected.value = 0;
    };

    window.addEventListener("blur", onBlur);

    // Return a cleanup function
    return () => {
      window.removeEventListener("blur", onBlur);
    };
  }, []);

  return <ListView />;
}

render(<App />, document.querySelector("body"));
