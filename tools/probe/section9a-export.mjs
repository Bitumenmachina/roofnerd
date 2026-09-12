// ── Section 9a's done-check ────────────────────────────────────────────────
// "Export CSV puts the lens in the job's own folder, and the file says what the
//  screen says."
//
// The file is the thing that leaves the office, so the check is on the file: it
// is read off the disk with Node's own fs — not read back through the program
// that wrote it — and every cell is held against the cells on the screen it was
// taken from. A total that drifts a cent between the sheet and the attachment
// is the defect this exists to catch.
//
// Everything goes through the front door: the start screen, the editor picker,
// the button. Nothing here calls a command to dress the scene.
//
//   pnpm build:app && node tools/probe/section9a-export.mjs

import { mkdir, readdir, readFile, rm, writeFile } from 'node:fs/promises';
import { realpathSync } from 'node:fs';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import { errorsSoFar, launch, openDemoJob, until, wait, watchErrors, commitStamp } from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = commitStamp();

// Where the front door leads. `openDemoJob` clicks the button on the start
// screen, and that button carries whatever `demo_folder` answered with, which
// in a checkout is this folder — canonical, because that is the form the shell
// hands back and the form the status bar prints.
const JOB = realpathSync(join(ROOT, 'jobs/demo-job'));
const EXPORTS = join(JOB, 'exports');

// The five, with whether money is part of what each one is for. A lens that
// conceals cost has no total to hold against anything — it has a quantity, and
// section 5 already checks that no money reaches it.
const LENSES = [
  { name: 'Drawing', id: 'drawing', money: false },
  { name: 'Stocking', id: 'stocking', money: false },
  { name: 'Condition Summary', id: 'condition-summary', money: true },
  { name: 'Recap', id: 'recap', money: true },
  { name: 'Consolidated', id: 'consolidated', money: true },
];

/** The name the editor gives a file: the lens, and the day it was taken. */
const today = () => {
  const now = new Date();
  const two = (n) => String(n).padStart(2, '0');
  return `${now.getFullYear()}-${two(now.getMonth() + 1)}-${two(now.getDate())}`;
};
const csvName = (id) => `${id}-${today()}.csv`;

