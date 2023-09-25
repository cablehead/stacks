import { signal, effect } from "@preact/signals";

import { invoke } from "@tauri-apps/api/tauri";

let matchMedia = window.matchMedia("(prefers-color-scheme: dark)");

function preferredMode(mql: MediaQueryList): string {
  return mql.matches ? "dark" : "light";
}

const themeMode = signal(preferredMode(matchMedia));

effect(() => {
    console.log("theme", themeMode.value);
    invoke("store_set_theme_mode", {mode: themeMode.value});
});

matchMedia.addEventListener("change", (e: MediaQueryListEvent) => {
  themeMode.value = preferredMode(e.target as MediaQueryList);
  console.log("SYSTEM PREFERRED COLOR SCHEME CHANGED:", themeMode.value);
});

export default themeMode;
