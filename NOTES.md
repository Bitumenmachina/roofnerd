# Build ledger

What was decided, and what the next gate has to know before it starts. Newest last.

## Gate 0 — skeleton

**Built:** the engine package (vocabulary as types, branded units, job folder read/write,
recap structure), the Tauri shell holding one document behind a lock, two editors that
prove live shared state, the demo job, the two gates.

**Not built, on purpose:** no PDF, no trace, no three.js, no recap arithmetic, no library,
no lenses, no Gantt.

**Decisions**

- **No UI framework.** Vanilla TypeScript and Vite. The code carried in at Gate 1 is plain
  DOM code, and a framework would sit between a roofer-legible codebase and the screen.
- **The document lives in Rust, not in a window.** Every window reads it and subscribes to
  changes. `doc_apply` mutates under the lock, clones, **drops the guard, then emits** —
  the lock is never held while serializing to every window, or one hung window stalls the
  document for the rest.
- **The estimating engine is a standalone package with a command line.** It runs and is
  tested with no window open. The application is one client of it; a script or the Gantt
  bonus can be another.
- **Local-only is enforced by the shell, not by grepping for URLs.** A URL string is not a
  network call. The CSP is what makes the webview unable to reach out; the grep is only a
  detector for the calls themselves.
- **Product name is isolated to three places** — `PRODUCT_NAME` in the engine,
  `productName` in `tauri.conf.json`, the README title. Renaming is three edits.

## Gate 1 — the working loop · read before starting

Two things that will bite, recorded now so nobody reaches for the easy wrong fix.

**1. The CSP will bite the port, and the answer is not `unsafe-inline`.**

The single-file build being carried in keeps every module in an inline `<script>` and its
styling in an inline `<style>`. `default-src 'self'` blocks both. That is the policy doing
its job, and the fix is the port itself: inline scripts become `.ts` files and inline
styles become `.css` files. Loosening the policy to admit them throws away the one
mechanism that makes the local-only claim true.

pdf.js additionally needs a worker and blob URLs. The policy gains exactly this at Gate 1:

    worker-src 'self' blob:
    img-src    'self' data: blob:

and **`connect-src 'self'` stays closed**. Nothing in that list lets the program reach the
network; they let it render a page it already has on disk.

**2. pdf.js is old.** The inlined copy is 3.11.174 (Apache-2.0, Mozilla, 2023). Upstream is
on 5.x. Decide at Gate 1 whether to carry the known-good 3.11 across first and upgrade
after the trace surface works, or upgrade during the port. Carrying it first is the
smaller change and keeps the port's failures attributable.

## This machine — WebKitGTK and the NVIDIA driver

The window died on startup with `Gdk-Message: Error 71 (Protocol error) dispatching to
Wayland display`, and under XWayland it instead logged `Failed to create GBM buffer of size
2200x1600`. Cause: WebKitGTK renders through DMABUF by default and that path is broken
against the proprietary NVIDIA driver.

`WEBKIT_DISABLE_DMABUF_RENDERER=1` fixes it. `src-tauri/src/lib.rs` now sets it at startup
when `/dev/nvidiactl` exists and the variable is not already set, so nothing needs to be
exported by hand here and machines on other GPUs keep the fast path.

Same family as the `QT_QPA_PLATFORM=xcb` wrapper CloudCompare needs on this box. Any future
GPU-adjacent tool is worth checking against both.

## What Gate 0 did not verify

- **The content security policy has not been tested against a release bundle.** Development
  serves the same embedded assets over the same protocol, which is why there is no dev
  server, but `pnpm tauri build` has not been run. Confirm the policy on a bundle before
  claiming it holds in a shipped application.
- **The two-window acceptance is Patrick's to run.** The document layer is tested from both
  sides and the application launches; whether a change in one window shows in the other is
  something a person has to see.

## This machine — the AppImage bundle and `strip`

`pnpm tauri build` produced the `.deb` and the `.rpm` and then failed the AppImage with
`failed to run linuxdeploy`. The real reason is further down its log: linuxdeploy carries a
`strip` from a 2024 build, and it does not understand the `.relr.dyn` sections a current
Fedora toolchain emits, so it errors on every system library it touches.

`NO_STRIP=true` is linuxdeploy's own escape and all three bundles build. `pnpm build:app`
sets it. The only cost is that the bundled libraries keep their symbols, so the AppImage is
larger than it needs to be — worth revisiting when linuxdeploy catches up, and not worth
anything before then.
