// ── The manufacturer's own chart, read for its printed edges ───────────────
// The solve carries a board bill — how many of each letter, how many square
// feet — but not the letter's own printed range, because a range is not a
// quantity a takeoff sums: it is the fact that names the panel. `profile.rs`
// compiles this same file into the core with `include_str!`; this reads the
// identical file so the two can never print different edges for the same
// letter. Nothing here recomputes what letter a thickness earns — that answer
// is `SolvedBoard.letter`, already in the solve.

import profileData from '../../../../data/profiles/versico-versicore-mp-h.json' with { type: 'json' };

interface PanelRow {
  readonly designation: string;
  readonly thin_edge_in: number;
  readonly thick_edge_in: number;
}
interface SlopeRow {
  readonly slope_in_per_ft: number;
  readonly standard: { readonly panels: readonly PanelRow[] } | null;
  readonly extended: { readonly panels: readonly PanelRow[] } | null;
}
interface ProfileFile {
  readonly manufacturer: string;
  readonly product: string;
  readonly document_id: string;
  readonly slopes: readonly SlopeRow[];
}

const DATA = profileData as ProfileFile;

/** The thin and thick edge the chart prints for one letter, or none. */
export interface PanelEdges {
  readonly thinEdgeInches: number;
  readonly thickEdgeInches: number;
}

/**
 * What the chart prints for this letter at this slope and set — or `null`
 * where this writer's own copy of the chart does not carry it, which is a fact
 * worth saying rather than a thickness worth guessing.
 *
 * `documentId` is checked so a bill from a chart this file has never seen does
 * not silently borrow another manufacturer's edges.
 */
export function panelEdges(
  documentId: string,
  slopeInchesPerFoot: number,
  profileSet: 'standard' | 'extended',
  letter: string,
): PanelEdges | null {
  if (documentId !== DATA.document_id) return null;
  const slope = DATA.slopes.find((s) => Math.abs(s.slope_in_per_ft - slopeInchesPerFoot) < 1e-9);
  const set = profileSet === 'standard' ? slope?.standard : slope?.extended;
  const row = set?.panels.find((p) => p.designation === letter);
  if (!row) return null;
  return { thinEdgeInches: row.thin_edge_in, thickEdgeInches: row.thick_edge_in };
}

export const PROFILE_MANUFACTURER = DATA.manufacturer;
export const PROFILE_PRODUCT = DATA.product;
export const PROFILE_DOCUMENT_ID = DATA.document_id;
