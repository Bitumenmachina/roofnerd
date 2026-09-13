// ── File → Export IFC ──────────────────────────────────────────────────────
// The menu item, and nothing the writer itself needs to know about: gather the
// document and the three solves the way `doc.ts` already pulls them, hand them
// to `roofPlan`, write the bytes with a live `IfcAPI`, and put the file where
// an estimator's other exports already land.

import { doc, solveFastening, solveTaper, solveWalls, writeExport } from '../doc.js';
import { sayInStatus } from '../chrome.js';
import { expectedOf, roofPlan, writeRoof } from './write.js';
import { ifcApi } from './runtime.js';
import { writeIfcx } from './ifcx.js';

const message = (e: unknown): string => (typeof e === 'string' ? e : e instanceof Error ? e.message : String(e));

/**
 * Today, on the calendar of whoever is sitting here — not the ISO string off a
 * `Date`, which is UTC and would misname a file exported at night on this
 * coast. `reports.ts` has the same function; a third file wanting the date is
 * not yet a shared module wanting to exist.
 */
function today(): string {
  const now = new Date();
  const two = (n: number) => String(n).padStart(2, '0');
  return `${now.getFullYear()}-${two(now.getMonth() + 1)}-${two(now.getDate())}`;
}

/** A job's name, turned into a name a filesystem accepts and a person reads. */
function slug(name: string): string {
  const safe = name.trim().toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '');
  return safe || 'roof';
}

/**
 * Export the open job as IFC4, and the alpha IFCX bridge beside it.
 *
 * `roofPlan` and `writeRoof` are pure; everything here is the wiring pure code
 * cannot do for itself — pulling the three solves, initialising `web-ifc`, and
 * writing the bytes into the job's own `exports/` folder. `expected.json`
 * lands beside the `.ifc` so the landing chain's IfcOpenShell script has the
 * solve's own numbers to check the reopened file against, without opening the
 * job itself.
 */
export async function exportIfc(from: HTMLElement): Promise<void> {
  const [taper, walls, fastening] = await Promise.all([solveTaper(), solveWalls(), solveFastening()]);
  const plan = roofPlan(doc(), taper, walls, fastening);
  const api = await ifcApi();
  const bytes = writeRoof(plan, api);
  const text = new TextDecoder().decode(bytes);
  const expected = expectedOf(plan);
  const ifcx = writeIfcx(plan);

  const name = `${slug(plan.job)}-${today()}`;
  const ifcPath = await writeExport(`${name}.ifc`, text);
  await writeExport(`${name}.expected.json`, `${JSON.stringify(expected, null, 2)}\n`);
  await writeExport(`${name}.ifcx`, `${ifcx}\n`);

  // What was left out goes FIRST, because the path does not fit in the bar and
  // whatever comes after it is the half that gets cut. The status line read
  // "Exported /home/…/exports/northgate-retail-ce…" and stopped, so the one
  // part of the sentence worth reading — that something did not make it into
  // the file — was the part nobody saw (C8 §10).
  const left = plan.leftOut.length;
  const note = left > 0 ? `${left} thing${left === 1 ? '' : 's'} left out, named on the panel — ` : '';
  sayInStatus(from, `${note}exported ${ifcPath}`);
}
