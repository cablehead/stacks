import { Icon } from "../ui/icons";
import { borderRight } from "../ui/app.css";

import { Item } from "../types";

import { stack } from "../modals/mainMode";

export function Nav() {
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
