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
import {
  detachCostPanel, launch, openDemoJob, showTab, tool, until, wait, commitStamp,
} from './tauri-harness.mjs';

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

  // Conditions are listed in the cost panel's Condition tab (D148).
  await showTab(session, 'Condition');

  // ── §1.2 what is traced is named as what it is ──────────────────────────
  // The defect was a flat list headed "Pages" that listed conditions under it.
  // There is no tree at all now (D132) and the claim survives it: the things an
  // estimator traced are listed as conditions, under a heading that says so,
  // with the drawing they are on named where the drawing is chosen. Neither
  // half is asserted less — the page and the conditions are still both on
  // screen and still not confused for one another.
  const listed = JSON.parse(await session.execute(function () {
    return JSON.stringify({
      heading: ((document.querySelector('.cost-condition .rail-heading') || {}).textContent || '').trim(),
      rows: [...document.querySelectorAll('.cost-condition .condition .condition-name')]
        .map(function (e) { return (e.textContent || '').trim(); }),
      drawing: (document.querySelector('.tool-strip') || {}).innerText || '',
    });
  }));
  check('§1.2 conditions are listed as conditions, and the drawing is named apart from them', () => {
    assert.equal(listed.heading, 'Conditions', `the list is headed "${listed.heading}"`);
    assert.ok(listed.rows.includes('Main Roof Field'),
      `the list reads ${JSON.stringify(listed.rows)}`);
    assert.ok(!listed.rows.includes('Roof Plan'),
      'the drawing is being listed as a condition');
    assert.match(listed.drawing, /Roof Plan/,
      `the strip names no drawing: "${listed.drawing.replace(/\s+/g, ' ')}"`);
  });

  // ── §1.3 no probe controls in the product chrome ────────────────────────
  const chrome = JSON.parse(await session.execute(function () {
    const bar = document.querySelector('.doc-bar');
    return JSON.stringify({
      docBar: bar ? (bar.innerText || '') : '',
      // The handoff: the document bar never holds a tool. Read as the thing
      // itself — no act from the rail is drawn in it.
      toolsInDocBar: [...document.querySelectorAll('.doc-bar .rail-tool')].length,
      status: (document.querySelector('.status-strip') || {}).innerText || '',
      hasMenu: !!document.querySelector('.doc-bar .menu-button'),
      list: (document.querySelector('.cost-condition') || document.body).innerText || '',
    });
  }));
  check('§1.3 the list carries no probe buttons, and there is a File menu', () => {
    for (const word of ['Open the demo job', 'Save', 'Tear off the Estimate Sheet']) {
      assert.ok(!chrome.list.includes(word), `"${word}" is still a button beside the drawing`);
    }
    assert.ok(chrome.hasMenu, 'no File menu');
  });
  check('§1.3 and the document bar holds no tool', () => {
    // The handoff's own rule for this region, which is the same rule §1.3 was
    // making: the bar is the identity of the open file and nothing else.
    assert.equal(chrome.toolsInDocBar, 0, 'an act from the rail is drawn in the document bar');
  });
  check('§1.3 the job path is in the status strip, not under the job name', () => {
    assert.match(chrome.status, /\//, `status strip read "${chrome.status.slice(0, 80)}"`);
  });

  await showTab(session, 'Estimate');

  // ── §1.5 the acts, and the settings they need ───────────────────────────
  //
  // §1.5 was a row: one row, nothing cut off mid-word, zoom and fit grouped
  // right and still reachable, and one Scale action rather than two. The row is
  // gone — the acts are the tool rail and their settings are the tool settings
  // strip — and every one of those claims has an owner in the new anatomy:
  //
  //   one row                → the strip is one 42px row and never two
  //   nothing cut off        → nothing in the strip is wider than its own box
  //   zoom and fit reachable → they are an overlay on the drawing, and an
  //                            overlay cannot be covered by a row it is not in
  //   one Scale action       → the rail has exactly one act that sets a scale
  //
  // The picker that read "Or pick a s" under the pinned zoom group is measured
  // the same way it was: what is actually drawn at its own edge and its own
  // middle, by `elementFromPoint`. That is the reading that caught it, and it
  // works wherever the control lives.
  await tool(session, 'Scale');

  const strip = JSON.parse(await session.execute(function () {
    const bar = document.querySelector('.tool-strip');
    if (!bar) return JSON.stringify({ found: false });
    const box = bar.getBoundingClientRect();
    // Centres, not tops: a 26px control and a 29px one on the same row sit two
    // pixels apart, and counting distinct tops called that two rows.
    const centres = [...bar.querySelectorAll('button, select, input, .strip-field')].map(function (b) {
      const r = b.getBoundingClientRect();
      return r.top + r.height / 2;
    });
    const rows = [];
    for (const c of centres) if (!rows.some(function (r) { return Math.abs(r - c) < 12; })) rows.push(c);

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
    const clipped = [];
    for (const el of bar.querySelectorAll('button, select')) {
      const text = el.tagName === 'SELECT'
        ? ((el.selectedOptions[0] || {}).textContent || '')
        : (el.textContent || '');
      if (text.trim() && tooWide(el, text.trim())) clipped.push(text.trim().slice(0, 40));
    }

    const at = function (x, y) {
      const top = document.elementFromPoint(Math.round(x), Math.round(y));
      return top ? (top.getAttribute('aria-label') || top.className || top.tagName) : 'nothing';
    };
    const pick = [...bar.querySelectorAll('select')]
      .find(function (el) { return (el.getAttribute('aria-label') || '') === 'Architectural scale'; });
    let picker = null;
    if (pick) {
      const p = pick.getBoundingClientRect();
      const own = function (x, y) {
        const top = document.elementFromPoint(Math.round(x), Math.round(y));
        return top ? (top === pick || pick.contains(top) ? 'itself' : (top.getAttribute('aria-label') || top.className || top.tagName)) : 'nothing';
      };
      picker = {
        label: ((pick.selectedOptions[0] || {}).textContent || '').trim(),
        pastTheBox: Math.round(Math.max(0, p.right - box.right)),
        scrollWidth: pick.scrollWidth,
        clientWidth: pick.clientWidth,
        atItsRightEdge: own(p.right - 6, p.top + p.height / 2),
        atItsMiddle: own(p.left + p.width / 2, p.top + p.height / 2),
      };
    }

    // The inner box as well as the bar. The bar clips at its own edge and the
    // field box clips inside it, so asking only the bar "do you hold your own
    // content" reads equal however much is crammed into it — the same
    // toothlessness the scale picker's `scrollWidth` had. What actually
    // overflows is the row of fields, and that is the number with the teeth.
    const fields = bar.querySelector('.tool-strip-fields');
    return JSON.stringify({
      found: true,
      rows: rows.length,
      height: Math.round(box.height),
      content: bar.scrollWidth,
      room: bar.clientWidth,
      fieldsContent: fields ? fields.scrollWidth : 0,
      fieldsRoom: fields ? fields.clientWidth : 0,
      text: bar.innerText || '',
      clipped,
      picker,
      at,
    });
  }));

  check('§1.5 the tool settings strip is one row', () => {
    assert.ok(strip.found, 'there is no tool settings strip');
    assert.ok(strip.rows <= 1, `its controls sit on ${strip.rows} rows`);
    assert.equal(strip.height, 42, `the strip is ${strip.height}px and is specified at 42px`);
  });
  check('§1.5 no control in it is cut off mid-word', () => {
    // Found by opening the picture: the scale picker read "Or pick a", which is
    // not a phrase. A control that shows most of its own label is worse than a
    // short one, because the reader cannot tell what was taken away.
    assert.deepEqual(strip.clipped, [], `truncated: ${strip.clipped.join(', ')}`);
  });
  check('§1.5 and the row it is in fits the window it is in', () => {
    // The row that does not overflow is the row nothing can be pinned over.
    // Both boxes: the bar, and the field row inside it that actually carries
    // the controls. The bar alone reads equal at any amount of overflow,
    // because the field row clips before the bar ever sees it.
    assert.ok(strip.content <= strip.room,
      `the strip holds ${strip.content}px of controls in ${strip.room}px`);
    assert.ok(strip.fieldsContent <= strip.fieldsRoom,
      `its controls come to ${strip.fieldsContent}px in ${strip.fieldsRoom}px of row`);
  });
  check('§1.5 the scale picker has nothing drawn over it', () => {
    assert.ok(strip.picker, 'no scale picker in the strip while the Scale act is up');
    assert.equal(strip.picker.atItsRightEdge, 'itself',
      `what is drawn at the end of the picker is ${strip.picker.atItsRightEdge}`);
    assert.equal(strip.picker.atItsMiddle, 'itself',
      `what is drawn in the middle of the picker is ${strip.picker.atItsMiddle}`);
    assert.equal(strip.picker.pastTheBox, 0,
      `it runs ${strip.picker.pastTheBox}px past the end of the strip`);
    // Asked for, and worth knowing it has no teeth of its own: a <select> is not
    // a scroller, so this reads equal even with most of the control covered. It
    // is here to say so, not to catch anything.
    assert.ok(strip.picker.scrollWidth <= strip.picker.clientWidth,
      `the picker holds ${strip.picker.scrollWidth}px in ${strip.picker.clientWidth}px`);
    assert.match(strip.picker.label, /scale/i, `it reads "${strip.picker.label}"`);
  });

  const viewNav = JSON.parse(await session.execute(function () {
    const box = document.querySelector('.surface-region').getBoundingClientRect();
    return JSON.stringify([...document.querySelectorAll('.view-nav .icon-button')].map(function (b) {
      const r = b.getBoundingClientRect();
      const top = document.elementFromPoint(
        Math.round(r.left + r.width / 2), Math.round(r.top + r.height / 2));
      return {
        label: b.getAttribute('aria-label') || (b.textContent || '').trim(),
        inside: r.left >= box.left - 1 && r.right <= box.right + 1
          && r.top >= box.top - 1 && r.bottom <= box.bottom + 1,
        owner: !top ? 'nothing' : ((top === b || b.contains(top)) ? 'itself' : (top.getAttribute('aria-label') || top.className || top.tagName)),
      };
    }));
  }));
  check('§1.5 zoom and fit are grouped at the drawing and stay reachable', () => {
    // They went 76px past the right edge of the old row the moment the selects
    // stopped being squeezed, and nothing would have noticed: a button off the
    // end of a scrolling row is not visibly broken, it is just gone. On the
    // drawing there is no row to run off, and the reading here is the one that
    // matters — a click at the middle of each one lands on it.
    assert.ok(viewNav.length >= 3, `only ${viewNav.length} view controls`);
    const lost = viewNav.filter((b) => !b.inside).map((b) => b.label);
    assert.deepEqual(lost, [], `outside the drawing: ${lost.join(', ')}`);
    const covered = viewNav.filter((b) => b.owner !== 'itself').map((b) => `${b.label} under ${b.owner}`);
    assert.deepEqual(covered, [], `covered: ${covered.join(' | ')}`);
  });

  const acts = JSON.parse(await session.execute(function () {
    return JSON.stringify([...document.querySelectorAll('.tool-rail .rail-tool')].map(function (b) {
      const r = b.getBoundingClientRect();
      const chip = b.querySelector('.rail-label');
      return {
        label: b.getAttribute('aria-label') || '',
        chip: chip ? (chip.textContent || '').trim() : '',
        width: Math.round(r.width), height: Math.round(r.height),
      };
    }));
  }));
  check('§1.5 scale is one act, not "Set scale" and "Rescale" both', () => {
    const scaling = acts.filter((a) => /scale/i.test(a.label));
    assert.equal(scaling.length, 1, `the rail offers ${JSON.stringify(scaling.map((a) => a.label))}`);
    assert.ok(!/Set scale/.test(strip.text), `the strip reads "${strip.text.replace(/\n/g, ' · ')}"`);
  });
  check('§1.5 every act carries its own word, and they are the trade\'s words', () => {
    // The rail is pictograms with the word on hover. A pictogram whose word is
    // missing is a thing a roofer has to learn, which is the defect §1.5 was
    // about in the first place.
    assert.ok(acts.length >= 5, `the rail offers ${acts.length} acts`);
    for (const a of acts) {
      assert.ok(a.label.length > 0, 'an act with no name at all');
      assert.equal(a.chip, a.label, `"${a.label}" hovers as "${a.chip}"`);
      assert.equal(a.width, 36, `"${a.label}" is ${a.width}px wide and is specified at 36`);
      assert.equal(a.height, 36, `"${a.label}" is ${a.height}px tall and is specified at 36`);
    }
    for (const want of ['Trace an area', 'Run a parapet', 'Drop a drain']) {
      assert.ok(acts.some((a) => a.label === want), `no act called "${want}"`);
    }
  });
  // "Does nothing" used to mean the rail offered a word for an act nobody had
  // wired, and the four here were exactly that. C6 wired them, so the row's own
  // notion of "does something" is stale in the other direction now — a fixed
  // act that still fails this row on the old (absence) reading would be a
  // regression this check could no longer see. What "does something" means for
  // any act on this rail, wired or not, is the one thing every act does before
  // anything is picked: choosing it puts its OWN strip up, named for it — the
  // same observation §1.5's other rows already make of `Scale` and of the
  // traced acts above. So each of the four is chosen in turn and the strip is
  // read, never assumed.
  const verbStrips = {};
  for (const label of ['Set high and low', 'Box out a curb', 'Measure', 'Cut a section']) {
    await tool(session, label);
    await wait(400);
    verbStrips[label] = JSON.parse(await session.execute(function () {
      const bar = document.querySelector('.tool-strip');
      const named = bar ? bar.querySelector('.tool-strip-name') : null;
      return JSON.stringify({ found: !!bar, name: named ? (named.textContent || '').trim() : '' });
    }));
  }
  check('§1.5 and nothing in the rail is an act this program cannot do', () => {
    for (const [label, strip] of Object.entries(verbStrips)) {
      assert.ok(strip.found, `"${label}" put no tool settings strip up at all`);
      assert.equal(strip.name, label, `"${label}" put up a strip named "${strip.name}"`);
    }
  });

  await tool(session, 'Select');

  // ── §1.7 the conditions header is not under the panel ───────────────────
  await showTab(session, 'Condition');
  const rail = JSON.parse(await session.execute(function () {
    // Pointed at the heading that actually exists. It used to look for three
    // class names none of which the Plan has ever used, so the row passed by
    // finding nothing — the weakest way a check can be green. The heading is
    // over the list, which is the Condition tab now (D148).
    const head = document.querySelector('.cost-condition .rail-heading');
    if (!head) return JSON.stringify({ found: false });
    const r = head.getBoundingClientRect();
    const top = document.elementFromPoint(Math.round(r.left + r.width / 2), Math.round(r.top + r.height / 2));
    return JSON.stringify({ found: true, visible: r.height > 0, own: !!top && (head === top || head.contains(top)) });
  }));
  check('§1.7 the conditions heading is not clipped under anything', () => {
    assert.ok(rail.found, 'the Plan has no conditions heading');
    assert.ok(rail.visible, 'the heading has no height');
    assert.ok(rail.own, 'something is drawn over the conditions heading');
  });

  await writeFile(join(EVIDENCE, `section7-plan-${COMMIT}.png`),
    Buffer.from(await session.screenshot(), 'base64'));

  // ── §1.9 in the window the money is actually worked in ──────────────────
  //
  // The sheet is torn off so it can have a monitor of its own — that is the
  // arrangement the README opens with, and it is now the cost panel that goes
  // there rather than an eleven-column table. §1.9's complaint survives the
  // change whole: at any width short of the table, SOME column was half covered
  // by the pinned pair, and the ✕ that takes a line off sat under the floating
  // scrollbar where a hand could not reach it.
  //
  // The compact line has four columns and fits the panel at every width the
  // panel has, so the reading is the one that says so: nothing is drawn over
  // any of them, and none of them is cut off by its own box. And the ✕ is no
  // longer at an edge at all — it is a named button inside the line's own
  // chain, which is the answer to 6c's red row.
  const oneWindow = await session.handles();
  const tore = await detachCostPanel(session);
  const handles = await session.handles();
  const main = oneWindow[0];
  const torn = handles.find((h) => !oneWindow.includes(h)) ?? null;

  check('§1.9 the cost panel detaches into its own window', () => {
    assert.ok(tore, 'no detach button beside the tabs');
    assert.ok(torn, `${oneWindow.length} window before and ${handles.length} after`);
  });

  if (torn) {
    await session.switchTo(torn);
    await until(session, () => !!document.querySelector('.cost-estimate'), { what: 'the detached lines' });
    await wait(1200);

    // Open a line, so the whole chain and its ✕ are on screen to be measured.
    await session.execute(function () {
      const line = document.querySelector('.cost-line .line-what');
      if (line) line.click();
    });
    await wait(900);

    const tornPanel = JSON.parse(await session.execute(function () {
      const scroll = document.querySelector('.cost-lines');
      const box = scroll.getBoundingClientRect();
      const owner = function (el) {
        const r = el.getBoundingClientRect();
        if (r.width < 1 || r.height < 1) return 'nothing — it has no box';
        const top = document.elementFromPoint(
          Math.round(r.left + r.width / 2), Math.round(r.top + r.height / 2));
        if (!top) return 'nothing';
        if (top === el || el.contains(top)) return 'itself';
        // An ANCESTOR answering is not something drawn over the top of this —
        // it is this not being painted at that point at all, because it has
        // been clipped out of the layout. Naming the container as the culprit
        // sent a reader looking for an overlap that was never there.
        if (top.contains(el)) return 'nothing — it is clipped out of the layout';
        return top.className || top.tagName;
      };
      const heads = [...document.querySelectorAll('.cost-columns > span')].map(function (h) {
        const r = h.getBoundingClientRect();
        return {
          text: (h.textContent || '').trim() || '(unit)',
          left: Math.round(r.left - box.left),
          right: Math.round(r.right - box.left),
          owner: owner(h),
          clipped: h.scrollWidth > h.clientWidth + 1,
        };
      });
      const cells = [...document.querySelectorAll('.cost-line .line-money, .cost-line .line-qty')]
        .map(function (c) {
          const r = c.getBoundingClientRect();
          return {
            text: (c.textContent || '').trim(),
            lines: c.getClientRects ? c.getClientRects().length : 0,
            right: Math.round(r.right),
            inside: r.right <= box.right + 1 && r.left >= box.left - 1,
          };
        });
      const off = document.querySelector('.line-chain .remove');
      return JSON.stringify({
        room: Math.round(scroll.clientWidth),
        needs: scroll.scrollWidth,
        windowWidth: window.innerWidth,
        windowHeight: window.innerHeight,
        heads,
        cells,
        removeAt: !off ? null : {
          text: (off.textContent || '').trim(),
          owner: owner(off),
          right: Math.round(off.getBoundingClientRect().right),
        },
        totalsPinned: (function () {
          const totals = document.querySelector('.cost-totals');
          const panel = document.querySelector('.cost-panel');
          if (!totals || !panel) return null;
          const t = totals.getBoundingClientRect();
          const p = panel.getBoundingClientRect();
          return { bottomGap: Math.round(p.bottom - t.bottom), height: Math.round(t.height) };
        })(),
      });
    }));

    check('§1.9 the detached panel shows every column at once', () => {
      // The root of it, in one number: what the lines need against what the
      // window gives them. At 760 the old table was 1136 against 756.
      assert.ok(tornPanel.needs <= tornPanel.room + 1,
        `the lines need ${tornPanel.needs}px and have ${tornPanel.room}px`
        + ` (window ${tornPanel.windowWidth})`);
    });
    check('§1.9 the window is the size the handoff gives it', () => {
      // 600 x 860 (D137). It is load-bearing: the panel's own widths are
      // specified against it, and nothing else asserts it.
      assert.ok(Math.abs(tornPanel.windowWidth - 600) <= 2,
        `it is ${tornPanel.windowWidth}px wide and is specified at 600`);
      assert.ok(Math.abs(tornPanel.windowHeight - 860) <= 2,
        `it is ${tornPanel.windowHeight}px tall and is specified at 860`);
    });
    check('§1.9 every column heading is visible with nothing drawn over it', () => {
      // The columns that carry a word, named rather than counted. The unit
      // column carries none on purpose — the unit is printed on each line
      // beside its quantity, so the header over it is an empty span with no box
      // at all, and reading "it has no box" as "something is drawn over it"
      // reported a defect that was the design. Naming the three is what keeps
      // this from being a loophole: a heading that lost its word would leave
      // the set, and that is a red row.
      const written = tornPanel.heads.filter((h) => h.text !== '(unit)');
      assert.deepEqual(written.map((h) => h.text), ['ITEM', 'QTY', 'COST'],
        `the columns are headed ${tornPanel.heads.map((h) => h.text).join(', ')}`);
      const covered = written.filter((h) => h.owner !== 'itself')
        .map((h) => `${h.text} under ${h.owner}`);
      assert.deepEqual(covered, [], `covered: ${covered.join(' | ')}`);
      const cut = written.filter((h) => h.clipped).map((h) => h.text);
      assert.deepEqual(cut, [], `cut off by their own box: ${cut.join(', ')}`);
    });
    check('§1.9 and every number holds one line, inside the panel', () => {
      // A money figure that wraps is a row twice as tall and a column that
      // reads like two — 6c's rule, at the surface that carries the money now.
      assert.ok(tornPanel.cells.length > 0, 'no quantities or money on screen');
      for (const c of tornPanel.cells) {
        assert.equal(c.lines, 1, `"${c.text}" is on ${c.lines} lines`);
        assert.ok(c.inside, `"${c.text}" ends at ${c.right}px, outside the lines it is in`);
      }
    });
    check('§1.9 and the button that takes a line off can be reached', () => {
      // 6c measured the ✕ as unreachable: it sat in a 34px column jammed to the
      // right edge of the scroller, under the GTK scrollbar that floats over
      // that edge instead of reserving room. It is a named button inside the
      // line's own chain now, away from every edge — and taking a line off is
      // not something to do by accident.
      assert.ok(tornPanel.removeAt, 'no open line has a button to take it off');
      assert.match(tornPanel.removeAt.text, /take this line off/i,
        `the button reads "${tornPanel.removeAt.text}"`);
      assert.equal(tornPanel.removeAt.owner, 'itself',
        `what is drawn over the middle of it is ${tornPanel.removeAt.owner}`);
    });
    check('§1.9 and the totals are pinned at the bottom of the panel', () => {
      assert.ok(tornPanel.totalsPinned, 'there are no pinned totals');
      assert.ok(Math.abs(tornPanel.totalsPinned.bottomGap) <= 1,
        `they sit ${tornPanel.totalsPinned.bottomGap}px off the bottom of the panel`);
    });

    await writeFile(join(EVIDENCE, `section7-detached-cost-${COMMIT}.png`),
      Buffer.from(await session.screenshot(), 'base64'));

    // Home again, so the rest of this runs in one window.
    await session.execute(function () {
      const b = document.querySelector('.cost-dock');
      if (b) b.click();
    });
    await wait(2500);
    await session.switchTo(main);
    await wait(900);
  }

  // ── the lines in the window they are read in, for §1.9 to §1.12 ─────────
  await showTab(session, 'Estimate');
  await until(session, () => !!document.querySelector('.cost-estimate'), { what: 'the lines' });
  await session.execute(function () {
    const line = document.querySelector('.cost-line .line-what');
    if (line) line.click();
  });
  await wait(900);

  const lines = JSON.parse(await session.execute(function () {
    const scroll = document.querySelector('.cost-lines');
    const box = scroll.getBoundingClientRect();
    const heads = [...document.querySelectorAll('.cost-columns > span')].map(function (h) {
      const r = h.getBoundingClientRect();
      return {
        text: (h.textContent || '').trim(),
        clipped: h.scrollWidth > h.clientWidth + 1,
        right: Math.round(r.right),
      };
    });
    const money = [...document.querySelectorAll('.cost-line .line-money')].map(function (c) {
      const r = c.getBoundingClientRect();
      return { text: (c.textContent || '').trim(), left: Math.round(r.left), right: Math.round(r.right) };
    });
    const codes = [...document.querySelectorAll('.line-chain .chain-value input')]
      .filter(function (i) { return /^\d{2}-\d{3}-\d{2}/.test(i.value || ''); })
      .map(function (i) { return { text: i.value, clipped: i.scrollWidth > i.clientWidth + 1 }; });
    // A line whose formula could not be worked out carries the reason under it,
    // and a line whose unit disagrees carries a note: both are meant to make
    // that row taller. They are left out here so this measures wrapping and not
    // the program saying something.
    const rowHeights = [...document.querySelectorAll('.cost-line')]
      .filter(function (r) { return !r.querySelector('.formula-error, .formula-note'); })
      .map(function (r) { return Math.round(r.getBoundingClientRect().height); });
    const numeric = [...document.querySelectorAll('.cost-line .line-qty, .cost-line .line-money')]
      .map(function (c) {
        return {
          text: (c.textContent || '').trim(),
          lines: c.getClientRects ? c.getClientRects().length : 0,
        };
      });
    return JSON.stringify({
      viewLeft: Math.round(box.left), viewRight: Math.round(box.right),
      heads, money, codes, rowHeights, numeric,
      html: (document.querySelector('.cost-estimate') || {}).innerHTML || '',
      text: (document.querySelector('.cost-panel') || document.body).innerText || '',
      rules: getComputedStyle(document.querySelector('.cost-line')).borderBottomWidth,
      formulaFont: getComputedStyle(document.querySelector('.line-chain input.formula') || document.body).fontFamily,
      chipFont: getComputedStyle(document.querySelector('.line-formula') || document.body).fontFamily,
    });
  }));

  check('§1.9 no column heading truncates', () => {
    const bad = lines.heads.filter((h) => h.clipped).map((h) => `"${h.text}"`);
    assert.equal(bad.length, 0, `truncated: ${bad.join(', ')}`);
  });
  check('§1.9 every line shows its money without scrolling', () => {
    // The defect this replaced: on a narrow window the sheet showed ITEM, CODE,
    // FORMULA, QTY, UNIT, WAS — and every money column past the right edge.
    assert.ok(lines.money.length > 0, 'no money on any line');
    for (const m of lines.money) {
      assert.ok(m.right <= lines.viewRight + 1 && m.left >= lines.viewLeft - 1,
        `a line's money sits at ${m.left}–${m.right} in a view of ${lines.viewLeft}–${lines.viewRight}`);
      assert.notEqual(m.text, '', 'a money cell is empty');
    }
  });
  check('§1.9 no cost code is cut off mid-code', () => {
    const bad = lines.codes.filter((c) => c.clipped).map((c) => c.text);
    assert.equal(bad.length, 0, `clipped codes: ${bad.join(', ')}`);
  });
  check('§1.9 every number holds one line, and the rows are one height', () => {
    // 6c's rule: a figure that wraps is a row twice as tall and a column that
    // reads like two. Read off the client rectangles, which is the only reading
    // of "one line" that does not depend on knowing the line height.
    for (const n of lines.numeric) {
      assert.equal(n.lines, 1, `"${n.text}" is drawn on ${n.lines} lines`);
    }
    const heights = [...new Set(lines.rowHeights)];
    assert.ok(heights.length <= 1,
      `the lines are ${heights.join(', ')}px tall — one of them has wrapped`);
  });
  check('§1.9 waste never shows a bare percent sign with no number', () => {
    assert.ok(!/>\s*%\s*</.test(lines.html), 'a bare "%" is in a cell');
  });
  check('§1.10 the explanatory paragraph is gone', () => {
    assert.ok(!/Every line shows the formula that produced its quantity/.test(lines.text),
      'the paragraph is still printed above the lines');
  });
  check('§1.11 rows have separators and formulas are monospace', () => {
    assert.notEqual(lines.rules, '0px', 'the lines have no rule between them');
    assert.match(lines.formulaFont, /mono/i, `the formula field is ${lines.formulaFont}`);
    assert.match(lines.chipFont, /mono/i, `the formula on the line is ${lines.chipFont}`);
  });
  check('§1.12 the pinned totals carry the classes and a selling price', () => {
    assert.match(lines.text, /MATERIAL/i, 'no class subtotal');
    assert.match(lines.text, /SELLING PRICE/i, 'no selling price');
  });

  await writeFile(join(EVIDENCE, `section7-lines-${COMMIT}.png`),
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
