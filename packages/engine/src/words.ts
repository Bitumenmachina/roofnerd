// ── Words that have to agree with a number ─────────────────────────────────
// One trace or two traces. Never "trace(s)".
//
// This lives in the engine and not only in the app because the engine writes
// text a person reads: the recap it prints under a total, and the reason a
// formula gives when it will not run — which the sheet puts on the line, in the
// window, beside the figure it refused to produce. Two copies of this rule would
// have meant the app obeyed it and the arithmetic did not, which is what
// happened: `3 line(s) not in the total` and `takes 2 number(s)` were both the
// engine's, and the gate that forbids them only read the app.

/** "1 trade", "2 trades" — the count and its word, agreeing. */
export const plural = (n: number, one: string, many = `${one}s`): string =>
  `${n.toLocaleString('en-US')} ${n === 1 ? one : many}`;
