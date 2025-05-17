# Stacks [![CI](https://github.com/cablehead/stacks/actions/workflows/rust.yml/badge.svg)](https://github.com/cablehead/stacks/actions/workflows/rust.yml) [![Discord](https://img.shields.io/discord/1182364431435436042?logo=discord)](https://discord.com/invite/YNbScHBHrh)

Stacks is a clipboard manager.

![screenshot](./docs/screenshots/screenshot.png)

## About

With so many great clipboard managers already available, why create another one?

I think of my clipboard as "picking things up" to move them around or redirect
them. In this sense, your system's clipboard acts as a strong proxy for your
["locus of attention"](https://www.oreilly.com/library/view/humane-interface-the/0201379376/0201379376_ch02lev1sec3.html)
when you're using a computer.

A clipboard manager, then, is a tool to capture and work with your "locus of
attention." It ambiently captures your current tasks and work _context_.

Stacks is an experimental tool for tracking and manipulating your current
context using pipes and filters. But that’s a lot to explain, so I usually just
describe it as a clipboard manager.

A humble clipboard manager [aspiring](https://x.com/cablelounger/status/1854955656526127398) to elevate the depth of our conversations—
no less.

## UX disclaimer

A quick note on the user experience (UX): it’s fair to say it’s still a bit
rough around the edges. If Stacks reaches a UX level similar to (neo)vim, I'd
consider that a success. It’s pretty spartan and utilitarian, so being
comfortable with the command line, or feeling adventurous, definitely helps.

Stacks is my personal
["tool for thought"](https://maggieappleton.com/tools-for-thought) that I use as
my daily driver. Eventually, I’d like Stacks to reach the polish of tools like
[Obsidian](https://obsidian.md), but for now, the focus is on its experimental
nature and the underlying
[event-sourcing store](https://github.com/cablehead/xs).

## Give it a try!

If you're into experimental tools and are okay with a minimalist,
utilitarian design, give Stacks a try-- I'd love to hear your thoughts!

### Download

`.DMG` installers that have been notarized by Apple.

- **Current version**: v0.15.13
- **Last release**: Jan 23, 2025

[![MacOS (Universal)](./docs/assets/MacOS-Universal.svg)](https://stacks.cross.stream/static/releases/Stacks_0.15.13_universal.dmg)

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
  <tr><td>Move an item down</td><td><code>&#8984; + &#8595;</code></td></tr>
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
