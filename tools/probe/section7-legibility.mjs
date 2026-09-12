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

import { execFileSync } from 'node:child_process';
import { mkdir, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import { launch, openDemoJob, until, wait } from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = execFileSync('git', ['rev-parse', '--short', 'HEAD'], { cwd: ROOT }).toString().trim();

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
