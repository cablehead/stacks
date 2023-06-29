import { Signal, useSignal } from "@preact/signals";

import { Item, Stack } from "./types";

export const createStack = (items: Signal<Item[]>) : Stack => {
    const selected = useSignal(0);
    return {
        items,
        selected,
    };
}
