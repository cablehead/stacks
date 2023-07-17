export function b64ToUtf8(str: string) {
  return decodeURIComponent(escape(window.atob(str)));
}
