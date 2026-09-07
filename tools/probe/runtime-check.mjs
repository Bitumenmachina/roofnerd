// ── The real-runtime check ─────────────────────────────────────────────────
// Addendum §2 and §3: a section is not done on Chrome. This runs against the
// SHIPPED application — the release binary, WebKitGTK rendering, the Rust shell
// holding the document, real OS windows — and it ends with the security policy
// tested rather than asserted.
//
// Run: node tools/probe/runtime-check.mjs

import { execFileSync } from 'node:child_process';
import { mkdir, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import { launch, scratchJob, until, wait } from './tauri-harness.mjs';
import { writeFile as writeBinary } from 'node:fs/promises';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = execFileSync('git', ['rev-parse', '--short', 'HEAD'], { cwd: ROOT }).toString().trim();

const results = [];
const check = (name, fn) => {
  try { fn(); results.push('PASS'); console.log(`  PASS  ${name}`); }
  catch (e) { results.push('FAIL'); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};

await mkdir(EVIDENCE, { recursive: true });
const job = await scratchJob();
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

  // ── one document, held by the shell ─────────────────────────────────────
  await session.execute((folder) =>
    window.__TAURI_INTERNALS__.invoke('doc_open', { folder }), job.dir);
  await wait(600);

  const opened = await session.execute(() => document.querySelector('#job-name')?.textContent);
  check('the shell opened a job folder off disk', () =>
    assert.equal(opened, 'Demo Warehouse Reroof', `job name reads "${opened}"`));

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

  const seenInSecond = await session.execute(() =>
    window.__TAURI_INTERNALS__.invoke('doc_get').then((d) => d.job?.name));
  check('the second window is on the same job, not its own copy', () =>
    assert.equal(seenInSecond, 'Demo Warehouse Reroof'));

  // Change the job from the FIRST window; the SECOND must hear about it.
  await session.switchTo(before[0]);
  await session.execute(() => window.__TAURI_INTERNALS__.invoke('doc_set', {
    pointer: '/job/name', value: 'Renamed From The Plan Window',
  }));
  await wait(800);

  await session.switchTo(second);
  const heard = await session.execute(() => window.__PROBE_LAST_NAME__ ?? null);
  const heardViaShell = await session.execute(() =>
    window.__TAURI_INTERNALS__.invoke('doc_get').then((d) => d.job?.name));
  check('a change in one window reaches the other', () =>
    assert.equal(heardViaShell, 'Renamed From The Plan Window',
      `the second window sees "${heardViaShell}" (page-side: ${heard})`));

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
  await job.remove();
}

const failed = results.filter((r) => r === 'FAIL').length;
console.log(failed ? `\nFAIL — ${failed} of ${results.length}` : `\nPASS — ${results.length}/${results.length}`);
process.exit(failed ? 1 : 0);
