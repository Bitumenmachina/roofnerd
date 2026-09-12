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
import {
  detachCostPanel, launch, openDemoJob, showTab, until, wait, commitStamp,
} from './tauri-harness.mjs';
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
  const opened = await session.execute(() => document.querySelector('.doc-job')?.textContent);
  check('a job opens from the start screen, by a button a person clicks', () =>
    assert.ok(opened && opened.length > 0, `job name reads "${opened}"`));

  // ── two real OS windows on that one document ────────────────────────────
  // Through the button, not the command: the panel is detached by pressing the
  // thing beside the tabs, which is the path a person has. This used to invoke
  // `open_editor` over the bridge, which tests that the shell can open a window
  // and says nothing about whether anybody could ask it to.
  const before = await session.handles();
  const pressed = await detachCostPanel(session);
  const after = await session.handles();

  check('the cost panel detaches into a second OS window', () => {
    assert.equal(pressed, true, 'there is no button beside the tabs to press');
    assert.ok(after.length > before.length, `${before.length} before, ${after.length} after`);
  });

  const second = after.find((h) => !before.includes(h));
  await session.switchTo(second);
  // The detached window IS the panel: there is no surface in it to wait for.
  await until(session, () => !!document.querySelector('.cost-panel.detached .cost-totals'),
    { what: 'the detached cost panel to mount' });

  // The detached window has no document bar — it is the panel and nothing else
  // — so what it knows about the job is read from its own title bar, which it
  // does have and which names the job whose money it is showing.
  const seenInSecond = await session.execute(() =>
    (document.querySelector('.cost-title-name') || {}).textContent || '');
  check('the second window is on the same job, not its own copy', () =>
    assert.ok(opened && seenInSecond.includes(opened), `it shows "${seenInSecond}"`));

  // Change the job by typing in a field in ONE window, and watch the OTHER
  // follow. Typing rather than writing to the document: the point is that the
  // loop works for a person, not that the bridge works.
  //
  // Which window does the typing has changed with the anatomy, and it had to.
  // The properties are the cost panel's Condition tab, and the panel is what
  // detaches — so with it on the second monitor the fields are THERE and the
  // drawing is here. That is the arrangement the handoff draws and it is the
  // stronger test of the same claim: the estimator types beside the money and
  // the roof follows on the other screen.
  await showTab(session, 'Condition');
  await session.execute(() => {
    const row = [...document.querySelectorAll('.cost-condition .condition')]
      .find((r) => /Parapet/.test(r.textContent || ''));
    if (row) row.click();
  });
  await wait(700);

  const heightWas = await session.execute(() => {
    const label = [...document.querySelectorAll('.condition-panel label')]
      .find((l) => l.querySelector('span')?.textContent === 'Height');
    const field = label ? label.querySelector('input') : null;
    return field ? field.value : null;
  });

  const typed = await session.execute(() => {
    const label = [...document.querySelectorAll('.condition-panel label')]
      .find((l) => l.querySelector('span')?.textContent === 'Height');
    const field = label ? label.querySelector('input') : null;
    if (!field) return null;
    const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
    setter.call(field, '7');
    field.dispatchEvent(new Event('input', { bubbles: true }));
    return true;
  });
  check('a property can be typed in the window the cost panel is in', () => {
    assert.equal(typed, true, `the panel read its height as "${heightWas}"`);
  });
  await wait(1400);

  await session.switchTo(before[0]);
  const heard = await session.execute(() => {
    const row = [...document.querySelectorAll('.plan-rail .condition')]
      .find((r) => /Parapet/.test(r.textContent || ''));
    return row ? (row.innerText || '').replace(/\s+/g, ' ') : '';
  });
  check('and the roof in the other window follows it', () =>
    assert.match(heard, /7 ft high/, `the condition reads "${heard}"`));

  // Put it back, so nothing after this reads a job this check moved.
  await session.switchTo(second);
  await session.execute((was) => {
    const label = [...document.querySelectorAll('.condition-panel label')]
      .find((l) => l.querySelector('span')?.textContent === 'Height');
    const field = label ? label.querySelector('input') : null;
    if (!field) return false;
    const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
    setter.call(field, String(was ?? ''));
    field.dispatchEvent(new Event('input', { bubbles: true }));
    return true;
  }, heightWas);
  await wait(900);

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
  await writeBinary(join(EVIDENCE, `runtime-cost-${COMMIT}.png`), Buffer.from(await session.screenshot(), 'base64'));
} catch (e) {
  results.push('FAIL');
  console.log(`  FAIL  the check stopped: ${e.message}`);
} finally {
  await app.close();
}

const failed = results.filter((r) => r === 'FAIL').length;
console.log(failed ? `\nFAIL — ${failed} of ${results.length}` : `\nPASS — ${results.length}/${results.length}`);
process.exit(failed ? 1 : 0);
