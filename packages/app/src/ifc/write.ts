// ── The roof, written as an IFC4 model ─────────────────────────────────────
// The same traces the estimate prices, written out as the file the rest of the
// construction industry reads. Nothing here is modelled by hand and nothing is
// invented: every ring comes off a trace, every thickness off the layers on the
// condition, every height and elevation off a property somebody typed. Where a
// number has not been stated the piece is left out and the plan says so, by
// name — an IfcSlab with a thickness nobody stated would be a picture of a roof
// rather than this roof.
//
// Two functions, on purpose. `roofPlan` reads the job and answers what can be
// stood up and what cannot, with no web-ifc anywhere near it; `writeRoof` turns
// that into bytes. The split is what lets the window say "the Low Roof is not
// stood up: nothing says how thick its build-up is" in the same words the file
// was written with, and it is what lets a test in Node read the bytes back.
//
// Units are FEET. IFC does not require metres — it requires the file to say
// what its numbers mean, which `IfcConversionBasedUnit` does — and a roof
// written in feet carries the sheet's own figures: the square feet in the model
// are the square feet on the estimate, with no conversion standing between the
// two where a rounding rule could get in. The conversion factor to the metre is
// stated in the file, so anything else that opens it knows exactly what a foot
// is.
//
// Geometry conventions, all of them the same as the Model editor's:
//   plan (x, y) becomes IFC (x, -y, z)   — plan y grows down the sheet, IFC +Y
//                                          is north, so the sheet keeps its
//                                          orientation and north stays north
//   a build-up grows UP from the deck    — `GROWS_UP_FROM_DECK` in the engine
//   a parapet stands INBOARD of its run  — the traced line is the outside face
//
// Every profile is written in plan feet with the solid sitting at the origin and
// the extrusion straight up, so no rotation is carried anywhere: a corner in the
// file is at the coordinate it was traced at. An element's own placement holds
// its elevation, which is the one thing the storey cannot say for it.

import { buildUpInches } from '@roofnerd/engine';
import type { Condition, Page, Point } from '@roofnerd/engine';
import * as WEBIFC from 'web-ifc';
import { feet, feetPerUnit } from '../scale.js';
import type { Doc } from '../doc.js';

/** An area condition with a stated build-up, ready to be an IfcSlab. */
export interface SlabPlan {
  readonly conditionId: string;
  readonly name: string;
  /** The traced outline, in feet, in plan coordinates. */
  readonly ring: readonly Point[];
  /** Top of structural deck, in feet. */
  readonly elevation: number;
  /** What the layers on the condition add up to, in feet. */
  readonly thickness: number;
}

/** One straight length of a parapet run or a curb, ready to be an IfcWall. */
export interface WallPlan {
  readonly conditionId: string;
  readonly name: string;
  /**
   * A curb, rather than the edge of the roof.
   *
   * Both are a run with a height and both stand up the same way, so both are an
   * IfcWall — but IFC has a type for a parapet and calling a curb one would tell
   * an architect's program that the box round the unit is the edge of the
   * building. `SOLIDWALL` is what is left when the specific type is wrong.
   */
  readonly curb: boolean;
  readonly from: Point;
  readonly to: Point;
  /** The base the run starts from, in feet. */
  readonly elevation: number;
  /** How far it stands above that base, in feet. */
  readonly height: number;
  /** How thick the wall is, in feet. */
  readonly thickness: number;
  /**
   * Which side of the traced line the wall stands on: +1 to the left of the
   * run's own direction, -1 to the right. Read off the whole run, never off one
   * length of it, because one length cannot tell inside from outside.
   */
  readonly inboard: 1 | -1;
}

/** Something the job carries that is not standing up, and why not. */
export interface LeftOut {
  readonly name: string;
  readonly why: string;
}

export interface RoofPlan {
  readonly job: string;
  readonly slabs: readonly SlabPlan[];
  readonly walls: readonly WallPlan[];
  readonly leftOut: readonly LeftOut[];
}

const conditionsOf = (d: Doc): Condition[] => (d['conditions'] as Condition[]) ?? [];
const pagesOf = (d: Doc): Page[] => (d['pages'] as Page[]) ?? [];
const jobName = (d: Doc): string => {
  const job = d['job'] as { name?: unknown } | undefined;
  return typeof job?.name === 'string' && job.name ? job.name : 'Untitled';
};

/** Inches to feet, because a thickness is stated in inches and the model is in feet. */
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

