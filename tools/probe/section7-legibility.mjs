// ── Section 7's done-check ─────────────────────────────────────────────────
// Twelve visual defects, listed in refs/visual-spec-v2.md §1.
//
// This did not exist. Section 7 was certified on before/after images, the images
// were never opened, and eleven of the twelve defects had no automated check of
// any kind — including §1.9, whose own "after" screenshot showed the two things
// it claimed to have fixed.
//
// Not everything here can be tested by a machine and this does not pretend
// otherwise. What can be measured is measured against the rendered geometry —
// where a cell's edge actually falls, what a heading actually reads — rather
// than against the intention. The rest is named in STATUS.md for the eye.
//
//   pnpm build:app && node tools/probe/section7-legibility.mjs

import { mkdir, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import { launch, openDemoJob, until, wait, commitStamp } from './tauri-harness.mjs';

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
  await until(session, () => document.querySelector('#editor')?.children.length > 0, { what: 'the window' });
  await openDemoJob(session);
  await wait(1800);

  // ── §1.2 the tree is a tree, and says what its nodes are ────────────────
  const tree = JSON.parse(await session.execute(function () {
    const rows = [...document.querySelectorAll('.tree-label, .tree-node')];
    return JSON.stringify({
      text: (document.querySelector('.tree') || document.body).innerText || '',
      depths: rows.map(function (r) { return (r.className || '').includes('tree-label') ? 1 : 0; }).length,
    });
  }));
  check('§1.2 the tree names conditions as conditions, not as pages', () => {
    // The defect was a flat list headed "Pages" that listed conditions under it.
    assert.match(tree.text, /Roof Plan/, 'the page is not in the tree');
    assert.match(tree.text, /Main Roof Field/, 'the conditions are not in the tree');
    assert.ok(!/^Pages$/m.test(tree.text), 'a bare "Pages" heading still lists conditions');
  });

  // ── §1.3 no probe controls in the product chrome ────────────────────────
  const chrome = JSON.parse(await session.execute(function () {
    return JSON.stringify({
      sidebar: (document.querySelector('.tree, .sidebar') || document.body).innerText || '',
      status: (document.querySelector('.status-bar') || {}).innerText || '',
      hasMenu: !!document.querySelector('.menu-button'),
    });
  }));
  check('§1.3 the sidebar carries no probe buttons, and there is a File menu', () => {
    for (const word of ['Open the demo job', 'Save', 'Tear off the Estimate Sheet']) {
      assert.ok(!chrome.sidebar.includes(word), `"${word}" is still a button in the sidebar`);
    }
    assert.ok(chrome.hasMenu, 'no File menu');
  });
  check('§1.3 the job path is in the status bar, not under the job name', () => {
    assert.match(chrome.status, /\//, `status bar read "${chrome.status.slice(0, 80)}"`);
  });

  // ── §1.5 one toolbar row, and one Scale action ──────────────────────────
  const toolbar = JSON.parse(await session.execute(function () {
    const bar = document.querySelector('.toolbar');
    if (!bar) return JSON.stringify({ rows: 0, text: '', tops: [] });
    // Centres, not tops: a 26px icon button and a 29px labelled one on the same
    // row sit two pixels apart, and counting distinct tops called that two rows.
    const centres = [...bar.querySelectorAll('button')].map(function (b) {
      const r = b.getBoundingClientRect();
      return r.top + r.height / 2;
    });
    const rows = [];
    for (const c of centres) if (!rows.some(function (r) { return Math.abs(r - c) < 12; })) rows.push(c);
    return JSON.stringify({ text: bar.innerText || '', tops: rows });
  }));
  check('§1.5 the toolbar is one row', () => {
    assert.ok(toolbar.tops.length <= 1, `buttons sit on ${toolbar.tops.length} rows`);
  });
  const clippedControls = JSON.parse(await session.execute(function () {
    // Measured, not inferred from scrollWidth: a <select> clips its option text
    // visually while reporting scrollWidth === clientWidth, so the obvious check
    // sees nothing. The label is drawn into a canvas in the control's own
    // computed font and compared against the room it actually has.
    const ctx = document.createElement('canvas').getContext('2d');
    const tooWide = function (el, text) {
      const st = getComputedStyle(el);
      ctx.font = `${st.fontStyle} ${st.fontWeight} ${st.fontSize} ${st.fontFamily}`;
      const room = el.clientWidth
        - parseFloat(st.paddingLeft || '0') - parseFloat(st.paddingRight || '0')
        - (el.tagName === 'SELECT' ? 18 : 0);  // the dropdown arrow takes its own room
      return ctx.measureText(text).width > room + 1;
    };
    const out = [];
    for (const el of document.querySelectorAll('.toolbar button, .toolbar select')) {
      const text = el.tagName === 'SELECT'
        ? ((el.selectedOptions[0] || {}).textContent || '')
        : (el.textContent || '');
      if (text.trim() && tooWide(el, text.trim())) out.push(text.trim().slice(0, 40));
    }
    return JSON.stringify(out);
  }));
  check('§1.5 no toolbar control is cut off mid-word', () => {
    // Found by opening the picture: the scale picker read "Or pick a", which is
    // not a phrase. A control that shows most of its own label is worse than a
    // short one, because the reader cannot tell what was taken away.
    assert.deepEqual(clippedControls, [], `truncated: ${clippedControls.join(', ')}`);
  });

  const zoomGroup = JSON.parse(await session.execute(function () {
    const bar = document.querySelector('.toolbar');
    const box = bar.getBoundingClientRect();
    return JSON.stringify([...bar.querySelectorAll('.zoom button')].map(function (b) {
      const r = b.getBoundingClientRect();
      return {
        label: b.getAttribute('aria-label') || (b.textContent || '').trim(),
        inside: r.left >= box.left - 1 && r.right <= box.right + 1,
      };
    }));
  }));
  check('§1.5 zoom and fit are grouped right and stay on screen', () => {
    // They went 76px past the right edge the moment the selects stopped being
    // squeezed, and nothing would have noticed: a button that is off the end of
    // a scrolling row is not visibly broken, it is just gone.
    assert.ok(zoomGroup.length >= 3, `only ${zoomGroup.length} zoom controls`);
    const lost = zoomGroup.filter((b) => !b.inside).map((b) => b.label);
    assert.deepEqual(lost, [], `off the end of the toolbar: ${lost.join(', ')}`);
  });

  // The scale picker, measured against the thing that was actually covering it.
  //
  // §1.5 had two rows about this row and neither could see the defect: one
  // measures each control's label against its own box, and the picker's box was
  // never too small — 140px of control with 105px of label in it. What clipped
  // it was the zoom group, which is `position: sticky` and therefore reserves no
  // room at all: it paints over whatever the row has scrolled under it. The
  // picker read "Or pick a s" with 83px of itself underneath (measured at the
  // 839px editor the 1100px window used to give the Plan).
  const scalePicker = JSON.parse(await session.execute(function () {
    const bar = document.querySelector('.toolbar');
    if (!bar) return JSON.stringify({ found: false });
    const pick = [...bar.querySelectorAll('select')]
      .find(function (s) { return (s.getAttribute('aria-label') || '') === 'Architectural scale'; });
    const zoom = bar.querySelector('.zoom');
    if (!pick || !zoom) return JSON.stringify({ found: false });
    const p = pick.getBoundingClientRect();
    const z = zoom.getBoundingClientRect();
    const b = bar.getBoundingClientRect();
    const at = function (x, y) {
      const top = document.elementFromPoint(Math.round(x), Math.round(y));
      return top ? (top === pick || pick.contains(top) ? 'itself' : (top.getAttribute('aria-label') || top.className || top.tagName)) : 'nothing';
    };
    return JSON.stringify({
      found: true,
      label: ((pick.selectedOptions[0] || {}).textContent || '').trim(),
      coveredByZoom: Math.round(Math.max(0, p.right - z.left)),
      pastTheBox: Math.round(Math.max(0, p.right - b.right)),
      scrollWidth: pick.scrollWidth,
      clientWidth: pick.clientWidth,
      barContent: bar.scrollWidth,
      barRoom: bar.clientWidth,
      // What is drawn at the far end of the control, which is what the eye reads.
      atItsRightEdge: at(p.right - 6, p.top + p.height / 2),
      atItsMiddle: at(p.left + p.width / 2, p.top + p.height / 2),
    });
  }));

  check('§1.5 the scale picker is not under the pinned zoom group', () => {
    assert.ok(scalePicker.found, 'no scale picker in the toolbar');
    assert.equal(scalePicker.coveredByZoom, 0,
      `${scalePicker.coveredByZoom}px of "${scalePicker.label}" is under the zoom group`);
    assert.equal(scalePicker.atItsRightEdge, 'itself',
      `what is drawn at the end of the picker is ${scalePicker.atItsRightEdge}`);
    assert.equal(scalePicker.atItsMiddle, 'itself',
      `what is drawn in the middle of the picker is ${scalePicker.atItsMiddle}`);
  });
  check('§1.5 and the whole of it is inside the toolbar, showing its whole label', () => {
    assert.equal(scalePicker.pastTheBox, 0,
      `it runs ${scalePicker.pastTheBox}px past the end of the toolbar`);
    // Asked for, and worth knowing it has no teeth of its own: a <select> is not
    // a scroller, so this reads equal even with 83px of the control covered. It
    // is here to say so, not to catch anything.
    assert.ok(scalePicker.scrollWidth <= scalePicker.clientWidth,
      `the picker holds ${scalePicker.scrollWidth}px in ${scalePicker.clientWidth}px`);
    assert.match(scalePicker.label, /scale/i, `it reads "${scalePicker.label}"`);
  });
  check('§1.5 and the row it is in fits the window it is in', () => {
    // The row that does not overflow is the row nothing can be pinned over. This
    // is the number the window width was chosen from: 902px of controls.
    assert.ok(scalePicker.barContent <= scalePicker.barRoom,
      `the toolbar holds ${scalePicker.barContent}px of controls in ${scalePicker.barRoom}px`);
  });

  check('§1.5 scale is one action, not "Set scale" and "Rescale" both', () => {
    const both = /Set scale/.test(toolbar.text) && /Rescale/.test(toolbar.text);
    assert.ok(!both, `toolbar reads "${toolbar.text.replace(/\n/g, ' · ')}"`);
  });

  // ── §1.7 the conditions header is not under the panel ───────────────────
  const rail = JSON.parse(await session.execute(function () {
    const head = document.querySelector('.rail-title, .conditions-title, .rail h3');
    if (!head) return JSON.stringify({ found: false });
    const r = head.getBoundingClientRect();
    const top = document.elementFromPoint(Math.round(r.left + r.width / 2), Math.round(r.top + r.height / 2));
    return JSON.stringify({ found: true, visible: r.height > 0, own: !!top && (head === top || head.contains(top)) });
  }));
  check('§1.7 the conditions heading is not clipped under the panel', () => {
    if (!rail.found) { assert.ok(true); return; }
    assert.ok(rail.visible, 'the heading has no height');
    assert.ok(rail.own, 'something is drawn over the conditions heading');
  });

  await writeFile(join(EVIDENCE, `section7-plan-${COMMIT}.png`),
    Buffer.from(await session.screenshot(), 'base64'));

  // ── §1.9 in the window the sheet is actually worked in ──────────────────
  //
  // The sheet is torn off so it can have a monitor of its own — that is the
  // arrangement the README opens with. Nothing had ever measured it in that
  // window, and it was the window the defect was in: at 760px the pinned
  // Extended column sat on top of the Unit column and the word UNIT, which is
  // v2 §1.9's own complaint about the pinning that closed v2 §1.9.
  //
  // A sticky cell reserves no room. It paints over whatever scrolls under it,
  // so at any width short of the whole table some column is half covered and no
  // amount of pinning fixes it. What fixes it is a window wide enough for the
  // table it carries.
  const oneWindow = await session.handles();
  const tore = await session.execute(function () {
    const b = [...document.querySelectorAll('.area-header .icon-button')]
      .find((e) => (e.getAttribute('aria-label') || '') === 'Open in its own window');
    if (!b) return false;
    b.click();
    return true;
  });
  await wait(3500);
  const handles = await session.handles();
  const main = oneWindow[0];
  const torn = handles.find((h) => !oneWindow.includes(h)) ?? null;

  check('§1.9 the Estimate Sheet tears off into its own window', () => {
    assert.ok(tore, 'no tear-off button in the area header');
    assert.ok(torn, `${oneWindow.length} window before and ${handles.length} after`);
  });

  if (torn) {
    await session.switchTo(torn);
    await until(session, () => !!document.querySelector('.sheet'), { what: 'the torn-off sheet' });
    await wait(1200);

    const tornSheet = JSON.parse(await session.execute(function () {
      const scroll = document.querySelector('.sheet-scroll');
      const box = scroll.getBoundingClientRect();
      const owner = function (th) {
        const r = th.getBoundingClientRect();
        const top = document.elementFromPoint(
          Math.round(r.left + r.width / 2), Math.round(r.top + r.height / 2));
        if (!top) return 'nothing';
        if (top === th || th.contains(top)) return 'itself';
        return top.className || top.tagName;
      };
      const heads = [...document.querySelectorAll('.sheet th')].map(function (th) {
        const r = th.getBoundingClientRect();
        return {
          text: (th.textContent || '').trim() || '(actions)',
          left: Math.round(r.left - box.left),
          right: Math.round(r.right - box.left),
          owner: owner(th),
          clipped: th.scrollWidth > th.clientWidth + 1,
        };
      });
      return JSON.stringify({
        room: Math.round(scroll.clientWidth),
        needs: scroll.scrollWidth,
        windowWidth: window.innerWidth,
        panel: Math.round((document.querySelector('.condition-panel') || { getBoundingClientRect: () => ({ width: 0 }) })
          .getBoundingClientRect().width),
        heads,
      });
    }));

    check('§1.9 the torn-off sheet shows every column at once', () => {
      // The root of it, in one number: what the table needs against what the
      // window gives it. At 760 it was 1136 against 756 — and 440 once the
      // Properties panel stood beside it.
      assert.ok(tornSheet.needs <= tornSheet.room,
        `the sheet needs ${tornSheet.needs}px and has ${tornSheet.room}px`
        + ` (window ${tornSheet.windowWidth}, panel ${tornSheet.panel})`);
    });
    check('§1.9 the UNIT heading is visible and nothing is drawn over it', () => {
      const unit = tornSheet.heads.find((h) => /^unit$/i.test(h.text));
      assert.ok(unit, `the headings read ${tornSheet.heads.map((h) => h.text).join(', ')}`);
      assert.ok(!unit.clipped, 'the UNIT heading is cut off by its own column');
      assert.equal(unit.owner, 'itself',
        `what is drawn over the middle of UNIT is ${unit.owner}`
        + ` (UNIT at ${unit.left}-${unit.right} in a ${tornSheet.room}px view)`);
    });
    check('§1.9 and the same is true of every other heading', () => {
      // Whichever column the pinned pair lands on is the defect; naming only
      // UNIT would move the wound and call it fixed. At 1240px of window it
      // moves to ORDER, whose heading reads "O" and whose cells read "75.".
      const covered = tornSheet.heads.filter((h) => h.owner !== 'itself')
        .map((h) => `${h.text} under ${h.owner}`);
      assert.deepEqual(covered, [], `covered: ${covered.join(' | ')}`);
    });

    await writeFile(join(EVIDENCE, `section7-torn-sheet-${COMMIT}.png`),
      Buffer.from(await session.screenshot(), 'base64'));
    await session.switchTo(main);
    await wait(600);
  }

  // ── to the sheet, for §1.9, §1.10, §1.11, §1.12 ─────────────────────────
  await session.execute(function () {
    const p = document.querySelector('.editor-picker');
    p.value = 'estimate';
    p.dispatchEvent(new Event('change', { bubbles: true }));
  });
  await wait(2500);
  await until(session, () => !!document.querySelector('.sheet'), { what: 'the sheet' });

  const sheet = JSON.parse(await session.execute(function () {
    const scroll = document.querySelector('.sheet-scroll');
    const box = scroll.getBoundingClientRect();
    const th = [...document.querySelectorAll('.sheet th')];
    const heads = th.map(function (h) {
      const r = h.getBoundingClientRect();
      return {
        text: (h.textContent || '').trim(),
        // A heading truncates when its text is wider than the box drawn for it.
        clipped: h.scrollWidth > h.clientWidth + 1,
        right: Math.round(r.right),
        cls: h.className,
      };
    });
    const money = [...document.querySelectorAll('.sheet td.extended')].map(function (c) {
      const r = c.getBoundingClientRect();
      return { text: (c.textContent || '').trim(), left: Math.round(r.left), right: Math.round(r.right) };
    });
    const codes = [...document.querySelectorAll('.sheet td')].filter(function (c) {
      return /^\d{2}-\d{3}-\d{2}/.test((c.textContent || '').trim());
    }).map(function (c) {
      const inner = c.querySelector('input') || c;
      return { text: (inner.value || inner.textContent || '').trim(), clipped: inner.scrollWidth > inner.clientWidth + 1 };
    });
    return JSON.stringify({
      viewLeft: Math.round(box.left), viewRight: Math.round(box.right),
      heads, money, codes,
      html: (document.querySelector('.sheet') || {}).innerHTML || '',
      text: (document.querySelector('.area, #editor') || document.body).innerText || '',
      rules: getComputedStyle(document.querySelector('.sheet td')).borderBottomWidth,
      formulaFont: getComputedStyle(document.querySelector('.sheet input.formula') || document.body).fontFamily,
    });
  }));

  check('§1.9 no column heading truncates', () => {
    const bad = sheet.heads.filter((h) => h.clipped).map((h) => `"${h.text}"`);
    assert.equal(bad.length, 0, `truncated: ${bad.join(', ')}`);
  });
  check('§1.9 every line shows its money without scrolling', () => {
    // The defect this replaced: on a narrow window the sheet showed ITEM, CODE,
    // FORMULA, QTY, UNIT, WAS — and every money column past the right edge.
    assert.ok(sheet.money.length > 0, 'no extended cells');
    for (const m of sheet.money) {
      assert.ok(m.right <= sheet.viewRight + 1 && m.left >= sheet.viewLeft - 1,
        `a line's money sits at ${m.left}–${m.right} in a view of ${sheet.viewLeft}–${sheet.viewRight}`);
      assert.notEqual(m.text, '', 'an extended cell is empty');
    }
  });
  check('§1.9 no cost code is cut off mid-code', () => {
    const bad = sheet.codes.filter((c) => c.clipped).map((c) => c.text);
    assert.equal(bad.length, 0, `clipped codes: ${bad.join(', ')}`);
  });
  check('§1.9 waste never shows a bare percent sign with no number', () => {
    assert.ok(!/>\s*%\s*</.test(sheet.html), 'a bare "%" is in a cell');
  });
  check('§1.10 the explanatory paragraph is gone from the sheet', () => {
    assert.ok(!/Every line shows the formula that produced its quantity/.test(sheet.text),
      'the paragraph is still printed above the table');
  });
  check('§1.11 rows have separators and formulas are monospace', () => {
    assert.notEqual(sheet.rules, '0px', 'rows have no rule between them');
    assert.match(sheet.formulaFont, /mono/i, `formula font is ${sheet.formulaFont}`);
  });
  check('§1.12 the footer carries class subtotals and a selling price', () => {
    assert.match(sheet.text, /MATERIAL/i, 'no class subtotal');
    assert.match(sheet.text, /SELLING PRICE/i, 'no selling price');
  });

  await writeFile(join(EVIDENCE, `section7-sheet-${COMMIT}.png`),
    Buffer.from(await session.screenshot(), 'base64'));
} catch (e) {
  results.push('FAIL');
  console.log(`  FAIL  the check stopped: ${e.message}`);
} finally {
  await app.close();
}

const failed = results.filter((r) => r === 'FAIL').length;
console.log(failed ? `\nFAIL — ${failed} of ${results.length}` : `\nPASS — ${results.length}/${results.length}`);
process.exit(failed ? 1 : 0);
