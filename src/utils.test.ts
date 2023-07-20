import { assertEquals } from "https://deno.land/std@0.106.0/testing/asserts.ts";
import { truncateUrl, b64ToUtf8 } from "./utils.ts";

Deno.test("should b64ToUtf8", () => {
  const result = b64ToUtf8('SGFp');
  assertEquals(result, 'Hai');
});

Deno.test("truncateUrl should replace http[s]:// with ht:/ and strip leading www.", () => {
  const result = truncateUrl('https://www.example.com', 50);
  assertEquals(result, 'ht:/example.com');
});

Deno.test("truncateUrl should truncate the domain name with .. if it exceeds the specified length", () => {
  const result = truncateUrl('https://www.verylongdomainname.com', 10);
  assertEquals(result, 'ht:/verylo..');
});

Deno.test("truncateUrl should include as much of the path as possible, keeping the end and working backwards", () => {
  const result = truncateUrl('https://www.example.com/path/to/resource', 30);
  assertEquals(result, 'ht:/example.com/path/to/resource');
});

Deno.test("truncateUrl should truncate the path with .. if it exceeds the remaining length", () => {
  const result = truncateUrl('https://www.example.com/path/to/resource', 20);
  assertEquals(result, 'ht:/example.com/../to/resource');
});
