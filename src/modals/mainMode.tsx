import { invoke } from "@tauri-apps/api/tauri";

import { useEffect } from "preact/hooks";
import { useSignal } from "@preact/signals";

import { Icon } from "../ui/icons";

import { Modes } from "./types";

import { default as actionsMode } from "./actionsMode";

import { Stack } from "../types";

import {
  borderRight,
  enchantedForestGradient,
  enchantedForestGradientActive,
} from "../ui/app.css";

const VertDiv = () => (
  <div
    className={borderRight}
    style={{
      width: "1px",
      height: "1.5em",
    }}
  />
);

const Lock = ({ stack }: { stack: Stack }) => {
  return (
    <div
      onMouseDown={() => stack.toggleLock()}
      class="hoverable"
    >
      <span style="
            display: inline-block;
            width: 1.5em;
            height: 1.5em;
            text-align: center;
            border-radius: 5px;
            ">
        {stack.isLocked()
          ? <Icon name="IconLockClosed" />
          : <Icon name="IconLockOpen" />}
      </span>
    </div>
  );
};

const SortOrder = ({ stack }: { stack: Stack }) => {
  const currStack = stack.nav.value.root?.selected;
  if (!currStack) return <span></span>;
  return (
    <div
      onMouseDown={() => {
        const command = currStack.ordered
          ? "store_stack_sort_auto"
          : "store_stack_sort_manual";
        invoke(command, { sourceId: currStack.id });
      }}
      class="hoverable"
    >
      <span style="
            display: inline-block;
            width: 1.5em;
            height: 1.5em;
            text-align: center;
            border-radius: 5px;
            ">
        {currStack.ordered
          ? <Icon name="IconStack" />
          : <Icon name="IconStackSorted" />}
      </span>
    </div>
  );
};

const Broadcast = ({ stack }: { stack: Stack }) => {
  const currStack = stack.nav.value.root?.selected;
  if (!currStack) return <span></span>;

  const tokenLooksGood = useSignal(false);

  useEffect(() => {
    (async () => {
      const settings = await invoke<Record<string, string>>(
        "store_settings_get",
        {},
      );
      if (
        settings && settings.cross_stream_access_token &&
        settings.cross_stream_access_token.length === 64
      ) {
        tokenLooksGood.value = true;
      }
    })();
  }, []);

  if (!tokenLooksGood.value) return <span></span>;

  const active = currStack.cross_stream;

  return (
    <>
      <div
        onMouseDown={() => {
          invoke("store_mark_as_cross_stream", { stackId: currStack.id });
        }}
        className={active
          ? enchantedForestGradientActive
          : enchantedForestGradient}
      >
        <span style="
            display: inline-block;
            width: 1.5em;
            height: 1.5em;
            text-align: center;
            border-radius: 5px;
            ">
          {active ? <Icon name="IconBolt" /> : <Icon name="IconBoltSlash" />}
        </span>
      </div>
      <VertDiv />
    </>
  );
};

export default {
  name: (stack: Stack) => {
    const selected = stack.nav.value.root?.selected;
    if (!selected) return <span></span>;
    const name = selected ? selected.name : "";
    return (
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: "0.5ch",
          marginLeft: "-1ch",
        }}
      >
        <Lock stack={stack} />
        <VertDiv />
        <SortOrder stack={stack} />
        <VertDiv />
        <Broadcast stack={stack} />
        <div>
          {name}
        </div>
      </div>
    );
  },
  hotKeys: (stack: Stack, modes: Modes) => {
    let ret = [];

    ret.push({
      name: "Copy clip",
      keys: [<Icon name="IconReturnKey" />],
      onMouseDown: () => {
        stack.triggerCopy();
      },
    });

    ret.push({
      name: "Actions",
      keys: [<Icon name="IconCommandKey" />, "K"],
      onMouseDown: () => {
        modes.toggle(stack, actionsMode);
      },
    });

    if (stack.filter.dirty()) {
      ret.push({
        name: "Clear filter",
        keys: ["ESC"],
        onMouseDown: () => {
          stack.filter.clear();
        },
      });
    }

    return ret;
  },
};
