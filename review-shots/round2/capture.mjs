// ── Round-2 captures, through the front door ────────────────────────────────
//   xvfb-run -a -s "-screen 0 1440x900x24" node review-shots/round2/capture.mjs <item> [shot ...]
// Shots: menu, start, plan-estimate, recap, condition-parapet, model, section, estimate-parapet, line-open
import { execFileSync } from 'node:child_process';
import { writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { launch, openDemoJob, showTab, showView, until, wait } from '../../tools/probe/tauri-harness.mjs';
const ROOT = resolve(import.meta.dirname, '../..');
const HERE = resolve(import.meta.dirname);
const modified = execFileSync('git', ['status', '--porcelain'], { cwd: ROOT }).toString().split('\n').filter((l) => l && !l.startsWith('??'));
if (modified.length) { console.error('modified tracked files:\n' + modified.join('\n')); process.exit(2); }
const COMMIT = execFileSync('git', ['rev-parse', '--short', 'HEAD'], { cwd: ROOT }).toString().trim();
const [item = 'x', ...want] = process.argv.slice(2);
const wants = (n) => want.length === 0 || want.includes(n);
const app = await launch(); const { session } = app;
const run = (fn, ...a) => session.execute(fn, ...a);
const shot = async (name) => {
  const path = join(HERE, `${item}-${name}-${COMMIT}.png`);
  await writeFile(path, Buffer.from(await session.screenshot(), 'base64'));
  console.log(`  ${path.replace(`${ROOT}/`, '')}`);
};
try {
  await until(session, () => document.querySelector('#editor')?.children.length > 0, { what: 'the window' });
  await wait(800);
  if (wants('start')) await shot('start');
  if (wants('menu')) {
    await run(function () { document.querySelector('.menu .menu-button')?.click(); });
    await wait(500); await shot('menu');
    await run(function () { document.body.click(); }); await wait(300);
  }
  const rest = ['plan-estimate', 'recap', 'condition-parapet', 'model', 'section', 'estimate-parapet', 'line-open'];
  if (rest.some(wants)) {
  await openDemoJob(session); await wait(1800);
  await until(session, () => document.querySelector('.surface-overlay')?.getAttribute('viewBox') !== null, { what: 'the drawing', timeout: 25000 });
  await wait(1200);
  if (wants('plan-estimate')) { await showTab(session, 'Estimate'); await shot('plan-estimate'); }
  if (wants('recap')) { await showTab(session, 'Recap'); await wait(600); await shot('recap'); }
  if (wants('condition-parapet')) {
    await showTab(session, 'Condition');
    await run(function () { const rows = [...document.querySelectorAll('.condition-list .condition')]; (rows.find((r) => /parapet/i.test(r.textContent || '')) || rows[0])?.click(); });
    await wait(800); await shot('condition-parapet');
  }
  if (wants('model')) { await showView(session, 'Model'); await wait(2200); await shot('model'); }
  if (wants('section')) { await showView(session, 'Section'); await wait(2200); await shot('section'); await showView(session, 'Plan'); await wait(800); }
  if (wants('estimate-parapet')) {
    await showTab(session, 'Estimate');
    await run(function () { const s = document.querySelector('.line-off'); if (s) s.scrollIntoView({ block: 'center' }); });
    await wait(600); await shot('estimate-parapet');
  }
  if (wants('line-open')) {
    await showTab(session, 'Estimate');
    await run(function () { document.querySelector('.cost-line .line-what')?.scrollIntoView({ block: 'start' }); document.querySelector('.cost-line .line-what')?.click(); });
    await wait(900); await shot('line-open');
  }
  }
} finally { await app.close(); }
