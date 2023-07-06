import { signal } from "@preact/signals";

function preferredMode(): string {
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

const themeMode = signal(preferredMode());

setInterval(() => {
  themeMode.value = preferredMode();
}, 60 * 60 * 1000); // 1 hour

export default themeMode;