/**
 * What of this job can be stood up, and what cannot.
 *
 * Pure: the job in, a description out, no wasm and no window. Every condition
 * lands in exactly one of the three lists, so nothing goes missing quietly —
 * that is the whole reason `leftOut` carries a reason rather than being a count.
 */
export function roofPlan(d: Doc): RoofPlan {
  const list = conditionsOf(d);
  const scales = feetPerUnit(pagesOf(d));
  const slabs: SlabPlan[] = [];
  const walls: WallPlan[] = [];
  const leftOut: LeftOut[] = [];

  for (const c of list) {
    if (c.hidden) continue;
    const name = c.name ?? c.id;
    const props = c.properties ?? {};
    const elevation = props['ELEV'] ?? 0;
    const rings = (c.traces ?? []).map((t) => feet(t, scales));
    const drawn = rings.filter((r) => r.length > 0);

    if (drawn.length === 0) {
      leftOut.push({ name, why: 'the sheet it was traced on has no scale yet' });
      continue;
    }

    if (c.kind === 'area') {
      const stack = buildUpInches(c.items ?? []);
      if (stack === null) {
        leftOut.push({ name, why: 'nothing says how thick the build-up is' });
        continue;
      }
      for (const ring of drawn) {
        if (ring.length < 3) {
          leftOut.push({ name, why: 'the outline has fewer than three corners' });
          continue;
        }
        slabs.push({
          conditionId: c.id, name, ring, elevation, thickness: inchesToFeet(stack),
        });
      }
      continue;
    }

    if (c.kind === 'line') {
      if (c.role === 'ridge') {
        leftOut.push({ name, why: 'a cricket is two planes off the drains, and they are not cut yet' });
        continue;
      }
      const height = props['H'] ?? 0;
      if (height <= 0) {
        leftOut.push({ name, why: 'it has no height, so it is a run on the plan rather than a wall' });
        continue;
      }
      const thick = props['WALL'];
      if (thick === undefined) {
        leftOut.push({ name, why: 'nothing says how thick the wall is' });
        continue;
      }
      for (const ring of drawn) {
        const middle = centre(ring);
        for (let i = 0; i < ring.length - 1; i += 1) {
          const from = ring[i]!;
          const to = ring[i + 1]!;
          walls.push({
            conditionId: c.id, name, from, to, curb: c.role === 'curb',
            elevation, height, thickness: inchesToFeet(thick),
            inboard: towards(from, to, middle),
          });
        }
      }
      continue;
    }

    leftOut.push({ name, why: 'a count is a fitting, and fittings are not stood up yet' });
  }

  return { job: jobName(d), slabs, walls, leftOut };
}

/** The job as IFC4 bytes. */
export function writeIfc(d: Doc, api: WEBIFC.IfcAPI): Uint8Array {
  return writeRoof(roofPlan(d), api);
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

  /** A closed plan profile, in feet, with its outer curve counter-clockwise. */
  const profile = (name: string, ring: readonly Point[]) => {
    const plan2d = ring.map((p) => ({ x: p.x, y: -p.y }));
    const wound = signedArea(plan2d) < 0 ? [...plan2d].reverse() : plan2d;
    const closed = [...wound, wound[0]!];
    return new IFC4.IfcArbitraryClosedProfileDef(
      IFC4.IfcProfileTypeEnum.AREA, label(name),
      new IFC4.IfcPolyline(closed.map((p) => point(p.x, p.y, 0))),
    );
  };

  const body = (solid: WEBIFC.IFC4.IfcExtrudedAreaSolid) =>
    new IFC4.IfcProductDefinitionShape(null, null, [
      new IFC4.IfcShapeRepresentation(context, label('Body'), label('SweptSolid'), [solid]),
    ]);

  const upward = (area: WEBIFC.IFC4.IfcArbitraryClosedProfileDef, depth: number) =>
    new IFC4.IfcExtrudedAreaSolid(
      area, placement(0, 0, 0), direction(0, 0, 1), new IFC4.IfcPositiveLengthMeasure(depth),
    );

  const elements: (WEBIFC.IFC4.IfcSlab | WEBIFC.IFC4.IfcWall)[] = [];

  for (const [n, slab] of plan.slabs.entries()) {
    elements.push(new IFC4.IfcSlab(
      guid(`slab:${slab.conditionId}:${n}`), null, label(slab.name), null, null,
      new IFC4.IfcLocalPlacement(storeyPlacement, placement(0, 0, slab.elevation)),
      body(upward(profile(slab.name, slab.ring), slab.thickness)),
      new IFC4.IfcIdentifier(slab.conditionId), IFC4.IfcSlabTypeEnum.ROOF,
    ));
  }

  for (const [n, wall] of plan.walls.entries()) {
    elements.push(new IFC4.IfcWall(
      guid(`wall:${wall.conditionId}:${n}`), null, label(wall.name), null, null,
      new IFC4.IfcLocalPlacement(storeyPlacement, placement(0, 0, wall.elevation)),
      body(upward(profile(wall.name, footprint(wall)), wall.height)),
      new IFC4.IfcIdentifier(wall.conditionId),
      wall.curb ? IFC4.IfcWallTypeEnum.SOLIDWALL : IFC4.IfcWallTypeEnum.PARAPET,
    ));
  }

  const lines: WEBIFC.IfcLineObject[] = [
    project, site, building, storey,
    new IFC4.IfcRelAggregates(guid(`aggregate:site:${plan.job}`), null, null, null, project, [site]),
    new IFC4.IfcRelAggregates(guid(`aggregate:building:${plan.job}`), null, null, null, site, [building]),
    new IFC4.IfcRelAggregates(guid(`aggregate:storey:${plan.job}`), null, null, null, building, [storey]),
    ...elements,
  ];
  if (elements.length > 0) {
    lines.push(new IFC4.IfcRelContainedInSpatialStructure(
      guid(`contains:${plan.job}`), null, null, null, elements, storey,
    ));
  }

  for (const line of lines) api.WriteLine(model, line);
  const bytes = api.SaveModel(model);
  api.CloseModel(model);
  return bytes;
}

