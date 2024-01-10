export function b64ToUtf8(str: string) {
  return decodeURIComponent(escape(window.atob(str)));
}

export function utf8ToB64(str: string) {
  return window.btoa(unescape(encodeURIComponent(str)));
}

export function truncateUrl(url: string, maxLength: number): string {
  const urlObj = new URL(url);

  const parts = {
    proto: urlObj.protocol + "//",
    hostname: urlObj.hostname,
    path: urlObj.pathname,
    q: urlObj.search,
  };

  let join = () => {
    return parts.proto + parts.hostname + parts.path + parts.q;
  };
  if (join().length <= maxLength) return join();

  parts.hostname = parts.hostname.replace(/^www\./, "");
  if (join().length <= maxLength) return join();

  // parts.proto = "ht:/";
  parts.proto = "";
  if (join().length <= maxLength) return join();

  const trunLength = maxLength - 2;

  let excess = join().length - trunLength;
  if (parts.q.length > excess) {
    parts.q = parts.q.substring(0, parts.q.length - excess) + "..";
    return join();
  }
  parts.q = "";
  if (join().length <= maxLength) return join();

  excess = join().length - trunLength;
  if (parts.path.length > excess) {
    parts.path = "/.." +
      parts.path.substring(excess + 1, parts.path.length);
    return join();
  }
  parts.path = "";

  excess = join().length - trunLength;
  parts.hostname = parts.hostname.substring(0, parts.hostname.length - excess) +
    "..";
  return join();
}

export function dn(): string {
  const date = new Date();
  const options: Intl.DateTimeFormatOptions = {
    weekday: "short",
    year: "numeric",
    month: "short",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: true,
    timeZoneName: "short",
  };
  const formattedDate = new Intl.DateTimeFormat("en-US", options).format(
    date,
  );
  return formattedDate;
}

interface MatchKeyOptions {
  shift?: boolean;
  ctrl?: boolean;
  alt?: boolean;
  meta?: boolean;
  key?: string;
  code?: string;
}

export function matchKeyEvent(
  event: KeyboardEvent,
  options: MatchKeyOptions,
): boolean {
  return (
    (!options.key || event.key === options.key) &&
    (!options.code || event.code === options.code) &&
    event.shiftKey === !!options.shift &&
    event.ctrlKey === !!options.ctrl &&
    event.altKey === !!options.alt &&
    event.metaKey === !!options.meta
  );
}
