# Stacks [![CI](https://github.com/cablehead/stacks/actions/workflows/rust.yml/badge.svg)](https://github.com/cablehead/stacks/actions/workflows/rust.yml) [![Discord](https://img.shields.io/discord/1182364431435436042?logo=discord)](https://discord.com/invite/YNbScHBHrh)

Stacks is a tool to track and manipulate your current context, with pipes and
filters. To be honest, I'm not quite sure what it is, or where it's going, but
let's keep things simple and say it's a clipboard manager.

A humble clipboard manager aspiring to elevate the depth of our conversations—
no less.

![screenshot](./docs/screenshots/screenshot.png)

It's free, as in beer, but mostly as in 'freedom', and it's also open source. If
you're into that sort of thing, give it a spin!

## Download

`.DMG` installers that have been notarized by Apple.

- **Current version**: v0.15.11
- **Last release**: Oct 14, 2024

[![MacOS (Universal)](./docs/assets/MacOS-Universal.svg)](https://stacks.cross.stream/releases/Stacks_0.15.11_universal.dmg)

## Community

- We have a [Discord channel](https://discord.gg/fDEcqjKHpv) where we chat about
  clipboard managers, flashcards, neo-browsers, Tauri, Rust, wasm, tools of
  thought, and generally fun geekery, and
- We're using
  [Github Discussions](https://github.com/cablehead/stacks/discussions) as a
  forum.

## Usage

<table>
  <tr><td>To launch Stacks</td><td><code>&#8963; + Space</code></td></tr>
  <tr><th colspan="2" align="left">Accessibility</th></tr>
  <tr><td>Increase font size</td><td><code>&#8984; + +</code></td></tr>
  <tr></tr>
  <tr><td>Decrease font size</td><td><code>&#8984; + -</code></td></tr>
  <tr><th colspan="2" align="left">Navigation</th></tr>
  <tr><td>Navigate down</td><td><code>&#8595;</code> or <code>&#8963; + n</code></td></tr>
  <tr></tr>
  <tr><td>Navigate up</td><td><code>&#8593;</code> or <code>&#8963; + p</code></td></tr>
  <tr></tr>
  <tr><td>Navigate left</td><td><code>&#8592;</code> or <code>&#8963; + h</code></td></tr>
  <tr></tr>
  <tr><td>Navigate right</td><td><code>&#8594;</code> or <code>&#8963; + l</code></td></tr>
  <tr></tr>
  <tr><td>Navigate to the stack below</td><td><code>&#x2325; + &#8595;</code></td></tr>
  <tr></tr>
  <tr><td>Navigate to the stack above</td><td><code>&#x2325; + &#8593;</code></td></tr>
  <tr></tr>
  <tr><td>Reset nav (clears filter and brings focus to the top)</td><td><code>&#8984; + 0</code></td></tr>
  <tr><th colspan="2" align="left">Item Manipulation</th></tr>
  <tr><td>Move an item down</td><td><code>&#8984; + &#8595;</code></code></td></tr>
  <tr></tr>
  <tr><td>Move an item up</td><td><code>&#8984; + &#8593;</code></td></tr>
  <tr></tr>
  <tr><td>Bring current item and stack to the top</td><td><code>&#8984; + t</code></td></tr>
  <tr><th colspan="2" align="left">Global shortcuts</th></tr>
  <tr><td>New note</td><td><code>&#8984; &#x21E7; + n</code></td></tr>
</table>

## Development

```bash
git clone https://github.com/cablehead/stacks.git
cd stacks
npm install
npm run tauri dev
```

## Built with:

[Rust](https://www.rust-lang.org), [Tauri](https://tauri.app),
[sled](https://github.com/spacejam/sled),
[cacache](https://github.com/zkat/cacache-rs),
[Tantivy](https://github.com/quickwit-oss/tantivy), [Tokio](https://tokio.rs),
[hyper](https://hyper.rs), [Comrak](https://crates.io/crates/comrak),
[syntect](https://github.com/trishume/syntect),
[TypeScript](https://www.typescriptlang.org), [Preact](https://preactjs.com),
[scru128](https://github.com/scru128/rust),
[tracing](https://docs.rs/tracing/latest/tracing/),

🙏💚
