// ── The formula on the line ────────────────────────────────────────────────
// The arithmetic that turns what was measured into how much of an item is
// needed. It is printed on the estimate line and edited there, because a
// formula you cannot see is a formula you cannot trust, and every estimator has
// been burned by a library number they could not check.
//
// The grammar is deliberately small and closed:
//
//   expression := term (('+' | '-') term)*
//   term       := factor (('*' | '/') factor)*
//   factor     := number | name | name '(' args ')' | '(' expression ')' | ('-'|'+') factor
//
// Names are the measures and properties of the condition — SF, LF, EA, SQ,
// PLAN_SF, H, W, T, PITCH, SIDES, STRETCHOUT, and anything the estimator added.
// Functions are ceil, floor, round, max, min and nothing else.
//
// It is parsed to a tree and walked. There is no eval anywhere in this program:
// a formula is data typed into a bid, and data does not get to run.

import { plural } from './words.js';
import { ceilPackages, floorWhole, roundWhole } from './rounding.js';

export type Node =
  | { readonly kind: 'number'; readonly value: number }
  | { readonly kind: 'name'; readonly name: string }
  | { readonly kind: 'unary'; readonly op: '-' | '+'; readonly operand: Node }
  | { readonly kind: 'binary'; readonly op: '+' | '-' | '*' | '/'; readonly left: Node; readonly right: Node }
  | { readonly kind: 'call'; readonly name: string; readonly args: readonly Node[] };

export class FormulaError extends Error {
  /** Where in the text it went wrong, so the line can point at it. */
  readonly position: number;
  constructor(message: string, position: number) {
    super(message);
    this.name = 'FormulaError';
    this.position = position;
  }
}

// An estimator who writes ceil(...) on a line gets the same protection the
// order-unit step gets: the crumbs are settled off before the rounding, so
// `ceil(LF * 1.1 / 10)` does not quietly buy one more of anything. See
// rounding.ts for why that is necessary at all.
const FUNCTIONS: Record<string, { arity: number | 'many'; apply: (args: number[]) => number }> = {
  ceil: { arity: 1, apply: ([a]) => ceilPackages(a!) },
  floor: { arity: 1, apply: ([a]) => floorWhole(a!) },
  round: { arity: 1, apply: ([a]) => roundWhole(a!) },
  max: { arity: 'many', apply: (args) => Math.max(...args) },
  min: { arity: 'many', apply: (args) => Math.min(...args) },
};

export const FUNCTION_NAMES = Object.keys(FUNCTIONS);

// ── Tokens ─────────────────────────────────────────────────────────────────

type Token =
  | { kind: 'number'; value: number; at: number }
  | { kind: 'name'; value: string; at: number }
  | { kind: 'symbol'; value: string; at: number };

function tokenize(source: string): Token[] {
  const tokens: Token[] = [];
  let i = 0;
  while (i < source.length) {
    const c = source[i]!;
    if (c === ' ' || c === '\t' || c === '\n') { i++; continue; }

    if ('+-*/(),'.includes(c)) {
      tokens.push({ kind: 'symbol', value: c, at: i });
      i++;
      continue;
    }

    if (/[0-9.]/.test(c)) {
      let j = i + 1;
      while (j < source.length && /[0-9.]/.test(source[j]!)) j++;
      const text = source.slice(i, j);
      // "1.2.3" is not a number. Accepting it would price the line on a figure
      // nobody typed, silently, which is exactly the kind of quiet wrong this
      // program is built to refuse.
      if (!/^(\d+\.?\d*|\.\d+)$/.test(text)) {
        throw new FormulaError(`"${text}" is not a number`, i);
      }
      tokens.push({ kind: 'number', value: parseFloat(text), at: i });
      i = j;
      continue;
    }

    if (/[A-Za-z_]/.test(c)) {
      let j = i + 1;
      while (j < source.length && /[A-Za-z0-9_]/.test(source[j]!)) j++;
      tokens.push({ kind: 'name', value: source.slice(i, j), at: i });
      i = j;
      continue;
    }

    throw new FormulaError(`"${c}" does not belong in a formula`, i);
  }
  return tokens;
}

// ── Parser ─────────────────────────────────────────────────────────────────

