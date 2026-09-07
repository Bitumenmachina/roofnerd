// ── The job folder ─────────────────────────────────────────────────────────
// A job is a folder of plain text, not a database file:
//
//   job.json          identity and the scenarios (the price sets)
//   conditions.json   everything traced
//   pages/pages.json  the drawings, and where each one's source file lives
//   exports/          what you sent out
//
// You can open it in Notepad, grep it, diff it, sync it, read it on a phone.
// The estimator owns the data. That is not a feature, it is the point.
//
// This file does no file I/O of its own — it takes a reader and a writer. That
// keeps the engine runnable in a test, in a browser and on a command line, and
// it is why the arithmetic can be checked without a window ever opening.

import type { Condition, CostCode, Job, JobDocument, Page } from './model.js';
import { JOB_FORMAT } from './model.js';

/** Reads a file inside the job folder. Returns null when it is not there. */
export type ReadFile = (relativePath: string) => string | null;
/** Writes a file inside the job folder, creating directories as needed. */
export type WriteFile = (relativePath: string, contents: string) => void;

export const JOB_FILE = 'job.json';
export const CONDITIONS_FILE = 'conditions.json';
export const PAGES_FILE = 'pages/pages.json';
export const CODES_FILE = 'cost-codes.json';

export class JobFolderError extends Error {}

function parse<T>(raw: string | null, what: string, fallback?: T): T {
  if (raw == null) {
    if (fallback !== undefined) return fallback;
    throw new JobFolderError(`${what} is missing from this job folder`);
  }
  try {
    return JSON.parse(raw) as T;
  } catch (e) {
    throw new JobFolderError(`${what} is not readable JSON: ${(e as Error).message}`);
  }
}

export function readJob(read: ReadFile): JobDocument {
  const job = parse<Job>(read(JOB_FILE), JOB_FILE);
  if (job.format !== JOB_FORMAT) {
    throw new JobFolderError(
      `this job was written by format ${job.format}; this build reads format ${JOB_FORMAT}`,
    );
  }
  return {
    job,
    pages: parse<Page[]>(read(PAGES_FILE), PAGES_FILE, []),
    conditions: parse<Condition[]>(read(CONDITIONS_FILE), CONDITIONS_FILE, []),
    costCodes: parse<CostCode[]>(read(CODES_FILE), CODES_FILE, []),
  };
}

/**
 * Written with sorted keys and a trailing newline, always. A saved job that
 * reorders its own keys shows up as a hundred changed lines in a diff and
 * hides the one line you actually changed.
 */
function stable(value: unknown): string {
  const seen = new WeakSet<object>();
  const sort = (v: unknown): unknown => {
    if (v === null || typeof v !== 'object') return v;
    if (seen.has(v as object)) throw new JobFolderError('a job cannot contain a cycle');
    seen.add(v as object);
    if (Array.isArray(v)) return v.map(sort);
    const out: Record<string, unknown> = {};
    for (const key of Object.keys(v as Record<string, unknown>).sort()) {
      out[key] = sort((v as Record<string, unknown>)[key]);
    }
    return out;
  };
  return `${JSON.stringify(sort(value), null, 2)}\n`;
}

export function writeJob(doc: JobDocument, write: WriteFile): void {
  write(JOB_FILE, stable(doc.job));
  write(PAGES_FILE, stable(doc.pages));
  write(CONDITIONS_FILE, stable(doc.conditions));
  write(CODES_FILE, stable(doc.costCodes));
}

/** A new, empty job. One scenario, because a job always has at least one price set. */
export function emptyJob(name: string): JobDocument {
  return {
    job: {
      format: JOB_FORMAT,
      name,
      activeScenarioId: 'scenario-1',
      scenarios: [{ id: 'scenario-1', name: 'Scenario 1', prices: {}, adders: {} }],
    },
    pages: [],
    conditions: [],
    costCodes: [],
  };
}
