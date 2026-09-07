// ── Rounding ───────────────────────────────────────────────────────────────
// One rule, in one place, for every point in this program where a measured
// quantity meets a step that rounds it.
//
// Why it has to exist: binary floating point cannot hold a tenth. `100 * 1.1`
// is `110.00000000000001`, and `Math.ceil(110.00000000000001 / 10)` is twelve
// rolls where eleven were wanted. The error is about one part in ten thousand
// billion — invisible until it lands next to a ceiling, and then it costs a
// roll of material on the bid and an argument on the job.
//
// The rule: before any rounding step, settle the number to SETTLE_PLACES
// decimal places. Nine places is a nanometre on a mile — far below anything an
// estimator measures, and far above the crumbs the arithmetic leaves behind.
//
// This is not a tolerance on the answer. The stored quantity keeps its full
// precision; only the value handed to a rounding step is settled.

/** Decimal places a quantity is settled to before it meets a rounding step. */
export const SETTLE_PLACES = 9;

const SCALE = 10 ** SETTLE_PLACES;

/** Drop the crumbs binary floating point leaves behind. */
export const settle = (value: number): number =>
  Number.isFinite(value) ? Math.round(value * SCALE) / SCALE : value;

/** Round up to a whole package. You cannot buy two thirds of a bucket. */
export const ceilPackages = (value: number): number => Math.ceil(settle(value));

/** Round to the nearest whole. */
export const roundWhole = (value: number): number => Math.round(settle(value));

/** Round down to a whole. */
export const floorWhole = (value: number): number => Math.floor(settle(value));