/**
 * The plan footprint of one length of parapet: the traced line, thickened
 * inboard.
 *
 * The traced line is the outside face — that is what a hand traces off a roof
 * plan, and it is the convention the Model editor already draws to, square to
 * the run by the wall's own thickness on whichever side faces the middle of the
 * run.
 */
function footprint(wall: WallPlan): Point[] {
  const dx = wall.to.x - wall.from.x;
  const dy = wall.to.y - wall.from.y;
  const len = Math.hypot(dx, dy) || 1;
  const nx = (-dy / len) * wall.thickness * wall.inboard;
  const ny = (dx / len) * wall.thickness * wall.inboard;
  return [
    wall.from,
    wall.to,
    { x: wall.to.x + nx, y: wall.to.y + ny },
    { x: wall.from.x + nx, y: wall.from.y + ny },
  ];
}

/** The middle of a run, which is what tells one side of it from the other. */
function centre(ring: readonly Point[]): Point {
  return {
    x: ring.reduce((sum, p) => sum + p.x, 0) / ring.length,
    y: ring.reduce((sum, p) => sum + p.y, 0) / ring.length,
  };
}

/** Whether the middle of the run lies to the left of this length or its right. */
function towards(from: Point, to: Point, middle: Point): 1 | -1 {
  const dx = to.x - from.x;
  const dy = to.y - from.y;
  const len = Math.hypot(dx, dy) || 1;
  const ux = -dy / len;
  const uy = dx / len;
  const mx = (from.x + to.x) / 2;
  const my = (from.y + to.y) / 2;
  return (middle.x - mx) * ux + (middle.y - my) * uy >= 0 ? 1 : -1;
}

/** Twice the area of a ring, signed: positive counter-clockwise, y up. */
function signedArea(ring: readonly Point[]): number {
  let sum = 0;
  for (let i = 0; i < ring.length; i += 1) {
    const a = ring[i]!;
    const b = ring[(i + 1) % ring.length]!;
    sum += a.x * b.y - b.x * a.y;
  }
  return sum;
}

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

function guid(key: string): WEBIFC.IFC4.IfcGloballyUniqueId {
  let bits = 0n;
  for (let lane = 0; lane < 4; lane += 1) {
    let h = 2166136261 ^ lane;
    for (let i = 0; i < key.length; i += 1) {
      h ^= key.charCodeAt(i);
      h = Math.imul(h, 16777619) >>> 0;
    }
    bits = (bits << 32n) | BigInt(h >>> 0);
  }
  const out: string[] = [];
  for (let i = 0; i < 21; i += 1) {
    out.push(GUID_ALPHABET[Number(bits & 63n)]!);
    bits >>= 6n;
  }
  out.push(GUID_ALPHABET[Number(bits & 3n)]!);
  return new WEBIFC.IFC4.IfcGloballyUniqueId(out.reverse().join(''));
}
