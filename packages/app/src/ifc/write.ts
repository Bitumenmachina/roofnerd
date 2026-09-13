// ── The solved roof, written as an IFC4 model ──────────────────────────────
// Not the job's raw traces re-measured: the SOLVE. `roofPlan` reads what
// `solve_get` already worked out — the tapered package's regions and facets,
// the board bill by letter, the wall and curb solids with their coping, the
// wind zones and fastener counts, the sumps and the crickets — and answers
// what of it can be stood up and what cannot, with no wasm and no window
// anywhere near it. `writeRoof` turns that answer into bytes. The split is
// what lets a test in Node read the bytes back, and what keeps this file from
// ever computing a quantity the core did not already produce: a slab's area is
// `field.squareFeet`, never a sum taken over its own triangles.
//
// Two solved things this file does not carry a third arithmetic for:
//   - a facet's own depth at a corner is `readFacetAt`, the one expression
//     `taper.rs`'s `read_at` is pinned to (D148) — the same evaluation the
//     Model view already stands the roof up with;
//   - a letter's printed thin and thick edge is read off the manufacturer's
//     own chart (`profile-data.ts`), the file `profile.rs` compiles into the
//     core, because a printed range is not a quantity the bill sums and so it
//     is not in `SolvedBoardBill` — but it is not invented here either.
//
// Where a number has not been stated or has not been solved, the piece is left
// out and the plan says so, by name — an IfcSlab with a thickness nobody
// stated would be a picture of a roof rather than this roof.
//
// Units are FEET (D144). IFC does not require metres — it requires the file to
// say what its numbers mean, which `IfcConversionBasedUnit` does — and a roof
// written in feet carries the sheet's own figures: the square feet in the
// model are the square feet on the estimate, with no conversion standing
// between the two where a rounding rule could get in.
//
// Geometry conventions, the Model editor's own:
//   plan (x, y) becomes IFC (x, -y, z)   — plan y grows down the sheet, IFC +Y
//                                          is north, so the sheet keeps its
//                                          orientation and north stays north
//   a build-up grows UP from the deck    — `GROWS_UP_FROM_DECK` in the engine
//   a parapet stands INBOARD of its run  — the traced line is the outside face
//
// Every profile is written in plan feet with the solid sitting at the origin
// and the extrusion straight up, so no rotation is carried anywhere: a corner
// in the file is at the coordinate it was traced at. An element's own
// placement holds its elevation, which is the one thing the storey cannot say
// for it.

import type {
  Condition, Item, Page, Point,
  SolvedBoardBill, SolvedCoping, SolvedCricket, SolvedFastenedField, SolvedFastening,
  SolvedSump, SolvedTaper, SolvedTaperField, SolvedWallRun, SolvedWalls, Spot,
} from '@roofnerd/engine';
import { cricketOf, fastenedOf, fieldOf, readFacetAt, wallOf } from '@roofnerd/engine';
import * as WEBIFC from 'web-ifc';
import { feet, feetPerUnit } from '../scale.js';
import type { Doc } from '../doc.js';
import { panelEdges } from './profile-data.js';
import { triangulateShape } from './triangulate.js';

// ── Part 1: what of this job can be stood up ───────────────────────────────

/** The roof surface itself, one per solved tapered field. */
export interface SlabPlan {
  readonly conditionId: string;
  readonly trace: number;
  readonly name: string;
  readonly costCode: string;
  readonly field: SolvedTaperField;
}

/** One insulation covering: a lettered course, the flat fill, cover board or membrane. */
export interface CoveringPlan {
  readonly conditionId: string;
  readonly name: string;
  readonly costCode: string;
  readonly role: 'insulation-letter' | 'insulation-fill' | 'cover-board' | 'membrane';
  readonly squareFeet: number;
  readonly boards?: number;
  readonly letter?: string;
  readonly thicknessInches?: number;
  readonly thinEdgeInches?: number;
  readonly thickEdgeInches?: number;
  readonly manufacturer?: string;
  readonly product?: string;
  readonly documentId?: string;
  readonly item?: Item;
}

/** One straight length of a parapet run or a curb, ready to be an IfcWall. */
export interface WallPlan {
  readonly conditionId: string;
  readonly trace: number;
  readonly name: string;
  readonly costCode: string;
  /**
   * A curb, rather than the edge of the roof.
   *
   * Both are a run with a height and both stand up the same way, so both are
   * an IfcWall — but IFC4 has a type for a parapet and calling a curb one
   * would tell an architect's program that the box round the unit is the
   * edge of the building. `SOLIDWALL` is what is left when the specific type
   * is wrong.
   */
  readonly curb: boolean;
  readonly run: SolvedWallRun;
}

/** The coping cap over a wall or curb run, when it has one to draw. */
export interface CopingPlan {
  readonly conditionId: string;
  readonly trace: number;
  readonly name: string;
  readonly costCode: string;
  readonly coping: SolvedCoping;
  readonly elevationFeet: number;
}

/** One drain, with its sump where the roof has one. */
export interface DrainPlan {
  readonly conditionId: string;
  readonly name: string;
  readonly costCode: string;
  readonly at: Spot;
  readonly sump: SolvedSump | null;
}

/** A cricket behind a curb, with the ridge it is traced as. */
export interface CricketPlan {
  readonly conditionId: string;
  readonly trace: number;
  readonly name: string;
  readonly costCode: string;
  readonly cricket: SolvedCricket;
  readonly ridge: readonly [Point, Point] | null;
}

