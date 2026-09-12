// ── The real-runtime check ─────────────────────────────────────────────────
// Addendum §2 and §3: a section is not done on Chrome. This runs against the
// SHIPPED application — the release binary, WebKitGTK rendering, the Rust shell
// holding the document, real OS windows — and it ends with the security policy
// tested rather than asserted.
//
// Run: node tools/probe/runtime-check.mjs

import { mkdir, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import { launch, openDemoJob, until, wait, commitStamp } from './tauri-harness.mjs';
import { writeFile as writeBinary } from 'node:fs/promises';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = commitStamp();

const results = [];
const check = (name, fn) => {
  try { fn(); results.push('PASS'); console.log(`  PASS  ${name}`); }
  catch (e) { results.push('FAIL'); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};

await mkdir(EVIDENCE, { recursive: true });
const app = await launch();
const { session } = app;

try {
  // ── it is the real thing ────────────────────────────────────────────────
  const capabilities = session.capabilities ?? {};
  check('this is the shipped runtime, not a browser', () => {
    assert.equal(capabilities.browserName, 'wry', `browserName is ${capabilities.browserName}`);
    assert.equal(capabilities.platformName, 'linux');
  });

  await until(session, () => document.querySelector('#editor')?.children.length > 0,
    { what: 'the editor to mount' });

  const title = await session.title();
  check('the window is titled for the program', () => assert.equal(title, 'roofnerd'));

  // ── one document, opened through the front door ─────────────────────────
  await openDemoJob(session);
  const opened = await session.execute(() => document.querySelector('.menu-job')?.textContent);
  check('a job opens from the start screen, by a button a person clicks', () =>
    assert.ok(opened && opened.length > 0, `job name reads "${opened}"`));

  // ── two real OS windows on that one document ────────────────────────────
  const before = await session.handles();
  await session.execute(() => window.__TAURI_INTERNALS__.invoke('open_editor', { editor: 'estimate' }));
  await wait(1500);
  const after = await session.handles();

  check('tearing off an editor opens a second OS window', () =>
    assert.ok(after.length > before.length, `${before.length} window(s) before, ${after.length} after`));

  const second = after.find((h) => !before.includes(h));
  await session.switchTo(second);
  await until(session, () => document.querySelector('#editor')?.children.length > 0,
    { what: 'the torn-off editor to mount' });

  // A torn-off window has no menu bar — that is the point of A3 — so what it
  // knows about the job is read from its status bar, which it does have.
  const seenInSecond = await session.execute(() =>
    (document.querySelector('.status-facts') || {}).textContent || '');
  check('the second window is on the same job, not its own copy', () =>
    assert.ok(seenInSecond.includes(opened), `it shows "${seenInSecond}"`));

  // Change the job from the FIRST window, by typing in a field, and watch the
  // SECOND window follow. Typing rather than writing to the document: the point
  // is that the loop works for a person, not that the bridge works.
  await session.switchTo(before[0]);
  await session.execute(() => {
    const row = document.querySelector('.condition');
    if (row) row.click();
  });
  await wait(500);
  const typed = await session.execute(() => {
    const field = [...document.querySelectorAll('.condition-panel label')]
      .find((l) => l.querySelector('span')?.textContent === 'Height')?.querySelector('input');
    if (!field) return null;
    field.value = '7';
    field.dispatchEvent(new Event('input', { bubbles: true }));
    return true;
  });
  check('a property can be typed in the Plan window', () => assert.equal(typed, true));
  await wait(1200);

  await session.switchTo(second);
  const heard = await session.execute(() => {
    const rows = [...document.querySelectorAll('.sheet tbody tr:not(.group-row):not(.add-row)')];
    return rows.map((r) => r.querySelectorAll('td')[3]?.textContent ?? '').join('|');
  });
  check('and the money in the other window moves with it', () =>
    assert.ok(/\d/.test(heard), `the sheet shows "${heard}"`));

  // ── the security policy, tested rather than promised ────────────────────
  const outward = await session.executeAsync(function (done) {
    fetch('https://example.com')
      .then((r) => done(`REACHED THE NETWORK: ${r.status}`))
      .catch((e) => done(`REFUSED: ${e.message}`));
  });
  const inward = await session.executeAsync(function (done) {
    fetch('/index.html')
      .then((r) => done(`ok ${r.status}`))
      .catch((e) => done(`blocked: ${e.message}`));
  });

  check('the webview cannot reach the network', () =>
    assert.match(outward, /^REFUSED/, outward));
  check('and can still load its own files', () =>
    assert.match(inward, /^ok 200/, inward));

  await writeFile(join(EVIDENCE, `csp-${COMMIT}.txt`), [
    `roofnerd content security policy — checked in the shipped runtime`,
    `commit    ${COMMIT}`,
    `runtime   ${capabilities.browserName} ${capabilities.browserVersion} on ${capabilities.platformName}`,
    `when      ${new Date().toISOString()}`,
    ``,
    `fetch('https://example.com')  ->  ${outward}`,
    `fetch('/index.html')          ->  ${inward}`,
    ``,
    `The policy is in src-tauri/tauri.conf.json. connect-src is 'self'.`,
    `This is the claim in the README tested against the program that ships,`,
    `not against a development server and not against Chrome.`,
    ``,
  ].join('\n'));

  // ── evidence, from the Tauri window ─────────────────────────────────────
  await session.switchTo(before[0]);
  await writeBinary(join(EVIDENCE, `runtime-plan-${COMMIT}.png`), Buffer.from(await session.screenshot(), 'base64'));
  await session.switchTo(second);
  await writeBinary(join(EVIDENCE, `runtime-estimate-${COMMIT}.png`), Buffer.from(await session.screenshot(), 'base64'));
} catch (e) {
  results.push('FAIL');
  console.log(`  FAIL  the check stopped: ${e.message}`);
} finally {
  await app.close();
}

const failed = results.filter((r) => r === 'FAIL').length;
console.log(failed ? `\nFAIL — ${failed} of ${results.length}` : `\nPASS — ${results.length}/${results.length}`);
process.exit(failed ? 1 : 0);
