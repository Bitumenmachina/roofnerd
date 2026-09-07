// ── The words on screen ────────────────────────────────────────────────────
// A measure is called SF in a formula because a formula is arithmetic and needs
// short names. A person reading a panel is not doing arithmetic, and should
// never be shown `PLAN_SF` or `SIDES 4`.
//
// This file is the one place the two vocabularies meet. Everything an estimator
// reads comes from here.

/** What a measure is called where a person reads it. */
export const MEASURE_LABELS: Readonly<Record<string, string>> = {
  SF: 'Area',
  PLAN_SF: 'Plan area',
  SQ: 'Squares',
  LF: 'Run',
  EA: 'Count',
  VERTICES: 'Corners',
  SEGMENTS: 'Straight pieces',
};

/** What a property is called, and the trade words that say what it is for. */
export const PROPERTY_LABELS: Readonly<Record<string, { label: string; hint: string; group: 'Geometry' | 'Metal' }>> = {
  H: { label: 'Height', hint: 'feet — how far the flashing runs up', group: 'Geometry' },
  W: { label: 'Width', hint: 'feet — coping width, cricket width', group: 'Geometry' },
  T: { label: 'Thickness', hint: 'inches — insulation, as it is sold', group: 'Geometry' },
  PITCH: { label: 'Pitch', hint: 'rise per 12 — 5 means 5:12', group: 'Geometry' },
  SIDES: { label: 'Sides', hint: 'how many', group: 'Geometry' },
  STRETCHOUT: { label: 'Stretch-out', hint: 'inches — flat metal the profile eats', group: 'Metal' },
};

/** What a traced thing is called. */
export const KIND_LABELS: Readonly<Record<string, string>> = {
  area: 'Area',
  line: 'Run',
  count: 'Count',
};

/** "1 trace", "2 traces" — never "trace(s)". */
export const plural = (n: number, one: string, many = `${one}s`): string =>
  `${n.toLocaleString('en-US')} ${n === 1 ? one : many}`;

/** A property as a person would say it: "4 sides", "1.5 ft high". */
export function propertyPhrase(name: string, value: number): string {
  switch (name) {
    case 'SIDES': return plural(value, 'side');
    case 'PITCH': return `${value}:12 pitch`;
    case 'H': return `${value} ft high`;
    case 'W': return `${value} ft wide`;
    case 'T': return `${value} in thick`;
    case 'STRETCHOUT': return `${value} in stretch-out`;
    default: return `${PROPERTY_LABELS[name]?.label ?? name.toLowerCase()} ${value}`;
  }
}

/**
 * A quantity, or a dash.
 *
 * A measure that cannot be worked out yet is a dash with the reason in its
 * tooltip. It is never a coloured word: red says "something is broken", and
 * nothing is broken — the sheet simply has not been scaled yet.
 */
export function quantity(value: number | null | undefined, places = 2): string {
  if (value == null) return '—';
  return value.toLocaleString('en-US', { minimumFractionDigits: places, maximumFractionDigits: places });
}

export const money = (v: number | null | undefined): string =>
  v == null ? '—' : v.toLocaleString('en-US', { style: 'currency', currency: 'USD' });

/** Why a measure has no number yet, for the tooltip on its dash. */
export const PENDING_REASON = 'This sheet has not been scaled yet — set the scale and the number appears.';
