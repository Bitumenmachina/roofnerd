// ── Measuring a whole job ──────────────────────────────────────────────────
// Ties the pieces together: pages give the scale, conditions give the traces,
// and a condition that says `from:` another one borrows its measures instead of
// being traced twice.

import type { Condition, JobDocument, Page } from './model.js';
import type { Calibrations, Measures } from './measures.js';
import { measure, scopeFor } from './measures.js';

/** What each page's scale is, keyed by page. Absent means nobody has scaled it. */
export function calibrationsOf(pages: readonly Page[]): Calibrations {
  const out: Record<string, { feetPerUnit: number } | undefined> = {};
  for (const page of pages) {
    if (page.feetPerUnit !== undefined && page.feetPerUnit > 0) {
      out[page.id] = { feetPerUnit: page.feetPerUnit };
    }
  }
  return out;
}

export class InheritanceError extends Error {}

/**
 * Measure every condition in the job.
 *
 * A condition with `from:` takes the measures of the one it names, and keeps its
 * own properties — that is the parapet → coping pattern: one run, measured once,
 * with a different height and a different item list hung off it.
 */
export function measureJob(doc: JobDocument): Map<string, Measures> {
  const calibrations = calibrationsOf(doc.pages);
  const byId = new Map(doc.conditions.map((c) => [c.id, c]));
  const done = new Map<string, Measures>();

  const resolve = (condition: Condition, seen: Set<string>): Measures => {
    const cached = done.get(condition.id);
    if (cached) return cached;

    let result: Measures;
    if (condition.from === undefined) {
      result = measure(condition.kind, condition.traces, condition.properties, calibrations);
    } else {
      if (seen.has(condition.id)) {
        throw new InheritanceError(
          `"${condition.name}" takes its measures from itself, round in a circle`,
        );
      }
      const parent = byId.get(condition.from);
      if (!parent) {
        throw new InheritanceError(
          `"${condition.name}" takes its measures from a condition that is not here`,
        );
      }
      seen.add(condition.id);
      result = resolve(parent, seen);
    }

    done.set(condition.id, result);
    return result;
  };

  for (const condition of doc.conditions) resolve(condition, new Set());
  return done;
}

/** Everything a formula on this condition's lines can see. */
export function scopeOf(doc: JobDocument, conditionId: string): Record<string, number | null> {
  const condition = doc.conditions.find((c) => c.id === conditionId);
  if (!condition) throw new InheritanceError(`there is no condition ${conditionId}`);
  const measures = measureJob(doc).get(conditionId)!;
  return scopeFor(measures, condition.properties);
}