/** The wind zones and fastener counts for one tapered field. */
export interface FasteningPlan {
  readonly conditionId: string;
  readonly trace: number;
  readonly field: SolvedFastenedField;
}

/** Something the job carries that is not standing up, and why not. */
export interface LeftOut {
  readonly name: string;
  readonly why: string;
}

export interface RoofPlan {
  readonly job: string;
  readonly slabs: readonly SlabPlan[];
  readonly coverings: readonly CoveringPlan[];
  readonly walls: readonly WallPlan[];
  readonly copings: readonly CopingPlan[];
  readonly drains: readonly DrainPlan[];
  readonly crickets: readonly CricketPlan[];
  readonly fastening: readonly FasteningPlan[];
  readonly leftOut: readonly LeftOut[];
}

const conditionsOf = (d: Doc): Condition[] => (d['conditions'] as Condition[]) ?? [];
const pagesOf = (d: Doc): Page[] => (d['pages'] as Page[]) ?? [];
const jobName = (d: Doc): string => {
  const job = d['job'] as { name?: unknown } | undefined;
  return typeof job?.name === 'string' && job.name ? job.name : 'Untitled';
};

/** The letters and the flat fill course, as insulation coverings. */
function insulationCoverings(
  name: string, conditionId: string, costCode: string, bill: SolvedBoardBill,
): CoveringPlan[] {
  const out: CoveringPlan[] = [];
  for (const [letter, row] of Object.entries(bill.byLetter)) {
    if (row.boards <= 0) continue;
    const edges = panelEdges(bill.documentId, bill.slopeInchesPerFoot, bill.profileSet, letter);
    out.push({
      conditionId, name: `${name} — tapered ${letter}`, costCode, role: 'insulation-letter',
      squareFeet: row.squareFeet, boards: row.boards, letter,
      thinEdgeInches: edges?.thinEdgeInches, thickEdgeInches: edges?.thickEdgeInches,
      manufacturer: bill.manufacturer, product: bill.product, documentId: bill.documentId,
    });
  }
  if (bill.fillBoards > 0) {
    out.push({
      conditionId, name: `${name} — flat fill`, costCode, role: 'insulation-fill',
      squareFeet: bill.fillSquareFeet, boards: bill.fillBoards,
      thicknessInches: bill.fillStepInches,
      manufacturer: bill.manufacturer, product: bill.product, documentId: bill.documentId,
    });
  }
  return out;
}

/** Cover board and membrane, off the condition's own layered items. */
function layeredCoverings(
  name: string, conditionId: string, items: readonly Item[], squareFeet: number, leftOut: LeftOut[],
): CoveringPlan[] {
  const out: CoveringPlan[] = [];
  for (const item of items) {
    if (item.layer !== 'cover board' && item.layer !== 'membrane') continue;
    if (item.thickness === undefined) {
      leftOut.push({ name: `${name} — ${item.layer}`, why: `nothing says how thick ${item.description} is` });
      continue;
    }
    out.push({
      conditionId, name: `${name} — ${item.layer}`, costCode: item.costCode,
      role: item.layer === 'cover board' ? 'cover-board' : 'membrane',
      squareFeet, thicknessInches: item.thickness, item,
    });
  }
  return out;
}

/**
 * What of this job can be stood up, and what cannot.
 *
 * Pure: the job and the three solves in, a description out, no wasm and no
 * window. Every non-hidden condition lands in one of the six plan lists or in
 * `leftOut` at least once, so nothing goes missing quietly.
 */
