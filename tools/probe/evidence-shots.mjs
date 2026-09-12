// ── Evidence from the shipped window ───────────────────────────────────────
// Screenshots for a section's done-check come from the Tauri window, never from
// a browser, and never from a scene dressed for the camera.
//
//   node tools/probe/evidence-shots.mjs <label>
//
// Writes evidence/<area>-<label>-<commit>.png
//
// This used to build a scratch job in a temp directory and hand it straight to
// `doc_open`, with a comment saying it "dresses a scene to photograph rather
// than checking a path". That was the wrong trade and it cost more than it
// saved: every image in `evidence/` was of a set rather than of the program, so
// the pictures three sections were certified against were not pictures of what
// anyone would see. They also showed a poorer job than the real one — three
// conditions where the demo has five, no library, no thicknesses.
//
// So it opens the demo job through the front door, the way a person does, and
// photographs what is actually there. The scratch job is gone; there is nothing
// it did that the shipped demo does not do better.

import { execFileSync } from 'node:child_process';
import { mkdir, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { launch, openDemoJob, until, wait } from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = execFileSync('git', ['rev-parse', '--short', 'HEAD'], { cwd: ROOT }).toString().trim();
const label = process.argv[2] ?? 'shot';

await mkdir(EVIDENCE, { recursive: true });
const app = await launch();
const { session } = app;

const shot = async (name) => {
  const path = join(EVIDENCE, `${name}-${label}-${COMMIT}.png`);
  await writeFile(path, Buffer.from(await session.screenshot(), 'base64'));
  console.log(`  ${path.replace(`${ROOT}/`, '')}`);
};

try {
  await until(session, () => document.querySelector('#editor')?.children.length > 0,
    { what: 'the window' });

  // The start screen, before anything is opened. It had never been photographed
  // once, and it is the first thing anybody sees.
  await wait(600);
  await shot('start-screen');

  await openDemoJob(session);
  await wait(1800);
  await until(session, () => document.querySelector('.surface-overlay')?.getAttribute('viewBox') !== null,
    { what: 'the drawing', timeout: 25000 });
  await wait(1200);
  await shot('plan');

  // The File menu, open. Also never photographed — a menu list is closed in
  // every shot that exists.
  await session.execute(function () {
    const button = document.querySelector('.menu-button');
    if (button) button.click();
  });
  await wait(500);
  await shot('menu');
  await session.execute(function () { document.body.click(); });
  await wait(400);

  // The condition panel, with a condition selected.
  await session.execute(function () {
    const first = document.querySelector('.condition');
    if (first) first.click();
  });
  await wait(700);
  await shot('condition-panel');

  // A help sheet, open. The third screen with no picture: nothing in the suite
  // has ever clicked a "?".
  await session.execute(function () {
    const help = document.querySelector('[aria-label="What this is for"]');
    if (help) help.click();
  });
  await wait(600);
  await shot('help-sheet');
  await session.execute(function () {
    const close = document.querySelector('.help-close');
    if (close) close.click();
  });
  await wait(400);

  // The estimate sheet, in the same window rather than a torn-off one — that is
  // where an estimator reads it, and it is the width the money has to survive.
  await session.execute(function () {
    const picker = document.querySelector('.editor-picker');
    if (picker) {
      picker.value = 'estimate';
      picker.dispatchEvent(new Event('change', { bubbles: true }));
    }
  });
  await wait(2200);
  await shot('estimate-sheet');
} finally {
  await app.close();
}
