import { assertEquals } from "https://deno.land/std@0.106.0/testing/asserts.ts";
import { b64ToUtf8, truncateUrl } from "./utils.ts";

Deno.test("should b64ToUtf8", () => {
  const result = b64ToUtf8("SGFp");
  assertEquals(result, "Hai");
});

Deno.test("should truncateUrl", () => {
  const url = "https://www.example.com/a/long/path?q=important";
  assertEquals( truncateUrl(url, 50), "https://www.example.com/a/long/path?q=important",);
  assertEquals( truncateUrl(url, 46), "https://example.com/a/long/path?q=important");
  assertEquals( truncateUrl(url, 35), "example.com/a/long/path?q=important");
  assertEquals( truncateUrl(url, 33), "example.com/a/long/path?q=impor..");
  assertEquals( truncateUrl(url, 27), "example.com/a/long/path?q..");
  assertEquals( truncateUrl(url, 25), "example.com/a/long/path");
  assertEquals( truncateUrl(url, 20), "example.com/..g/path");
  assertEquals( truncateUrl(url, 15), "example.com/..h");
  assertEquals( truncateUrl(url, 14), "example.com/..");
  assertEquals( truncateUrl(url, 13), "example.com..");
  assertEquals( truncateUrl(url, 8), "exampl..");
});
