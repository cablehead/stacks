import { borderBottom, iconStyle, overlay } from "./app.css.ts";

function ActionRow({ name, keys }: { name: string; keys?: string[] }) {
  return (
    <div
      className={"terserow"}
      style="
        display: flex;
        width: 100%;
        overflow: hidden;
        padding: 0.5ch 0.75ch;
        justify-content: space-between;
        border-radius: 6px;
        cursor: pointer;
        "
    >
      <div>
        {name}
      </div>
      <div>
        {keys
          ? keys.map((key, index) => (
            <span
              className={iconStyle}
              style={index !== keys.length - 1 ? { marginRight: "0.25ch" } : {}}
            >
              {key}
            </span>
          ))
          : ""}
      </div>
    </div>
  );
}

export function Actions() {
  return (
    <div
      className={overlay}
      style={{
        position: "absolute",
        width: "40ch",
        overflow: "auto",
        bottom: "0.25lh",
        fontSize: "0.9rem",
        right: "4ch",
        borderRadius: "0.5rem",
        zIndex: 100,
      }}
    >
      <div
        className={borderBottom}
        style="
        padding:1ch;
        display: flex;
        width: 100%;
        align-items: center;
        "
      >
        <div style="width: 100%">
          <input
            type="text"
            placeholder="Search..."
          />
        </div>
      </div>

      <div style="
        padding:1ch;
        ">
        <ActionRow name={"Delete"} keys={["Ctrl", "DEL"]} />
        <ActionRow name={"Microlink Screenshot"} />
      </div>
    </div>
  );
}
