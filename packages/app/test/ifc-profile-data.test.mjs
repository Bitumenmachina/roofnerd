// ── The chart's own edges, read and not typed in ───────────────────────────
// `profile.rs` compiles this same file into the core with `include_str!`; this
// reads the identical file off disk so a letter's printed range can never
// drift between the solver and the writer. Nothing here is a fixture value —
// every figure asserted below is copied out of `data/profiles/…json` itself,
// which is public manufacturer data and not a client's.
//
//   node --test test/ifc-profile-data.test.mjs

import './typescript.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';

const { panelEdges, PROFILE_DOCUMENT_ID } = await import('../src/ifc/profile-data.ts');

const ROOT = resolve(import.meta.dirname, '../../..');
const RAW = JSON.parse(readFileSync(join(ROOT, 'data/profiles/versico-versicore-mp-h.json'), 'utf8'));

test('the document id matches the file on disk', () => {
  assert.equal(PROFILE_DOCUMENT_ID, RAW.document_id);
});

test('every panel the extended 1/4 : 12 chart prints comes back exactly', () => {
  const slope = RAW.slopes.find((s) => s.slope_in_per_ft === 0.25);
  for (const row of slope.extended.panels) {
    const got = panelEdges(RAW.document_id, 0.25, 'extended', row.designation);
    assert.deepEqual(got, { thinEdgeInches: row.thin_edge_in, thickEdgeInches: row.thick_edge_in },
      `letter ${row.designation}`);
  }
});

test('a letter the chart does not print at this slope is null, not a guess', () => {
  assert.equal(panelEdges(RAW.document_id, 0.25, 'extended', 'ZZZZ'), null);
});

test('a document id this file has never seen borrows nothing', () => {
  assert.equal(panelEdges('some-other-manufacturer-sheet', 0.25, 'extended', 'X'), null);
});

test('a slope the chart does not print is null', () => {
  assert.equal(panelEdges(RAW.document_id, 99, 'extended', 'X'), null);
});
