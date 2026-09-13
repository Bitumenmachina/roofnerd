// Vendor data: the tapered-package solver (C2) and the fastening solver (C4) read these two
// files off disk rather than a manufacturer's PDF. This test only checks their SHAPE — that a
// panel set's designations don't repeat and its thicknesses climb without a gap, and that every
// fastening pattern's fastener count and FM class are well-formed. It says nothing about whether
// the numbers themselves are right; the manufacturer's own document is authority for that, not
// this file (see data/README.md).

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const ROOT = resolve(import.meta.dirname, '../../..');
const versico = JSON.parse(readFileSync(resolve(ROOT, 'data/profiles/versico-versicore-mp-h.json'), 'utf8'));
const dr05 = JSON.parse(readFileSync(resolve(ROOT, 'data/fastening/dr-05-2024-07.json'), 'utf8'));

// ── Versico VersiCore MP-H tapered profiles ─────────────────────────────────

function eachPanelSet(fn) {
  let sets = 0;
  for (const slope of versico.slopes) {
    for (const setName of ['standard', 'extended']) {
      const set = slope[setName];
      if (!set) continue;
      fn(`${slope.label} ${setName}`, set);
      sets++;
    }
  }
  return sets;
}

test('every Versico panel set has designations that do not repeat', () => {
  const sets = eachPanelSet((label, set) => {
    const seen = new Set();
    for (const p of set.panels) {
      assert.ok(!seen.has(p.designation), `${label}: ${p.designation} appears twice`);
      seen.add(p.designation);
    }
  });
  assert.ok(sets >= 6, 'expected at least the six documented panel sets (1/8, 1/4, 1/2 x standard/extended)');
});

test('every Versico panel set tapers up with no gap between panels', () => {
  eachPanelSet((label, set) => {
    for (const p of set.panels) {
      assert.ok(p.thick_edge_in > p.thin_edge_in, `${label}: ${p.designation} does not taper up`);
    }
    for (let i = 1; i < set.panels.length; i++) {
      const prev = set.panels[i - 1];
      const cur = set.panels[i];
      assert.equal(cur.thin_edge_in, prev.thick_edge_in,
        `${label}: ${cur.designation}'s thin edge (${cur.thin_edge_in}) does not meet ${prev.designation}'s thick edge (${prev.thick_edge_in})`);
    }
  });
});

test('every Versico panel names its fill the way the profile drawing prints it', () => {
  eachPanelSet((label, set) => {
    assert.match(set.fill_in, /^(2\.0|4\.0) ISO$/, `${label}: fill_in "${set.fill_in}" is not "2.0 ISO" or "4.0 ISO"`);
    for (const p of set.panels) {
      assert.equal(p.fill_in, set.fill_in, `${label}: ${p.designation} names a different fill than its set`);
    }
  });
});

// ── Carlisle DR-05 fastening patterns ────────────────────────────────────────

test('DR-05 patterns count matches the length of the list', () => {
  assert.equal(dr05.patterns.length, dr05.pattern_count);
  assert.ok(dr05.patterns.length > 0);
});

test('every DR-05 pattern carries a positive integer fastener count', () => {
  for (const p of dr05.patterns) {
    assert.ok(Number.isInteger(p.fasteners) && p.fasteners > 0,
      `${p.board}/${p.fasteners}: fastener count is not a positive integer`);
  }
});

test('every DR-05 rating\'s fm_class matches 1-\\d+', () => {
  let checked = 0;
  for (const p of dr05.patterns) {
    for (const r of p.ratings) {
      assert.match(r.fm_class, /^1-\d+$/,
        `${p.board}/${p.fasteners}: fm_class "${r.fm_class}" does not match 1-\\d+`);
      checked++;
    }
  }
  assert.ok(checked > 0, 'expected at least one rating to check');
});

test('a DR-05 pattern is enhancement-only XOR it carries a rating', () => {
  for (const p of dr05.patterns) {
    if (p.enhancement_only) {
      assert.equal(p.ratings.length, 0, `${p.board}/${p.fasteners}: enhancement-only but carries ratings`);
      assert.ok(p.enhancement_text && p.enhancement_text.length > 0);
    } else {
      assert.ok(p.ratings.length > 0, `${p.board}/${p.fasteners}: not enhancement-only but carries no ratings`);
    }
  }
});

test('a DR-05 pattern\'s transcribed positions add up to its printed fastener count', () => {
  for (const p of dr05.patterns) {
    const total = p.layout.positions.reduce((n, row) => n + row.x_in.length, 0);
    assert.equal(total, p.fasteners,
      `${p.board}/${p.fasteners}: layout carries ${total} transcribed fastener positions`);
  }
});