export function roofPlan(d: Doc, taper: SolvedTaper, walls: SolvedWalls, fastening: SolvedFastening): RoofPlan {
  const list = conditionsOf(d);
  const scales = feetPerUnit(pagesOf(d));
  const slabs: SlabPlan[] = [];
  const coverings: CoveringPlan[] = [];
  const wallPlans: WallPlan[] = [];
  const copings: CopingPlan[] = [];
  const drains: DrainPlan[] = [];
  const crickets: CricketPlan[] = [];
  const fasteningPlans: FasteningPlan[] = [];
  const leftOut: LeftOut[] = [];

  const refusalFor = (conditionId: string, trace: number) =>
    taper.refused.find((r) => r.conditionId === conditionId && r.trace === trace)?.why ?? null;

  for (const c of list) {
    if (c.hidden) continue;
    const name = c.name ?? c.id;
    const costCode = (c.items ?? [])[0]?.costCode ?? '';
    const drawn = (c.traces ?? []).map((t) => feet(t, scales));

    if (c.kind === 'area') {
      let any = false;
      drawn.forEach((ring, trace) => {
        if (ring.length === 0) {
          leftOut.push({ name, why: 'the sheet it was traced on has no scale yet' });
          return;
        }
        const field = fieldOf(taper, c.id, trace);
        if (!field) {
          leftOut.push({ name, why: refusalFor(c.id, trace) ?? 'the tapered layout has not solved this field' });
          return;
        }
        any = true;
        slabs.push({ conditionId: c.id, trace, name, costCode, field });
        if (field.bill) coverings.push(...insulationCoverings(name, c.id, costCode, field.bill));
        else leftOut.push({ name: `${name} — insulation`, why: field.billRefused ?? 'the board bill did not solve' });
        coverings.push(...layeredCoverings(name, c.id, c.items ?? [], field.squareFeet, leftOut));
        const fastened = fastenedOf(fastening, c.id, trace);
        if (fastened) fasteningPlans.push({ conditionId: c.id, trace, field: fastened });
      });
      if (!any && drawn.length === 0) leftOut.push({ name, why: 'the sheet it was traced on has no scale yet' });
      continue;
    }

    if (c.kind === 'line' && c.role === 'ridge') {
      drawn.forEach((ring, trace) => {
        if (ring.length === 0) {
          leftOut.push({ name, why: 'the sheet it was traced on has no scale yet' });
          return;
        }
        const cricket = cricketOf(taper, c.id, trace);
        if (!cricket) {
          leftOut.push({ name, why: refusalFor(c.id, trace) ?? 'the cricket has not solved a saddle behind this ridge' });
          return;
        }
        const first = ring[0]!;
        const last = ring[ring.length - 1]!;
        crickets.push({ conditionId: c.id, trace, name, costCode, cricket, ridge: [first, last] });
      });
      continue;
    }

    if (c.kind === 'line') {
      drawn.forEach((ring, trace) => {
        if (ring.length === 0) {
          leftOut.push({ name, why: 'the sheet it was traced on has no scale yet' });
          return;
        }
        const run = wallOf(walls, c.id, trace);
        if (!run) {
          leftOut.push({ name, why: 'the wall solver has not solved this run' });
          return;
        }
        if (run.wallInches === null || !run.inboard) {
          leftOut.push({ name, why: 'nothing says how thick the wall is' });
        } else {
          wallPlans.push({ conditionId: c.id, trace, name, costCode, curb: c.role === 'curb', run });
        }
        if (run.coping.capOuter && run.coping.capInner) {
          copings.push({
            conditionId: c.id, trace, name: `${name} — coping`, costCode,
            coping: run.coping, elevationFeet: run.elevationFeet + run.heightFeet,
          });
        } else {
          leftOut.push({ name: `${name} — coping`, why: 'the cap has nothing to centre on' });
        }
      });
      continue;
    }

    if (c.kind === 'count' && c.role === 'drain') {
      const sumps = taper.sumps.filter((s) => s.conditionId === c.id);
      let any = false;
      drawn.forEach((ring) => {
        ring.forEach((at, i) => {
          any = true;
          drains.push({ conditionId: c.id, name, costCode, at, sump: sumps[i] ?? null });
        });
      });
      if (!any) leftOut.push({ name, why: 'the sheet it was traced on has no scale yet' });
      continue;
    }

    leftOut.push({ name, why: 'a count is a fitting, and fittings are not stood up yet' });
  }

  return {
    job: jobName(d), slabs, coverings, walls: wallPlans, copings, drains, crickets,
    fastening: fasteningPlans, leftOut,
  };
}

// ── Part 2: the plan as IFC4 bytes ─────────────────────────────────────────

/** Inches to feet: thicknesses and heights arrive from the solve stated in inches. */
const inchesToFeet = (n: number) => n / 12;

/**
 * The foot, as a definition rather than a measurement: the international foot
 * has been exactly this many metres since 1959.
 *
 * It is written into the file as the conversion factor on the length unit. It
 * is also what the drawing on screen is scaled by, and that is not the same
 * statement: the fragments loader normalises every model it reads to metres,
 * whatever the file's own unit says, so the roof in the toolkit's world is
 * this many three.js units to the foot. The file keeps the trade's numbers;
 * the view keeps the toolkit's. Anything measured off the view crosses back
 * here, once.
 */
export const METRES_PER_FOOT = 0.3048;

/** A property's value, wrapped in whichever primitive IFC type its own JS type asks for. */
export type PropValue = string | number | boolean;

/** One quantity to carry in a Qto set — which physical measure it is decides which class wraps it. */
export interface QtyEntry {
  readonly name: string;
  readonly kind: 'area' | 'length' | 'count' | 'volume';
  readonly value: number;
}

/**
 * Where a number came from and how firm it is — buildingSMART's own shape for
 * the property set, `src`/`hier`/`constraint`/`overridable`, and the reason the
 * vocabulary gate reads the word "authority" nowhere else in this program
 * (D159).
 */
export interface AuthorityInfo {
  readonly src: string;
  readonly hier: 'CODE' | 'MFR' | 'FIELD';
  readonly constraint: string;
  readonly overridable: boolean;
}

/** Which line on the sheet an element buckets to, and how much of it. */
export interface CostAxisInfo {
  readonly division: string;
  readonly code: string;
  readonly quantity: number;
  readonly unit: string;
}

/** A CSI cost code's own division: the digits before its first dash. */
const csiDivision = (code: string): string => code.split('-')[0] ?? '';

/** A plan point in three feet — x and the plan's own y, plus a height. */
export interface Point3 { readonly x: number; readonly y: number; readonly z: number; }

/**
 * The job as IFC4 bytes.
 *
 * `api` has to have been through `Init()` already — where the wasm came from is
 * the caller's business, and it differs between the window and a test, which is
 * exactly why this function does not decide it.
 */
export function writeIfc(
  d: Doc, taper: SolvedTaper, walls: SolvedWalls, fastening: SolvedFastening, api: WEBIFC.IfcAPI,
): Uint8Array {
  return writeRoof(roofPlan(d, taper, walls, fastening), api);
}

/**
 * The plan as IFC4 bytes.
 *
 * `api` has to have been through `Init()` already — where the wasm came from is
 * the caller's business, and it differs between the window and a test, which is
 * exactly why this function does not decide it.
 */
