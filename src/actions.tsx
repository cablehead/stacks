import { borderBottom, iconStyle, overlay, terserow } from "./app.css.ts";

export function Actions() {
  return (
    <div
      className={overlay}
      style={{
        position: "absolute",
        width: "30ch",
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
            Delete
          </div>
          <div>
            <span className={iconStyle} style="margin-right: 0.25ch;">
              Ctrl
            </span>
            <span className={iconStyle}>
              DEL
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