const results = [];
const check = (name, fn) => {
  try { fn(); results.push('PASS'); console.log(`  PASS  ${name}`); }
  catch (e) { results.push('FAIL'); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};

/**
 * A CSV, parsed rather than split.
 *
 * A cost column carries `$1,234.56`, which the builder quotes because of the
 * comma. Splitting on commas would read that as two cells and every total would
 * come out wrong — so the check parses the file the way a spreadsheet does.
 */
function parseCsv(text) {
  const rows = [[]];
  let cell = '';
  let quoted = false;
  for (let i = 0; i < text.length; i += 1) {
    const c = text[i];
    if (quoted) {
      if (c !== '"') { cell += c; continue; }
      if (text[i + 1] === '"') { cell += '"'; i += 1; continue; }
      quoted = false;
      continue;
    }
    if (c === '"') { quoted = true; continue; }
    if (c === ',') { rows.at(-1).push(cell); cell = ''; continue; }
    if (c === '\n') { rows.at(-1).push(cell); cell = ''; rows.push([]); continue; }
    if (c === '\r') continue;
    cell += c;
  }
  rows.at(-1).push(cell);
  // A text file ends in a newline, which leaves one empty row behind it.
  if (rows.at(-1).length === 1 && rows.at(-1)[0] === '') rows.pop();
  return rows;
}

/** Money, in cents, or null for a cell that is not a figure. */
const cents = (cell) => {
  const text = String(cell).trim();
  if (!/^-?\$[\d,]+\.\d\d$/.test(text)) return null;
  return Math.round(Number(text.replace(/[$,]/g, '')) * 100);
};

/** Add up one column of a table, counting only the cells that carry money. */
function totalOf(headings, rows) {
  const at = headings.findIndex((h) => /^cost\b/i.test(String(h).trim()));
  if (at < 0) return null;
  let sum = 0;
  let found = 0;
  for (const row of rows) {
    const c = cents(row[at]);
    if (c !== null) { sum += c; found += 1; }
  }
  return { cents: sum, cells: found, column: at };
}

/** Pick a lens off the picker on the left, by the name on it. */
const pickLens = async (session, name) => {
  const ok = await session.execute(function (want) {
    const b = [...document.querySelectorAll('.lens')].find((e) => (e.querySelector('strong') || {}).textContent === want);
    if (!b) return false;
    b.click();
    return true;
  }, name);
  await wait(900);
  return ok;
};

/** The rendered lens, the status bar, and the state of the two buttons. */
const sheetNow = (session) => session.execute(function () {
  const el = document.querySelector('.lens-sheet');
  if (!el) return null;
  const bar = document.querySelector('.status-bar');
  const trouble = document.querySelector('.lens-trouble');
  const write = document.querySelector('.lens-write');
  return JSON.stringify({
    headings: [...el.querySelectorAll('thead th')].map((t) => t.textContent),
    rows: [...el.querySelectorAll('tbody tr')].map((tr) => [...tr.querySelectorAll('td')].map((td) => td.textContent)),
    pending: (el.querySelector('.lens-pending') || {}).textContent || '',
    status: bar ? (bar.textContent || '') : '',
    statusSeen: bar ? (bar.innerText || '') : '',
    trouble: trouble && !trouble.hidden ? trouble.textContent : '',
    label: write ? write.textContent.trim() : null,
    off: write ? !!write.disabled : null,
  });
});

/**
 * The shape of the window, measured rather than looked at.
 *
 * Twice now this check has been green while the window was wrong, and both
 * times the only thing that caught it was a person opening the picture. First
 * the export message pushed the fixed fields and "Units SF · LF · EA · SQ"
 * dropped to a second line in a bar that is one line high. Then, with the
 * fields pinned, the bar answered the body grid's question — how narrow can you
 * be — with the width of its own unbreakable text, the single column came out
 * 1314 px in a 1100 px window, every row stretched to it, and the recap's last
 * column, the lens buttons and the job path all went off the right edge.
 *
 * So the reading is of the whole window, not of the bar: how wide the page lays
 * itself out against how wide the window is, and where the right edge of
 * everything that lives at the right edge actually falls. A rectangle whose
 * right side is past `innerWidth` is a thing the estimator cannot see.
 */
const windowNow = (session) => session.execute(function () {
  const bar = document.querySelector('.status-bar');
  if (!bar) return null;
  const facts = bar.querySelector('.status-facts');
  const path = bar.querySelector('.status-path');
  const message = bar.querySelector('.status-message');
  const sheet = document.querySelector('.lens-sheet');
  const headings = [...document.querySelectorAll('.lens-sheet thead th')];
  const right = function (el) { return el ? Math.round(el.getBoundingClientRect().right) : null; };
  return JSON.stringify({
    // ── the window ──
    innerWidth: window.innerWidth,
    // The body clips, so the page does not scroll — it is simply cut off. Its
    // scrollWidth is what it WOULD need, which is the number that tells you.
    pageWidth: document.body.scrollWidth,
    documentWidth: document.documentElement.scrollWidth,
    frameWidth: Math.round((document.querySelector('.frame') || bar).getBoundingClientRect().width),
    // ── the bar ──
    height: Math.round(bar.getBoundingClientRect().height),
    content: bar.scrollHeight,
    room: bar.clientHeight,
    factsHeight: facts ? Math.round(facts.getBoundingClientRect().height) : 0,
    fields: [...bar.querySelectorAll('.status-field')].map(function (f) {
      return { text: (f.textContent || '').trim(), top: Math.round(f.getBoundingClientRect().top) };
    }),
    path: path ? (path.textContent || '') : '',
    pathRight: right(path),
    // The last folder is the job, and it is the part that must survive.
    pathTail: (bar.querySelector('.path-tail') || {}).textContent || '',
    messageRight: right(message),
    messageCut: message ? message.scrollWidth > message.clientWidth : null,
    // ── what lives at the right edge of the lens ──
    buttons: [...document.querySelectorAll('.lens-actions button')].map(function (b) {
      return { text: (b.textContent || '').trim(), right: right(b) };
    }),
    lastHeading: headings.length
      ? { text: headings[headings.length - 1].textContent, right: right(headings[headings.length - 1]) }
      : null,
    // A table may be wider than the sheet it sits in — that is what the sheet's
    // own scroller is for. Carried so a narrow window can be told apart from a
    // page that has run off the edge.
    sheetScrolls: sheet ? sheet.scrollWidth > sheet.clientWidth : null,
  });
});

/**
 * Press Export CSV, the way a hand does.
 *
 * It waits for the word to be back on the button first: the button says
 * "Exported" for a moment after a press, and a second press inside that moment
 * is the estimator double-clicking, not the check exercising an overwrite.
 */
const exportNow = async (session) => {
  await until(session, function () {
    const b = document.querySelector('.lens-write');
    return !!b && b.textContent.trim() === 'Export CSV' && !b.disabled;
  }, { what: 'the export button to be ready' });

  const pressed = await session.execute(function () {
    const b = document.querySelector('.lens-write');
    if (!b || b.disabled || b.textContent.trim() !== 'Export CSV') return false;
    b.click();
    return true;
  });
  await wait(1200);
  return pressed;
};

/**
 * The status bar on its own, cropped by the driver where it will do it, and the
 * whole window where it will not. Evidence is never the reason a check fails.
 */
async function statusBarShot(session) {
  try {
    const found = await session.call('POST', '/element', { using: 'css selector', value: '.status-bar' });
    const id = Object.values(found)[0];
    return await session.call('GET', `/element/${id}/screenshot`);
  } catch {
    return await session.screenshot();
  }
}

await mkdir(EVIDENCE, { recursive: true });

// A clean slate in the job's exports folder: the sheets a previous run left
// there, and nothing else. "One file, not two" has to be a statement about this
// run — and the folder itself is tracked, so it is emptied, never removed.
await mkdir(EXPORTS, { recursive: true });
for (const f of await readdir(EXPORTS)) {
  if (f.endsWith('.csv')) await rm(join(EXPORTS, f), { force: true });
}

const app = await launch();
const { session } = app;

try {
  await until(session, () => document.querySelector('#editor')?.children.length > 0, { what: 'the window' });

  // Before anything is open. The strongest form of "the button is off with no
  // job" is that there is no button: the window is on the start screen and the
  // editor is not mounted at all.
  const cold = JSON.parse(await session.execute(function () {
    return JSON.stringify({
      start: !!document.querySelector('.start'),
      write: !!document.querySelector('.lens-write'),
    });
  }));
  check('with no job open there is no export button to press', () => {
    assert.ok(cold.start, 'the window did not open on the start screen');
    assert.equal(cold.write, false, 'an export button is reachable with no job open');
  });

  await openDemoJob(session);
  await wait(1200);

  await session.execute(function () {
    const p = document.querySelector('.editor-picker');
    p.value = 'reports';
    p.dispatchEvent(new Event('change', { bubbles: true }));
  });
  await wait(2500);
  await until(session, () => !!document.querySelector('.lens-sheet'), { what: 'the reports editor' });
  // After the switch, not before it: the picker reloads the window.
  await watchErrors(session);

  // The window as it stands with a job open and nothing said in the bar yet.
  const barBefore = JSON.parse(await windowNow(session));

  // ── every lens, out of the window and onto the disk ─────────────────────
  for (const lens of LENSES) {
    const picked = await pickLens(session, lens.name);
    check(`the ${lens.name} lens can be picked`, () => assert.ok(picked, 'it is not in the picker'));

    const before = JSON.parse(await sheetNow(session));
    check(`${lens.name}: the button offers the words, and a job is open so it works`, () => {
      assert.equal(before.label, 'Export CSV', `the button reads "${before.label}"`);
      assert.equal(before.off, false, 'it is switched off with a job open');
    });

    const pressed = await exportNow(session);
    check(`${lens.name}: pressing it wrote something`, () => assert.ok(pressed, 'the button could not be pressed'));

    const after = JSON.parse(await sheetNow(session));
    check(`${lens.name}: nothing went wrong in the window`, () => {
      assert.equal(after.trouble, '', `the window says: ${after.trouble}`);
    });

    // Read with Node's own fs, out of the job folder, rather than back through
    // the program that wrote it. A file that only the writer can see is not a
    // file anybody can attach to an email.
    const path = join(EXPORTS, csvName(lens.id));
    const text = await readFile(path, 'utf8').catch(() => '');
    check(`${lens.name}: ${csvName(lens.id)} is in the job's own exports folder`, () => {
      assert.ok(text.length > 0, `nothing at ${path}`);
    });

    const csv = parseCsv(text);
    check(`${lens.name}: it parses, and every line is the same width`, () => {
      assert.ok(csv.length > 1, 'a heading and nothing under it');
      for (const [i, row] of csv.entries()) {
        assert.equal(row.length, csv[0].length, `line ${i + 1} has ${row.length} cells, not ${csv[0].length}`);
      }
    });

    check(`${lens.name}: the headings are the screen's headings`, () => {
      assert.deepEqual(csv[0], before.headings.map((h) => String(h)));
    });

    check(`${lens.name}: every row on the screen is in the file, cell for cell`, () => {
      assert.ok(before.rows.length > 0, 'the screen shows no rows');
      for (const [i, row] of before.rows.entries()) {
        assert.deepEqual(csv[i + 1], row.map((c) => String(c)), `row ${i + 1}`);
      }
    });

    if (lens.money) {
      check(`${lens.name}: the file's total is the screen's total, to the cent`, () => {
        const screen = totalOf(before.headings, before.rows);
        const file = totalOf(csv[0], csv.slice(1));
        assert.ok(screen, `no cost column on screen: ${before.headings.join(', ')}`);
        assert.ok(file, `no cost column in the file: ${csv[0].join(', ')}`);
        assert.ok(screen.cells > 0, 'no money on the screen to compare');
        assert.equal(file.cents, screen.cents,
          `the file adds to ${file.cents} cents and the screen to ${screen.cents}`);
      });
    }

    // What a total leaves out has to travel with it. The file is the copy that
    // goes to accounting, which is the one place a footnote must not be the
    // thing that stayed behind on the screen.
    if (before.pending) {
      check(`${lens.name}: what the total left out is in the file too`, () => {
        const said = /Not in this total \((\d+)\)/.exec(before.pending);
        assert.ok(said, `the screen's footnote reads "${before.pending.slice(0, 80)}"`);
        assert.ok(text.includes(`Not in this total (${said[1]})`), 'the file carries no footnote');
      });
    }

    check(`${lens.name}: the status bar names the file, and says it on screen`, () => {
      assert.ok(after.status.includes(csvName(lens.id)),
        `the status bar reads "${after.status.trim()}"`);
      assert.ok(after.status.includes(EXPORTS), `it does not say where: "${after.status.trim()}"`);
      // And it is rendered, not merely in the markup. What it looks like is the
      // image beside this check; that it is there at all is this line.
      assert.match(after.statusSeen, /Exported/, `nothing of it is on screen: "${after.statusSeen.trim()}"`);
    });
  }

  // ── the supply house gets no money, on the disk either ──────────────────
  const stocking = await readFile(join(EXPORTS, csvName('stocking')), 'utf8').catch(() => '');
  check('the stocking file carries no cost column', () => {
    const headings = parseCsv(stocking)[0] ?? [];
    assert.ok(headings.length > 0, 'no stocking file to read');
    for (const h of headings) assert.ok(!/cost|price|\$/i.test(h), `heading "${h}"`);
  });
  check('and not a dollar figure anywhere in it', () => {
    assert.ok(!stocking.includes('$'), 'a dollar sign reached the supply house');
  });

  // ── twice in a day is one file ──────────────────────────────────────────
  await pickLens(session, 'Recap');
  const first = await readFile(join(EXPORTS, csvName('recap')), 'utf8');
  const again = await exportNow(session);
  check('the same lens can be exported again', () => assert.ok(again, 'the button could not be pressed'));

  const files = (await readdir(EXPORTS)).filter((f) => f.endsWith('.csv')).sort();
  check('a second export of the same lens overwrites rather than duplicates', () => {
    const mine = files.filter((f) => /^recap-\d{4}-\d{2}-\d{2}\.csv$/.test(f));
    assert.deepEqual(mine, [csvName('recap')], `the exports folder holds ${files.join(', ')}`);
  });
  check('and every lens has exactly one file, not five with a copy each', () => {
    assert.deepEqual(files, LENSES.map((l) => csvName(l.id)).sort());
  });
  const second = await readFile(join(EXPORTS, csvName('recap')), 'utf8');
  check('and the file was replaced, not added to', () => {
    assert.equal(parseCsv(second).length, parseCsv(first).length,
      'the second export left a longer file — it appended');
    assert.deepEqual(parseCsv(second)[0], parseCsv(first)[0], 'the headings changed');
  });

  // ── the bar is still one line, and the window still fits ───────────────
  const barAfter = JSON.parse(await windowNow(session));
  check('the status bar is still one line after an export', () => {
    assert.equal(barAfter.height, barBefore.height,
      `the bar was ${barBefore.height}px and is now ${barAfter.height}px`);
    assert.equal(barAfter.factsHeight, barBefore.factsHeight,
      `the fields were ${barBefore.factsHeight}px and are now ${barAfter.factsHeight}px — they wrapped`);
    assert.ok(barAfter.content <= barAfter.room + 1,
      `${barAfter.content}px of content in a ${barAfter.room}px bar`);
  });
  check('and Job, Scenario, Scale and Units are all on that line', () => {
    const shown = barAfter.fields.map((f) => `${f.text}@${f.top}`).join(' / ');
    for (const want of ['Job', 'Scenario', 'Scale', 'Units']) {
      assert.ok(barAfter.fields.some((f) => f.text.startsWith(want)), `no ${want} field: ${shown}`);
    }
    const lines = new Set(barAfter.fields.map((f) => f.top));
    assert.equal(lines.size, 1, `the fields sit on ${lines.size} lines: ${shown}`);
  });
  check('and the bar still says where the job is', () => {
    assert.ok(barAfter.path.includes(JOB), `the path slot reads "${barAfter.path}"`);
    assert.equal(barAfter.pathTail, '/demo-job',
      `the last folder is what must survive, and the path ends "${barAfter.pathTail}"`);
  });

  // The window itself. The bar saying its line is not the same claim as the
  // window still fitting in the window — the second one is what the picture
  // showed and the first one did not.
  check('the page is no wider than the window after an export', () => {
    assert.equal(barAfter.pageWidth, barAfter.innerWidth,
      `the page needs ${barAfter.pageWidth}px in a ${barAfter.innerWidth}px window`
      + ` (it needed ${barBefore.pageWidth}px before the export)`);
    assert.ok(barAfter.documentWidth <= barAfter.innerWidth,
      `the document is ${barAfter.documentWidth}px in a ${barAfter.innerWidth}px window`);
    assert.equal(barAfter.frameWidth, barBefore.frameWidth,
      `the frame was ${barBefore.frameWidth}px and the export stretched it to ${barAfter.frameWidth}px`);
  });
  check('both buttons on the lens are inside the window', () => {
    assert.equal(barAfter.buttons.length, 2, `${barAfter.buttons.length} buttons on the header`);
    for (const b of barAfter.buttons) {
      assert.ok(b.right <= barAfter.innerWidth,
        `"${b.text}" ends at ${b.right}px in a ${barAfter.innerWidth}px window`);
    }
  });
  check('the last column of the recap is inside the window', () => {
    assert.ok(barAfter.lastHeading, 'the recap has no headings');
    assert.ok(barAfter.lastHeading.right <= barAfter.innerWidth,
      `"${barAfter.lastHeading.text}" ends at ${barAfter.lastHeading.right}px in a`
      + ` ${barAfter.innerWidth}px window (the sheet's own scroller is`
      + ` ${barAfter.sheetScrolls ? 'in use — the window may simply be narrow' : 'not in use'})`);
  });
  check('and the message gave way instead, ellipsized inside the bar', () => {
    assert.ok(barAfter.messageRight <= barAfter.innerWidth,
      `the message ends at ${barAfter.messageRight}px in a ${barAfter.innerWidth}px window`);
    assert.ok(barAfter.pathRight <= barAfter.innerWidth,
      `the job path ends at ${barAfter.pathRight}px in a ${barAfter.innerWidth}px window`);
  });

  await writeFile(join(EVIDENCE, `section9a-export-${COMMIT}.png`),
    Buffer.from(await session.screenshot(), 'base64'));
  await writeFile(join(EVIDENCE, `section9a-status-${COMMIT}.png`),
    Buffer.from(await statusBarShot(session), 'base64'));

  // ── nothing broke on the way ────────────────────────────────────────────
  const errors = await errorsSoFar(session);
  check('and the window logged nothing going wrong', () => {
    assert.deepEqual(errors, [], errors.join(' | '));
  });
} catch (e) {
  results.push('FAIL');
  console.log(`  FAIL  the check stopped: ${e.message}`);
} finally {
  await app.close();
}

const failed = results.filter((r) => r === 'FAIL').length;
console.log(failed ? `\nFAIL — ${failed} of ${results.length}` : `\nPASS — ${results.length}/${results.length}`);
process.exit(failed ? 1 : 0);
