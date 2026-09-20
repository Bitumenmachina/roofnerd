// ── Review pictures, gate C ──────────────────────────────────────────────────
// Review-only. Opens the shipped binary through the front door with the probe
// harness (read, not edited) and saves the work order's shots S1–S9 here.
//
//   xvfb-run -a -s "-screen 0 1440x900x24" node review-shots/capture.mjs
//
// Stamp: the short HEAD hash. The tree may carry UNTRACKED review files
// (review-shots/, .claude/); it must carry no modified tracked file.
import { execFileSync } from 'node:child_process';
import { writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import {
  detachCostPanel, launch, openDemoJob, showTab, showView, until, wait,
} from '../tools/probe/tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '..');
const HERE = resolve(import.meta.dirname);
const modified = execFileSync('git', ['status', '--porcelain'], { cwd: ROOT }).toString()
  .split('\n').filter((l) => l && !l.startsWith('??'));
if (modified.length) { console.error('modified tracked files:\n' + modified.join('\n')); process.exit(2); }
const COMMIT = execFileSync('git', ['rev-parse', '--short', 'HEAD'], { cwd: ROOT }).toString().trim();
const notes = [];
const note = (s) => { notes.push(s); console.log('  ' + s); };

const app = await launch();
const { session } = app;
const shot = async (name) => {
  const path = join(HERE, `${name}-${COMMIT}.png`);
  await writeFile(path, Buffer.from(await session.screenshot(), 'base64'));
  console.log(`  ${path.replace(`${ROOT}/`, '')}`);
};
const run = (fn, ...a) => session.execute(fn, ...a);

try {
  await until(session, () => document.querySelector('#editor')?.children.length > 0, { what: 'the window' });
  await wait(800);
  await shot('S1-start-screen');

  // S2 — the blank-job walk, as far as the front door goes.
  const start = await run(function () {
    return [...document.querySelectorAll('button, [role=button], a')].map((b) => (b.textContent || '').trim()).filter(Boolean);
  });
  note(`S2 start-screen controls: ${JSON.stringify(start)}`);
  await run(function () { const b = document.querySelector('.menu-button'); if (b) b.click(); });
  await wait(500);
  const menu = await run(function () {
    return [...document.querySelectorAll('.menu-list *, .menu *, [role=menu] *')]
      .map((e) => (e.childElementCount === 0 ? (e.textContent || '').trim() : '')).filter(Boolean);
  });
  note(`S2 File menu items: ${JSON.stringify(menu)}`);
  await shot('S2-blank-job-file-menu');
  await run(function () { document.body.click(); });
  await wait(300);
  note('S2 stopped: no control creates a job; "Open a job…" opens the operating-system folder picker, which WebDriver cannot drive.');

  // S3 — the demo, Plan with the cost panel on Estimate.
  await openDemoJob(session);
  await wait(1800);
  await until(session, () => document.querySelector('.surface-overlay')?.getAttribute('viewBox') !== null,
    { what: 'the drawing', timeout: 25000 });
  await wait(1200);
  await showTab(session, 'Estimate');
  await shot('S3-plan-estimate');

  // S4 — Recap.
  await showTab(session, 'Recap');
  await wait(600);
  await shot('S4-recap');

  // S5 — Condition tab, the parapet picked.
  await showTab(session, 'Condition');
  const picked = await run(function () {
    const rows = [...document.querySelectorAll('.cost-condition .condition-list .condition, .condition-list .condition')];
    const p = rows.find((r) => /parapet/i.test(r.textContent || '')) || rows[0];
    if (!p) return null; p.click(); return (p.textContent || '').trim().slice(0, 60);
  });
  note(`S5 condition picked: ${picked}`);
  await wait(800);
  await shot('S5-condition-parapet');

  // S6 — Model at the opening camera.
  await showView(session, 'Model');
  await wait(2200);
  await shot('S6-model-opening');

  // S7 — Section; it opens on the first drain.
  const sec = await showView(session, 'Section');
  await wait(2200);
  note(`S7 Section view offered: ${sec}`);
  await shot('S7-section-drain-1');
  await showView(session, 'Plan');
  await wait(900);

  // S8 — Estimate: a line open, then a struck line scrolled into view.
  await showTab(session, 'Estimate');
  await run(function () { const l = document.querySelector('.cost-line .line-what'); if (l) l.click(); });
  await wait(900);
  await shot('S8a-estimate-line-open');
  const struck = await run(function () {
    const s = document.querySelector('.line-off');
    if (!s) return null; s.scrollIntoView({ block: 'center' }); return (s.textContent || '').trim().slice(0, 80);
  });
  note(`S8 struck line found: ${struck}`);
  await wait(600);
  await shot('S8b-estimate-struck-line');

  // S9 — the cost panel detached; the whole virtual screen.
  await detachCostPanel(session);
  await wait(1500);
  try {
    execFileSync('import', ['-window', 'root', join(HERE, `S9-two-windows-${COMMIT}.png`)]);
    note('S9 whole-screen capture taken with ImageMagick import');
  } catch (e) { note(`S9 whole-screen capture failed: ${e.message}`); }
  await shot('S9-main-after-detach');
  const handles = await session.handles();
  note(`S9 window handles: ${handles.length}`);
  if (handles.length > 1) { await session.switchTo(handles[handles.length - 1]); await wait(1200); await shot('S9-cost-window'); }
} finally {
  await app.close();
  await writeFile(join(HERE, `capture-notes-${COMMIT}.md`), `# Capture notes at ${COMMIT}\n\n` + notes.map((n) => `- ${n}`).join('\n') + '\n');
}