export function parse(source: string): Node {
  const tokens = tokenize(source);
  let i = 0;

  const peek = () => tokens[i];
  const end = () => (tokens.length ? tokens[tokens.length - 1]!.at + 1 : 0);

  const eat = (value: string): boolean => {
    const t = tokens[i];
    if (t && t.kind === 'symbol' && t.value === value) { i++; return true; }
    return false;
  };

  const expect = (value: string) => {
    if (!eat(value)) {
      const t = peek();
      throw new FormulaError(`expected "${value}"`, t ? t.at : end());
    }
  };

  function expression(): Node {
    let left = term();
    for (;;) {
      const t = peek();
      if (t?.kind === 'symbol' && (t.value === '+' || t.value === '-')) {
        i++;
        left = { kind: 'binary', op: t.value as '+' | '-', left, right: term() };
      } else return left;
    }
  }

  function term(): Node {
    let left = factor();
    for (;;) {
      const t = peek();
      if (t?.kind === 'symbol' && (t.value === '*' || t.value === '/')) {
        i++;
        left = { kind: 'binary', op: t.value as '*' | '/', left, right: factor() };
      } else return left;
    }
  }

  function factor(): Node {
    const t = peek();
    if (!t) throw new FormulaError('the formula stops early', end());

    if (t.kind === 'symbol' && (t.value === '-' || t.value === '+')) {
      i++;
      return { kind: 'unary', op: t.value as '-' | '+', operand: factor() };
    }
    if (t.kind === 'symbol' && t.value === '(') {
      i++;
      const inner = expression();
      expect(')');
      return inner;
    }
    if (t.kind === 'number') { i++; return { kind: 'number', value: t.value }; }
    if (t.kind === 'name') {
      i++;
      if (eat('(')) {
        const args: Node[] = [];
        if (!eat(')')) {
          do { args.push(expression()); } while (eat(','));
          expect(')');
        }
        return { kind: 'call', name: t.value, args };
      }
      return { kind: 'name', name: t.value };
    }
    throw new FormulaError(`"${t.value}" does not belong here`, t.at);
  }

  if (tokens.length === 0) throw new FormulaError('the formula is empty', 0);
  const tree = expression();
  const leftover = peek();
  if (leftover) throw new FormulaError(`"${leftover.value}" is left over`, leftover.at);
  return tree;
}

// ── Evaluation ─────────────────────────────────────────────────────────────

/**
 * Work the formula out against what the condition measures.
 *
 * A name the scope holds as `null` — a length on a page nobody has scaled yet —
 * makes the whole answer null. It does not become zero. A zero would go on to
 * price as nothing at all and look like a finished line.
 */
export function evaluate(tree: Node, scope: Readonly<Record<string, number | null>>): number | null {
  switch (tree.kind) {
    case 'number':
      return tree.value;

    case 'name': {
      if (!(tree.name in scope)) throw new FormulaError(`nothing here is called "${tree.name}"`, 0);
      return scope[tree.name] ?? null;
    }

    case 'unary': {
      const v = evaluate(tree.operand, scope);
      return v === null ? null : tree.op === '-' ? -v : v;
    }

    case 'binary': {
      const left = evaluate(tree.left, scope);
      const right = evaluate(tree.right, scope);
      if (left === null || right === null) return null;
      switch (tree.op) {
        case '+': return left + right;
        case '-': return left - right;
        case '*': return left * right;
        case '/':
          if (right === 0) throw new FormulaError('this divides by zero', 0);
          return left / right;
      }
    }

    case 'call': {
      const fn = FUNCTIONS[tree.name];
      if (!fn) throw new FormulaError(`there is no function called "${tree.name}"`, 0);
      if (fn.arity === 'many' ? tree.args.length === 0 : tree.args.length !== fn.arity) {
        throw new FormulaError(
          `${tree.name} takes ${fn.arity === 'many' ? 'at least one number' : plural(fn.arity, 'number')}`,
          0,
        );
      }
      const args: number[] = [];
      for (const arg of tree.args) {
        const v = evaluate(arg, scope);
        if (v === null) return null;
        args.push(v);
      }
      const out = fn.apply(args);
      return Number.isFinite(out) ? out : null;
    }
  }
}

/** What a line shows: a number, or the reason there isn't one. */
export interface FormulaResult {
  readonly value: number | null;
  /** Present when the formula itself is wrong. Shown on the line, next to it. */
  readonly error?: string;
  /** Where in the text the trouble is. */
  readonly position?: number;
}

/** Parse and work out a formula in one go, turning any throw into something to show. */
export function run(source: string, scope: Readonly<Record<string, number | null>>): FormulaResult {
  try {
    return { value: evaluate(parse(source), scope) };
  } catch (e) {
    if (e instanceof FormulaError) return { value: null, error: e.message, position: e.position };
    throw e;
  }
}

/** The names a formula mentions — for telling an estimator what a line depends on. */
export function namesUsed(tree: Node, into: Set<string> = new Set()): Set<string> {
  switch (tree.kind) {
    case 'name': into.add(tree.name); break;
    case 'unary': namesUsed(tree.operand, into); break;
    case 'binary': namesUsed(tree.left, into); namesUsed(tree.right, into); break;
    case 'call': for (const a of tree.args) namesUsed(a, into); break;
    case 'number': break;
  }
  return into;
}
