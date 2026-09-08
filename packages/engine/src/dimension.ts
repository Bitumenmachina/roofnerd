// ── Does the unit follow from the formula? ─────────────────────────────────
// A line said `LF * H` and priced it as LF. Feet times feet is an area. The
// wrong unit was displayed and the money was worked out on it.
//
// This is deliberately narrow, and the narrowness is the design rather than a
// shortcoming. Every estimating system worth copying — Edge, Sage, PlanSwift —
// declares the unit on the item and never checks it against the arithmetic,
// because in those systems the "formula" is a coverage factor (`Q`, `Q/200`)
// with nothing in it to check. roofnerd can check, and only because it does the
// thing Edge hides: the formula is a visible expression over named measures.
//
// So: the declared unit stays authoritative and is never overridden. Where the
// expression is made **entirely** of known measures, its dimension is knowable
// and worth saying out loud when it disagrees. Where a bare number sits in a
// multiplying or dividing position, it is silent — because that number carries
// a dimension nobody can recover. The handoff's own example says why:
//
//     ceil(LF * STRETCHOUT / 12 / 30)
//
// The 30 is square feet per sheet. The expression looks like an area and is a
// count of sheets, and no amount of analysis will ever know that. Staying quiet
// there is the correct answer, not a gap.

import type { Node } from './formula.js';

/**
 * How many lengths a measure is made of. Two for a surface, one for a run,
 * none for a count or a ratio.
 *
 * Inches and feet are both lengths and are not told apart here. `LF * T` is an
 * area in feet-inches, which this reports as an area — the dimension is right
 * and the scale is the estimator's business, the way it is in every tool they
 * already use.
 */
const LENGTHS: Readonly<Record<string, number>> = {
  SF: 2, PLAN_SF: 2, SQ: 2,
  LF: 1, H: 1, W: 1, ELEV: 1, SUMP: 1, T: 1, STRETCHOUT: 1,
  EA: 0, VERTICES: 0, SEGMENTS: 0, SIDES: 0, BOARDS: 0, PITCH: 0, TAPER: 0,
};

/** What a declared unit is made of. */
const UNIT_LENGTHS: Readonly<Record<string, number>> = { SF: 2, SQ: 2, LF: 1, EA: 0 };

/** Functions that hand back whatever they were given. */
const KEEPS = new Set(['ceil', 'floor', 'round', 'abs', 'max', 'min']);

/** `null` means "not decidable, and saying nothing is the right answer". */
function dimensionOf(node: Node): number | null {
  switch (node.kind) {
    case 'number':
      return 0;
    case 'name':
      return LENGTHS[node.name] ?? null;
    case 'unary':
      return dimensionOf(node.operand);
    case 'binary': {
      const left = dimensionOf(node.left);
      const right = dimensionOf(node.right);
      if (left === null || right === null) return null;
      // A bare number multiplying or dividing is a dimensioned constant in
      // disguise — the square feet in a sheet, the inches in a foot. Out of
      // scope on purpose.
      if (node.op === '*' || node.op === '/') {
        if (node.left.kind === 'number' || node.right.kind === 'number') return null;
        return node.op === '*' ? left + right : left - right;
      }
      // Adding a run to a surface has no single dimension to report.
      return left === right ? left : null;
    }
    case 'call': {
      if (!KEEPS.has(node.name)) return null;
      const parts = node.args.map(dimensionOf);
      if (parts.some((p) => p === null)) return null;
      return parts.every((p) => p === parts[0]) ? parts[0]! : null;
    }
    default:
      return null;
  }
}

/** What to call a dimension where a person reads it. */
const NAMES: Readonly<Record<number, string>> = { 0: 'a count', 1: 'a length', 2: 'an area' };

/**
 * Does this expression's dimension disagree with the unit the item declares?
 *
 * Returns the sentence to show on the line, or `null` for "nothing to say" —
 * which covers both agreement and the whole out-of-scope territory above.
 */
export function unitDisagreement(tree: Node, declaredUnit: string): string | null {
  const declared = UNIT_LENGTHS[declaredUnit];
  if (declared === undefined) return null;
  const found = dimensionOf(tree);
  if (found === null || found === declared) return null;
  const got = NAMES[found] ?? `a length to the power of ${found}`;
  const want = NAMES[declared] ?? declaredUnit;
  return `This works out to ${got}, and the line is priced as ${declaredUnit} — ${want}.`;
}
