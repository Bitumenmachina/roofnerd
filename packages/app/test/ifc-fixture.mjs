// ── A made-up solve, shared by every IFC test ──────────────────────────────
// The demo job's own conditions (`c-field`, `c-parapet`, `c-curb`, `c-drains`,
// `c-cricket`) paired with a solve of the real shape but invented geometry —
// C1's own convention (`takeoff.test.mjs` makes up a field rather than
// reaching for a fixture), because nothing in Node can run the Rust solver.
// `ifc-write.test.mjs` and `ifc-ifcx.test.mjs` both read this, so a field
// added to `solved.ts` gets one made-up value here rather than two that could
// drift apart.

import { readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';

export const ROOT = resolve(import.meta.dirname, '../../..');
const DEMO = join(ROOT, 'jobs/demo-job');

/** The manufacturer's chart this fixture's bill claims — read for real in the tests that check it. */
export const PROFILE_DOCUMENT_ID = 'VE-7830'; // data/profiles/versico-versicore-mp-h.json's own document_id

/** The demo job as the shell hands it to a window: one key per file it came from. */
export const demoJob = () => ({
  job: JSON.parse(readFileSync(join(DEMO, 'job.json'), 'utf8')),
  conditions: JSON.parse(readFileSync(join(DEMO, 'conditions.json'), 'utf8')),
  pages: JSON.parse(readFileSync(join(DEMO, 'pages/pages.json'), 'utf8')),
});

/**
 * A solve of the real shape, over a field simple enough to check by hand.
 *
 * Matched to the demo job's own condition ids so `roofPlan`'s accounting pairs
 * each condition with one of these — the geometry inside is invented, because
 * nothing here re-derives it from the trace; only the pairing has to be real.
 * The demo's `c-field` is one tapered field with one drainage region and one
 * governing plane, sloping in x at 1/4:12 off a 1/2 in start with no cap, so
 * `readFacetAt` reads 1/2 in at x=0 and 3 in at x=10.
 */
export function madeUpSolve() {
  const square = { ring: [{ x: 0, y: 0 }, { x: 10, y: 0 }, { x: 10, y: 10 }, { x: 0, y: 10 }], holes: [] };
  const plane = { origin: { x: 0, y: 0 }, direction: { x: 1, y: 0 }, slope: 0.25 };
  const bill = {
    manufacturer: 'Made-up Manufacturer', product: 'Made-up Tapered Panel', documentId: PROFILE_DOCUMENT_ID,
    profileSet: 'extended', slopeInchesPerFoot: 0.25, fillStepInches: 4, panelFeet: 4, staggerFeet: 2,
    byLetter: { X: { boards: 3, squareFeet: 48 }, Y: { boards: 2, squareFeet: 32 }, Z: { boards: 0, squareFeet: 0 } },
    boards: 5, cutBoards: 0, squareFeet: 80, wasteSquareFeet: 0, laidSquareFeet: 80,
    fillBoards: 5, fillSquareFeet: 80, coursesMax: 2, unlettered: 0,
  };
  const region = {
    drain: { x: 0, y: 0 }, shapes: [square], squareFeet: 100,
    axisAlong: { x: 1, y: 0 }, axisAcross: { x: 0, y: 1 }, planes: [plane], capInches: null,
    facets: [{ shape: square, plane: 0, flat: false }],
    valleys: [], ridges: [], noFallSquareFeet: 0, thinInches: 0.5, thickInches: 3,
    boards: [], bill,
  };
  const field = {
    conditionId: 'c-field', trace: 0, ring: square.ring, elevation: 0, startInches: 0.5,
    slopeInchesPerFoot: 0.25, capInches: null, capReason: null,
    axisAlong: { x: 1, y: 0 }, axisAcross: { x: 0, y: 1 }, axisEdge: [square.ring[0], square.ring[1]],
    regions: [region], flow: [], squareFeet: 100, noFallSquareFeet: 0, thinInches: 0.5, thickInches: 3,
    layout: 'four-way', layoutRefused: null, profileSetRefused: null, bill, billRefused: null, sumps: [],
  };

  const sump = {
    conditionId: 'c-drains', trace: 0, at: { x: 0, y: 0 }, sideFeet: 4, fromBowlFeet: 4,
    shapes: [{ ring: [{ x: -2, y: -2 }, { x: 2, y: -2 }, { x: 2, y: 2 }, { x: -2, y: 2 }], holes: [] }],
    squareFeet: 16, clipped: false, rimInches: 1, depthInches: 0.5, panels: 1, panelFeet: 4,
  };

  const cricket = {
    conditionId: 'c-cricket', trace: 0,
    positions: [0, 0, 0, 4, 0, 0, 4, 1, 2], // one triangle, y up
    lengthToWidth: 1.25, maxLengthToWidth: 3, offBy: 0, slopeDisagrees: false,
    planSquareFeet: 8, slopeInchesPerFoot: 0.5, derived: true, fromSolvedSurface: true,
  };

  const taper = {
    solveId: 1, drains: [{ x: 0, y: 0 }], fields: [field], sumps: [sump], crickets: [cricket], refused: [],
  };

  const fullCoping = {
    capWidthFeet: 1.5, capWidthStated: true, girthInches: 26, faceHeightInches: 5, gaugeGa: 24,
    capOuter: { ring: [{ x: -1, y: -0.5 }, { x: 9, y: -0.5 }, { x: 9, y: 0.5 }, { x: -1, y: 0.5 }], holes: [] },
    capInner: { ring: [{ x: -0.7, y: -0.2 }, { x: 8.7, y: -0.2 }, { x: 8.7, y: 0.2 }, { x: -0.7, y: 0.2 }], holes: [] },
    sectionFeet: 10, sectionStated: false, sectionOutOfRange: false,
    cornersOutside: 4, cornersInside: 0, runFeet: 8, piecesEa: 5, cleatFeet: 16,
    fastenOcInches: 12, fastenOcStated: true,
  };
  const noCapCoping = {
    capWidthFeet: null, capWidthStated: false, girthInches: null, faceHeightInches: null, gaugeGa: null,
    capOuter: null, capInner: null, sectionFeet: 10, sectionStated: false, sectionOutOfRange: false,
    cornersOutside: 0, cornersInside: 0, runFeet: 6, piecesEa: 1, cleatFeet: 12,
    fastenOcInches: 12, fastenOcStated: false,
  };

  const parapetRun = {
    conditionId: 'c-parapet', trace: 0, role: null, name: 'Parapet Wall Flashing',
    outer: { ring: [{ x: 0, y: 0 }, { x: 8, y: 0 }, { x: 8, y: 8 }, { x: 0, y: 8 }], holes: [] },
    inboard: { ring: [{ x: 0.7, y: 0.7 }, { x: 7.3, y: 0.7 }, { x: 7.3, y: 7.3 }, { x: 0.7, y: 7.3 }], holes: [] },
    elevationFeet: 0, heightFeet: 3.5, wallInches: 8,
    flashupInches: 24, flashupStated: true, flashMinInches: 24, flashSquareFeet: 60, curbRaiseInches: 0,
    refused: null, coping: fullCoping,
  };
  const curbRun = {
    conditionId: 'c-curb', trace: 0, role: 'curb', name: 'RTU-1 Curb',
    outer: { ring: [{ x: 20, y: 20 }, { x: 24, y: 20 }, { x: 24, y: 24 }, { x: 20, y: 24 }], holes: [] },
    inboard: { ring: [{ x: 20.5, y: 20.5 }, { x: 23.5, y: 20.5 }, { x: 23.5, y: 23.5 }, { x: 20.5, y: 23.5 }], holes: [] },
    elevationFeet: 0, heightFeet: 2, wallInches: 6,
    flashupInches: 18, flashupStated: true, flashMinInches: 17.5, flashSquareFeet: 12, curbRaiseInches: 0.5,
    refused: null, coping: noCapCoping,
  };
  const walls = { solveId: 1, runs: [parapetRun, curbRun] };

  const zones = [
    { zone: '1', label: 'the field', shapes: [square], squareFeet: 60, designPsf: -71, asdPsf: null,
      requiredPsf: -71, fmClass: 90, boards: 60, straddling: 0, thinBoards: 0, fastenersPerBoard: 4,
      pattern: null, fasteners: 240, adheredSquareFeet: 0, adhesiveSets: null, refused: null },
    { zone: '2', label: 'the perimeter', shapes: [square], squareFeet: 30, designPsf: -96, asdPsf: null,
      requiredPsf: -96, fmClass: 120, boards: 30, straddling: 5, thinBoards: 0, fastenersPerBoard: 10,
      pattern: null, fasteners: 300, adheredSquareFeet: 0, adhesiveSets: null, refused: null },
    { zone: '3', label: 'a corner', shapes: [square], squareFeet: 10, designPsf: -122, asdPsf: null,
      requiredPsf: -122, fmClass: 150, boards: 10, straddling: 1, thinBoards: 0, fastenersPerBoard: 10,
      pattern: null, fasteners: 100, adheredSquareFeet: 0, adhesiveSets: null, refused: null },
  ];
  const fastenedField = {
    conditionId: 'c-field', trace: 0, ring: square.ring, squareFeet: 100, standard: 'Carlisle DR-05 2024-07',
    zoneRule: 'ASCE 7-16 Appendix 1', basis: 'strength', basisStated: true, heightFeet: 22,
    leastDimensionFeet: 10, aFeet: 4, aBound: 'floor', zoneOnePrime: 'not drawn',
    baseBoards: 100, straddling: 6, baseBoardInches: 4, thinBoards: 0, boardSize: '4x4',
    productFamily: 'polyiso', membrane: 'TPO', minimumThicknessNote: '', ribbonSpacingInches: 4,
    adhesiveBasis: 'Versico Manual Table VIII', bondingGallons: 20, plates: 60, zones, refused: null,
  };
  const fastening = { solveId: 1, fields: [fastenedField] };

  return { taper, walls, fastening };
}
