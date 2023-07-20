export function b64ToUtf8(str: string) {
  return decodeURIComponent(escape(window.atob(str)));
}

export function truncateUrl(urlString: string, length: number): string {
  if (typeof urlString !== "string") {
    throw new TypeError("Expected input to be a string");
  }

  if (typeof length !== "number") {
    throw new TypeError("Expected length to be a number");
  }

  let cleanedUrl = urlString.replace(/(http[s]?:\/\/)?(www\.)?/, "ht:/");

  if (cleanedUrl.length <= length) {
    return cleanedUrl;
  }

  let [domain, ...pathParts] = cleanedUrl.split("/");

  if (domain.length > length) {
    return domain.slice(0, length - 2) + "..";
  }

  let truncatedUrl = domain;
  let remainingLength = length - domain.length;

  let path = "";
  for (let i = pathParts.length - 1; i >= 0; i--) {
    let part = "/" + pathParts[i];
    if (part.length + 2 > remainingLength) {
      path = "/.." + path;
      break;
    }

    path = part + path;
    remainingLength -= part.length;
  }

  truncatedUrl += path;

  return truncatedUrl;
}
