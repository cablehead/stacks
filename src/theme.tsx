import { signal } from "@preact/signals";

let matchMedia = window.matchMedia("(prefers-color-scheme: dark)");

function preferredMode(mql: MediaQueryList): string {
  return mql.matches ? "dark" : "light";
}

const themeMode = signal(preferredMode(matchMedia));

matchMedia.addEventListener("change", (e: MediaQueryListEvent) => {
  themeMode.value = preferredMode(e.target as MediaQueryList);
  console.log("SYSTEM PREFERRED COLOR SCHEME CHANGED:", themeMode.value);
});

export default themeMode;