export function writeRoof(plan: RoofPlan, api: WEBIFC.IfcAPI): Uint8Array {
  const { IFC4 } = WEBIFC;
  const model = api.CreateModel({ schema: WEBIFC.Schemas.IFC4, name: `${plan.job}.ifc` });

  const length = (v: number) => new IFC4.IfcLengthMeasure(v);
  const ratio = (v: number) => new IFC4.IfcReal(v);
  const label = (v: string) => new IFC4.IfcLabel(v);
  const identifier = (v: string) => new IFC4.IfcIdentifier(v);
  const point = (x: number, y: number, z: number) => new IFC4.IfcCartesianPoint([length(x), length(y), length(z)]);
  const direction = (x: number, y: number, z: number) => new IFC4.IfcDirection([ratio(x), ratio(y), ratio(z)]);
  const placement = (x: number, y: number, z: number) => new IFC4.IfcAxis2Placement3D(point(x, y, z), null, null);

  // ── units ───────────────────────────────────────────────────────────────
  // The foot, stated against the metre the way IFC states any non-SI unit.
  const metre = new IFC4.IfcSIUnit(IFC4.IfcUnitEnum.LENGTHUNIT, null, IFC4.IfcSIUnitName.METRE);
  const squareMetre = new IFC4.IfcSIUnit(IFC4.IfcUnitEnum.AREAUNIT, null, IFC4.IfcSIUnitName.SQUARE_METRE);
  const cubicMetre = new IFC4.IfcSIUnit(IFC4.IfcUnitEnum.VOLUMEUNIT, null, IFC4.IfcSIUnitName.CUBIC_METRE);
  const exponents = (l: number) => new IFC4.IfcDimensionalExponents(l, 0, 0, 0, 0, 0, 0);
  const foot = new IFC4.IfcConversionBasedUnit(
    exponents(1), IFC4.IfcUnitEnum.LENGTHUNIT, label('FOOT'),
    new IFC4.IfcMeasureWithUnit(length(METRES_PER_FOOT), metre),
  );
  const squareFoot = new IFC4.IfcConversionBasedUnit(
    exponents(2), IFC4.IfcUnitEnum.AREAUNIT, label('SQUARE FOOT'),
    new IFC4.IfcMeasureWithUnit(new IFC4.IfcAreaMeasure(METRES_PER_FOOT ** 2), squareMetre),
  );
  const cubicFoot = new IFC4.IfcConversionBasedUnit(
    exponents(3), IFC4.IfcUnitEnum.VOLUMEUNIT, label('CUBIC FOOT'),
    new IFC4.IfcMeasureWithUnit(new IFC4.IfcVolumeMeasure(METRES_PER_FOOT ** 3), cubicMetre),
  );
  const units = new IFC4.IfcUnitAssignment([
    foot, squareFoot, cubicFoot,
    new IFC4.IfcSIUnit(IFC4.IfcUnitEnum.PLANEANGLEUNIT, null, IFC4.IfcSIUnitName.RADIAN),
  ]);

  // ── the spatial structure ───────────────────────────────────────────────
  const context = new IFC4.IfcGeometricRepresentationContext(
    null, label('Model'), new IFC4.IfcDimensionCount(3), ratio(1e-5), placement(0, 0, 0), null,
  );
  const project = new IFC4.IfcProject(
    guid(`project:${plan.job}`), null, label(plan.job), null, null, null, null, [context], units,
  );
  const sitePlacement = new IFC4.IfcLocalPlacement(null, placement(0, 0, 0));
  const site = new IFC4.IfcSite(
    guid(`site:${plan.job}`), null, label('Site'), null, null, sitePlacement, null, null,
    IFC4.IfcElementCompositionEnum.ELEMENT, null, null, null, null, null,
  );
  const buildingPlacement = new IFC4.IfcLocalPlacement(sitePlacement, placement(0, 0, 0));
  const building = new IFC4.IfcBuilding(
    guid(`building:${plan.job}`), null, label(plan.job), null, null, buildingPlacement, null, null,
    IFC4.IfcElementCompositionEnum.ELEMENT, null, null, null,
  );
  const storeyPlacement = new IFC4.IfcLocalPlacement(buildingPlacement, placement(0, 0, 0));
  const storey = new IFC4.IfcBuildingStorey(
    guid(`storey:${plan.job}`), null, label('Roof'), null, null, storeyPlacement, null, null,
    IFC4.IfcElementCompositionEnum.ELEMENT, length(0),
  );

  const lines: WEBIFC.IfcLineObject[] = [project, site, building, storey];
  const elements: WEBIFC.IFC4.IfcProduct[] = [];
  const push = (...items: WEBIFC.IfcLineObject[]) => { lines.push(...items); };

  // ── plan (x, y) becomes IFC (x, -y): the sheet keeps its own orientation ──
  const toIfc = (p: { x: number; y: number }) => ({ x: p.x, y: -p.y });

  /** Twice the signed area of a plan ring: positive counter-clockwise. */
  const signedArea = (ring: readonly { x: number; y: number }[]): number => {
    let sum = 0;
    for (let i = 0; i < ring.length; i += 1) {
      const a = ring[i]!; const b = ring[(i + 1) % ring.length]!;
      sum += a.x * b.y - b.x * a.y;
    }
    return sum;
  };

  /** A ring wound the way asked — CCW for a profile's outer edge, CW for a void. */
  const wound = (ring: readonly Point[], ccw: boolean): Point[] =>
    (signedArea(ring) > 0) === ccw ? ring.slice() : ring.slice().reverse();

  /** A closed polyline in the XY plane at z = 0, from plan points. */
  const polyline = (ring: readonly Point[]) => {
    const closed = [...ring, ring[0]!].map(toIfc);
    return new IFC4.IfcPolyline(closed.map((p) => point(p.x, p.y, 0)));
  };

  /** A profile with one hole in it — a wall's outer trace and the line `WALL` grows in from. */
  const profileWithVoid = (name: string, outer: readonly Point[], inner: readonly Point[]) =>
    new IFC4.IfcArbitraryProfileDefWithVoids(
      IFC4.IfcProfileTypeEnum.AREA, label(name), polyline(wound(outer, true)), [polyline(wound(inner, false))],
    );

  const extrudedBody = (area: WEBIFC.IFC4.IfcProfileDef, depth: number) =>
    new IFC4.IfcProductDefinitionShape(null, null, [
      new IFC4.IfcShapeRepresentation(context, label('Body'), label('SweptSolid'), [
        new IFC4.IfcExtrudedAreaSolid(
          area, placement(0, 0, 0), direction(0, 0, 1), new IFC4.IfcPositiveLengthMeasure(depth),
        ),
      ]),
    ]);

  /**
   * A tessellated face set from triangles already in IFC feet — the roof
   * surface and the crickets, the two shapes here with no single extrusion
   * depth to sweep. Vertices that coincide (to the nearest thousandth of a
   * foot — a fraction of a hundredth of an inch, well under anything `geo`'s
   * own grid can tell apart) share one entry in the coordinate list, which is
   * what makes the mesh a set of faces sharing edges rather than one
   * disconnected triangle per facet.
   */
  const tessellatedBody = (triangles: readonly (readonly [Point3, Point3, Point3])[]) => {
    if (triangles.length === 0) return null;
    const coords: number[][] = [];
    const index = new Map<string, number>();
    const at = (p: Point3): number => {
      const key = `${p.x.toFixed(3)},${p.y.toFixed(3)},${p.z.toFixed(3)}`;
      const found = index.get(key);
      if (found !== undefined) return found + 1; // IFC's own indices are 1-based
      coords.push([p.x, p.y, p.z]);
      index.set(key, coords.length - 1);
      return coords.length;
    };
    const coordIndex = triangles.map(([a, b, c]) => [
      new IFC4.IfcPositiveInteger(at(a)), new IFC4.IfcPositiveInteger(at(b)), new IFC4.IfcPositiveInteger(at(c)),
    ]);
    const points = new IFC4.IfcCartesianPointList3D(coords.map((c) => c.map((v) => length(v))));
    const faceSet = new IFC4.IfcTriangulatedFaceSet(points, null, new IFC4.IfcBoolean(true), coordIndex, null);
    return new IFC4.IfcProductDefinitionShape(null, null, [
      new IFC4.IfcShapeRepresentation(context, label('Body'), label('Tessellation'), [faceSet]),
    ]);
  };

  // ── property sets and quantities: Pset_CostAxis, Pset_TimeAxis, Pset_Authority ──
  // The one place in this program the word "authority" is allowed to appear
  // (`tools/gate-language.sh`, C7): it is buildingSMART's own name for a
  // property set, the shorthand the operator's own cross-project convention
  // gives it, and not this program's governance language.

  const singleValue = (v: PropValue) => {
    if (typeof v === 'string') return new IFC4.IfcText(v);
    if (typeof v === 'boolean') return new IFC4.IfcBoolean(v);
    return new IFC4.IfcReal(v);
  };

  /** One property set, and the relationship that puts it on an element. */
  const pset = (key: string, name: string, entries: readonly (readonly [string, PropValue])[], on: WEBIFC.IFC4.IfcObjectDefinition) => {
    const props = entries.map(([n, v]) => new IFC4.IfcPropertySingleValue(identifier(n), null, singleValue(v), null));
    const set = new IFC4.IfcPropertySet(guid(`${key}:set`), null, label(name), null, props);
    push(set, new IFC4.IfcRelDefinesByProperties(guid(`${key}:rel`), null, null, null, [on], set));
  };

  const quantityOf = (q: QtyEntry) => {
    if (q.kind === 'area') return new IFC4.IfcQuantityArea(label(q.name), null, null, new IFC4.IfcAreaMeasure(q.value), null);
    if (q.kind === 'volume') return new IFC4.IfcQuantityVolume(label(q.name), null, null, new IFC4.IfcVolumeMeasure(q.value), null);
    if (q.kind === 'count') return new IFC4.IfcQuantityCount(label(q.name), null, null, new IFC4.IfcCountMeasure(q.value), null);
    return new IFC4.IfcQuantityLength(label(q.name), null, null, length(q.value), null);
  };

  /** One Qto set, and the relationship that puts it on an element. */
  const qto = (key: string, name: string, entries: readonly QtyEntry[], on: WEBIFC.IFC4.IfcObjectDefinition) => {
    const set = new IFC4.IfcElementQuantity(guid(`${key}:qto`), null, label(name), null, null, entries.map(quantityOf));
    push(set, new IFC4.IfcRelDefinesByProperties(guid(`${key}:qtorel`), null, null, null, [on], set));
  };

  /** The three axes every element in this file carries — cost, time, authority. */
  const axes = (key: string, on: WEBIFC.IFC4.IfcObjectDefinition, cost: CostAxisInfo, authority: AuthorityInfo) => {
    pset(`${key}:cost`, 'Pset_CostAxis', [
      ['division', cost.division], ['code', cost.code], ['quantity', cost.quantity], ['unit', cost.unit],
    ], on);
    // Nothing in this job states a schedule — no phase, no task, no date. The
    // one true thing every element here can say is which side of the work it
    // is on: everything this writer stands up is the proposed roof, never the
    // existing one being torn off, and that is a fact about the export and not
    // a number invented to fill a field nobody asked for.
    pset(`${key}:time`, 'Pset_TimeAxis', [['phase', 'new work']], on);
    pset(`${key}:authority`, 'Pset_Authority', [
      ['src', authority.src], ['hier', authority.hier],
      ['constraint', authority.constraint], ['overridable', authority.overridable],
    ], on);
  };

  // ── the roof slabs: one per solved tapered field, its facets triangulated ──
  for (const [n, slab] of plan.slabs.entries()) {
    const field = slab.field;
    const triangles: [Point3, Point3, Point3][] = [];
    for (const region of field.regions) {
      for (const facet of region.facets) {
        for (const [a, b, c] of triangulateShape(facet.shape)) {
          const at3 = (p: { x: number; y: number }): Point3 => {
            const q = toIfc(p);
            return { x: q.x, y: q.y, z: inchesToFeet(readFacetAt(field, region, facet, p)) };
          };
          triangles.push([at3(a), at3(b), at3(c)]);
        }
      }
    }
    const body = tessellatedBody(triangles);
    if (!body) {
      // Every facet failed to triangulate — a field with a ring `geo` handed
      // back too small to earn even one triangle. Recorded rather than
      // written as a slab with nothing in it.
      continue;
    }
    const key = `slab:${slab.conditionId}:${n}`;
    const element = new IFC4.IfcSlab(
      guid(key), null, label(slab.name), null, null,
      new IFC4.IfcLocalPlacement(storeyPlacement, placement(0, 0, field.elevation)),
      body, identifier(slab.conditionId), IFC4.IfcSlabTypeEnum.ROOF,
    );
    push(element);
    elements.push(element);
    qto(key, 'Qto_SlabBaseQuantities', [
      { name: 'NetArea', kind: 'area', value: field.squareFeet },
      { name: 'ThinEdge', kind: 'length', value: inchesToFeet(field.thinInches ?? 0) },
      { name: 'ThickEdge', kind: 'length', value: inchesToFeet(field.thickInches ?? 0) },
      {
        name: 'NetVolume', kind: 'volume',
        value: triangles.reduce((sum, [a, b, c]) => {
          const area2 = Math.abs((b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y));
          return sum + (area2 / 2) * ((a.z + b.z + c.z) / 3);
        }, 0),
      },
    ], element);
    axes(key, element, { division: csiDivision(slab.costCode), code: slab.costCode, quantity: field.squareFeet, unit: 'SF' }, {
      src: 'the roof plan trace and the job’s typed T, TAPER and BOARDS',
      hier: 'FIELD',
      constraint: 'NRCA Membrane Roof Systems 2019 pp.162-169 (D149)',
      overridable: true,
    });
  }

  // ── the coverings: insulation by letter, the flat fill, cover board, membrane ──
  for (const [n, covering] of plan.coverings.entries()) {
    const key = `covering:${covering.conditionId}:${covering.role}:${covering.letter ?? n}`;
    const predefined = covering.role === 'membrane' ? IFC4.IfcCoveringTypeEnum.MEMBRANE
      : IFC4.IfcCoveringTypeEnum.INSULATION;
    const element = new IFC4.IfcCovering(
      guid(key), null, label(covering.name), null, null,
      new IFC4.IfcLocalPlacement(storeyPlacement, placement(0, 0, 0)), null,
      identifier(covering.conditionId), predefined,
    );
    push(element);
    elements.push(element);
    const quantities: QtyEntry[] = [{ name: 'NetArea', kind: 'area', value: covering.squareFeet }];
    if (covering.boards !== undefined) quantities.push({ name: 'BoardCount', kind: 'count', value: covering.boards });
    if (covering.thicknessInches !== undefined) {
      quantities.push({ name: 'Width', kind: 'length', value: inchesToFeet(covering.thicknessInches) });
    }
    qto(key, 'Qto_CoveringBaseQuantities', quantities, element);

    if (covering.role === 'insulation-letter' || covering.role === 'insulation-fill') {
      pset(key, 'InsulationPanel', [
        ['letter', covering.letter ?? 'fill'],
        ...(covering.thinEdgeInches !== undefined ? [['thinEdgeInches', covering.thinEdgeInches] as const] : []),
        ...(covering.thickEdgeInches !== undefined ? [['thickEdgeInches', covering.thickEdgeInches] as const] : []),
        ...(covering.thicknessInches !== undefined ? [['thicknessInches', covering.thicknessInches] as const] : []),
        ['manufacturer', covering.manufacturer ?? ''],
        ['product', covering.product ?? ''],
        ['documentId', covering.documentId ?? ''],
      ], element);
      axes(key, element, {
        division: csiDivision(covering.costCode), code: covering.costCode, quantity: covering.squareFeet, unit: 'SF',
      }, {
        src: `${covering.manufacturer ?? ''} ${covering.product ?? ''} (${covering.documentId ?? ''})`.trim(),
        hier: 'MFR',
        constraint: 'the panel’s own printed thin and thick edge',
        overridable: false,
      });
    } else {
      axes(key, element, {
        division: csiDivision(covering.costCode), code: covering.costCode, quantity: covering.squareFeet, unit: 'SF',
      }, {
        src: covering.item ? covering.item.description : 'the condition’s own line',
        hier: 'FIELD',
        constraint: 'none stated',
        overridable: true,
      });
    }

    // The fastening this field's base course is held down with — zones, class,
    // counts, standard, basis — lands on the fastened layer and nowhere else,
    // which on this program is the flat fill (C4: "a fastened base with
    // everything above it adhered").
    if (covering.role === 'insulation-fill') {
      const fastened = plan.fastening.find((f) => f.conditionId === covering.conditionId);
      if (fastened) {
        const f = fastened.field;
        const fasteningProps: [string, PropValue][] = [['standard', f.standard], ['basis', f.basis]];
        for (const zone of f.zones) {
          fasteningProps.push([`zone${zone.zone}Label`, zone.label]);
          if (zone.fmClass !== null) fasteningProps.push([`zone${zone.zone}FmClass`, zone.fmClass]);
          if (zone.fasteners !== null) fasteningProps.push([`zone${zone.zone}Fasteners`, zone.fasteners]);
        }
        pset(`${key}:fastening`, 'Fastening', fasteningProps, element);
      }
    }
  }

  // ── the walls and curbs: outer trace and the line WALL grows in from ──────
  for (const [n, wall] of plan.walls.entries()) {
    const run = wall.run;
    const key = `wall:${wall.conditionId}:${n}`;
    const body = extrudedBody(profileWithVoid(wall.name, run.outer.ring, run.inboard!.ring), run.heightFeet);
    const element = new IFC4.IfcWall(
      guid(key), null, label(wall.name), null, null,
      new IFC4.IfcLocalPlacement(storeyPlacement, placement(0, 0, run.elevationFeet)),
      body, identifier(wall.conditionId),
      wall.curb ? IFC4.IfcWallTypeEnum.SOLIDWALL : IFC4.IfcWallTypeEnum.PARAPET,
    );
    push(element);
    elements.push(element);
    qto(key, 'Qto_WallBaseQuantities', [
      { name: 'Height', kind: 'length', value: run.heightFeet },
      { name: 'Width', kind: 'length', value: inchesToFeet(run.wallInches ?? 0) },
      ...(run.flashSquareFeet !== null
        ? [{ name: 'FlashingArea', kind: 'area' as const, value: run.flashSquareFeet }] : []),
      ...(run.flashMinInches !== null
        ? [{ name: 'FlashingHeightLeast', kind: 'length' as const, value: inchesToFeet(run.flashMinInches) }] : []),
      ...(run.curbRaiseInches !== null
        ? [{ name: 'RaiseNeeded', kind: 'length' as const, value: inchesToFeet(run.curbRaiseInches) }] : []),
    ], element);
    axes(key, element, { division: csiDivision(wall.costCode), code: wall.costCode, quantity: run.heightFeet, unit: 'LF' }, {
      src: 'NRCA Membrane Roof Systems 2019 p.285 (D153)',
      hier: 'CODE',
      constraint: 'the least flashing height a curb or a parapet is raised to meet',
      overridable: false,
    });
  }

  // ── the coping cap: outer and inner edge, centred on the wall it caps ─────
  for (const [n, coping] of plan.copings.entries()) {
    const c = coping.coping;
    const key = `coping:${coping.conditionId}:${n}`;
    const body = extrudedBody(
      profileWithVoid(coping.name, c.capOuter!.ring, c.capInner!.ring), COPING_METAL_FEET,
    );
    const element = new IFC4.IfcCovering(
      guid(key), null, label(coping.name), null, null,
      new IFC4.IfcLocalPlacement(storeyPlacement, placement(0, 0, coping.elevationFeet)),
      body, identifier(coping.conditionId), IFC4.IfcCoveringTypeEnum.USERDEFINED,
    );
    element.ObjectType = label('COPING'); // IFC4's own enum has no COPING value; IFC4X3 adds one
    push(element);
    elements.push(element);
    qto(key, 'Qto_CoveringBaseQuantities', [
      { name: 'Length', kind: 'length', value: c.runFeet },
      { name: 'Width', kind: 'length', value: c.capWidthFeet ?? 0 },
      { name: 'PieceCount', kind: 'count', value: c.piecesEa },
      { name: 'CleatLength', kind: 'length', value: c.cleatFeet },
    ], element);
    axes(key, element, { division: csiDivision(coping.costCode), code: coping.costCode, quantity: c.runFeet, unit: 'LF' }, {
      src: `NRCA Architectural Metal Flashing 2022 p.58 — ${c.gaugeGa ?? 'no'} gauge`,
      hier: 'CODE',
      constraint: 'section length 8–20 ft; gauge by face height (Fig. 2-3)',
      overridable: false,
    });
  }

  // ── the drains: a fitting at each traced point, the sump as a property ────
  for (const [n, drain] of plan.drains.entries()) {
    const key = `drain:${drain.conditionId}:${n}`;
    const p = toIfc(drain.at);
    const element = new IFC4.IfcFlowTerminal(
      guid(key), null, label(drain.name), null, label('Roof drain'),
      new IFC4.IfcLocalPlacement(storeyPlacement, placement(p.x, p.y, 0)), null, identifier(drain.conditionId),
    );
    push(element);
    elements.push(element);
    const sumpProps: [string, PropValue][] = drain.sump
      ? [
        ['sumpSideFeet', drain.sump.sideFeet], ['sumpSquareFeet', drain.sump.squareFeet],
        ['sumpDepthInches', drain.sump.depthInches], ['sumpPanels', drain.sump.panels],
      ]
      : [['sump', 'a drain has no sump: nothing states its bowl or its width']];
    pset(key, 'Sump', sumpProps, element);
    axes(key, element, { division: csiDivision(drain.costCode), code: drain.costCode, quantity: 1, unit: 'EA' }, {
      src: 'the roof plan trace', hier: 'FIELD', constraint: 'none stated', overridable: true,
    });
  }

  // ── the crickets: the saddle behind a curb, the ridge it is traced as ─────
  for (const [n, cricket] of plan.crickets.entries()) {
    const cr = cricket.cricket;
    const triangles: [Point3, Point3, Point3][] = [];
    for (let i = 0; i + 8 < cr.positions.length; i += 9) {
      const at3 = (o: number): Point3 => {
        const q = toIfc({ x: cr.positions[o]!, y: cr.positions[o + 2]! });
        return { x: q.x, y: q.y, z: cr.positions[o + 1]! };
      };
      triangles.push([at3(i), at3(i + 3), at3(i + 6)]);
    }
    const body = tessellatedBody(triangles);
    if (!body) continue;
    const key = `cricket:${cricket.conditionId}:${n}`;
    const element = new IFC4.IfcSlab(
      guid(key), null, label(cricket.name), null, null,
      new IFC4.IfcLocalPlacement(storeyPlacement, placement(0, 0, 0)),
      body, identifier(cricket.conditionId), IFC4.IfcSlabTypeEnum.USERDEFINED,
    );
    element.ObjectType = label('CRICKET');
    push(element);
    elements.push(element);
    qto(key, 'Qto_SlabBaseQuantities', [
      { name: 'NetArea', kind: 'area', value: cr.planSquareFeet },
    ], element);
    const ridgeProps: [string, PropValue][] = cricket.ridge
      ? [
        ['ridgeFromX', cricket.ridge[0].x], ['ridgeFromY', cricket.ridge[0].y],
        ['ridgeToX', cricket.ridge[1].x], ['ridgeToY', cricket.ridge[1].y],
      ]
      : [];
    pset(key, 'Cricket', [
      ...ridgeProps,
      ['lengthToWidth', cr.lengthToWidth], ['derived', cr.derived],
    ], element);
    axes(key, element, { division: csiDivision(cricket.costCode), code: cricket.costCode, quantity: cr.planSquareFeet, unit: 'SF' }, {
      src: cr.derived ? 'the box-out-a-curb solve (C6)' : 'the ridge as traced',
      hier: 'FIELD',
      constraint: 'NRCA Figs. 8-1, 10-13 and 10-14',
      overridable: true,
    });
  }

  const relations: WEBIFC.IfcLineObject[] = [
    new IFC4.IfcRelAggregates(guid(`aggregate:site:${plan.job}`), null, null, null, project, [site]),
    new IFC4.IfcRelAggregates(guid(`aggregate:building:${plan.job}`), null, null, null, site, [building]),
    new IFC4.IfcRelAggregates(guid(`aggregate:storey:${plan.job}`), null, null, null, building, [storey]),
  ];
  push(...relations);
  if (elements.length > 0) {
    push(new IFC4.IfcRelContainedInSpatialStructure(
      guid(`contains:${plan.job}`), null, null, null, elements, storey,
    ));
  }

  for (const line of lines) api.WriteLine(model, line);
  const bytes = api.SaveModel(model);
  api.CloseModel(model);
  return bytes;
}

