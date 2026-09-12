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
  // Not cricket width any more. A cricket's width is half the span between the
  // drains it serves, and NRCA bounds it against the length — it is worked out,
  // not typed. Leaving it here as something to type was modelling an output.
  W: { label: 'Width', hint: 'feet — coping width', group: 'Geometry' },
  T: { label: 'Thickness', hint: 'inches — insulation, as it is sold', group: 'Geometry' },
  // Its own name rather than sharing T. T is what goes on the roof; this is
  // what the roof stops against, and one property meaning two things depending
  // on what kind of condition it sits on is how a number ends up in the wrong
  // place. Left empty it reads pending, which is a true thing to say about a
  // parapet nobody has measured — and is not the same as inventing a width.
  WALL: { label: 'Wall thickness', hint: 'inches — the parapet itself, not what is on the roof', group: 'Geometry' },
  PITCH: { label: 'Pitch', hint: 'rise per 12 — 5 means 5:12', group: 'Geometry' },
  TAPER: { label: 'Taper', hint: 'inches per foot — 1/4 means 1/4" in 12"', group: 'Geometry' },
  ELEV: { label: 'Elevation', hint: 'feet — top of deck on an area, base on a run', group: 'Geometry' },
  SUMP: { label: 'Sump width', hint: 'feet — the square cut down to the drain', group: 'Geometry' },
  BOARDS: { label: 'Boards', hint: 'how many the taper runs over', group: 'Geometry' },
  SIDES: { label: 'Sides', hint: 'how many', group: 'Geometry' },
  STRETCHOUT: { label: 'Girth', hint: 'inches — flat width of coil this profile eats', group: 'Metal' },
};

/**
 * What a condition's role is called. No role is the normal case and shows
 * nothing at all — a parapet is not "no role", it is a parapet.
 */
export const ROLE_LABELS: Readonly<Record<string, string>> = {
  drain: 'Drain',
  ridge: 'Ridge',
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
/**
 * A decimal inch as the trade writes it — 0.25 is a quarter, not "0.25".
 *
 * Sixteenths, because that is as fine as a taper is ever specified and it is
 * what a tape measure reads.
 */
function inchFraction(v: number): string {
  const whole = Math.floor(v);
  const sixteenths = Math.round((v - whole) * 16);
  if (sixteenths === 0) return `${whole}`;
  if (sixteenths === 16) return `${whole + 1}`;
  let n = sixteenths;
  let d = 16;
  while (n % 2 === 0) { n /= 2; d /= 2; }
  return whole === 0 ? `${n}/${d}` : `${whole} ${n}/${d}`;
}

export function propertyPhrase(name: string, value: number): string {
  switch (name) {
    case 'SIDES': return plural(value, 'side');
    case 'PITCH': return `${value}:12 pitch`;
    case 'H': return `${value} ft high`;
    case 'W': return `${value} ft wide`;
    case 'T': return `${value} in thick`;
    case 'STRETCHOUT': return `${value} in girth`;
    // A thickness with no unit beside it is a number nobody can act on, and the
    // default arm prints exactly that. Eight what — inches, or feet of wall?
    case 'WALL': return `${value} in wall`;
    // The same defect one property along. `Taper 0.25` sits between `0.5 in
    // thick` and `14 in girth` and is the only one that does not say what it is
    // measured in — and a taper is the one number on a low-slope roof that
    // decides whether the water leaves. It is inches per foot, and it is
    // written the way a roofer writes it.
    case 'TAPER': return `${inchFraction(value)} in 12`;
    case 'ELEV': return `elevation ${value} ft`;
    case 'SUMP': return `${value} ft sump`;
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