/**
 * How thick the coping's own metal is, drawn — never claimed as a solved
 * quantity, because nothing in the job states a gauge's own thickness in
 * inches (only its GAUGE NUMBER, which `Qto_CoveringBaseQuantities` does not
 * carry a unit for). An eighth of an inch is enough for the solid to be a
 * solid; the number an estimator reads for the metal is the gauge, in the
 * property set beside it.
 */
const COPING_METAL_FEET = 1 / 96;

/**
 * An `IfcGloballyUniqueId` for a piece of this roof.
 *
 * Derived from what the piece is rather than drawn at random, so writing the
 * same roof twice writes the same file: two exports of an unchanged job differ
 * only in their header timestamp, and a difference anywhere else is a real one.
 *
 * The encoding is buildingSMART's own — 128 bits in base 64 over their
 * alphabet, 22 characters, the first carrying the two bits that do not fit into
 * the other twenty-one. The 128 bits are four rounds of FNV-1a over the key,
 * which is a hash and not a random number: a name collision here would be two
 * pieces of one roof claiming the same identity, and the key includes the
 * condition and its ordinal so that cannot happen inside a job.
 */
const GUID_ALPHABET = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_$';

/** The 128 bits an identity is built from — shared by the compressed GlobalId and the IFCX bridge's UUID. */
export function identityBits(key: string): bigint {
  let bits = 0n;
  for (let lane = 0; lane < 4; lane += 1) {
    let h = 2166136261 ^ lane;
    for (let i = 0; i < key.length; i += 1) {
      h ^= key.charCodeAt(i);
      h = Math.imul(h, 16777619) >>> 0;
    }
    bits = (bits << 32n) | BigInt(h >>> 0);
  }
  return bits;
}

function guid(key: string): WEBIFC.IFC4.IfcGloballyUniqueId {
  let bits = identityBits(key);
  const out: string[] = [];
  for (let i = 0; i < 21; i += 1) {
    out.push(GUID_ALPHABET[Number(bits & 63n)]!);
    bits >>= 6n;
  }
  out.push(GUID_ALPHABET[Number(bits & 3n)]!);
  return new WEBIFC.IFC4.IfcGloballyUniqueId(out.reverse().join(''));
}
