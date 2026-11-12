#!/usr/bin/env python3
"""A workbook's formulas worked out by a second hand.

    tools/workbook.py <workbook.xlsx>
    tools/workbook.py <workbook.xlsx> --set <sheet> <element> <line> <new unit cost>

Reads the workbook this program writes (`plumbline export estimate job out.xlsx`) and
works every formula in it out again from the cells the formula names, knowing only
ROUND, SUM, SUMIF, COUNT, COUNTIF, COUNTIFS, IF, OR, comparisons, the four operations, brackets and
cell references. It knows nothing
of this program. Each formula has to come to the figure written beside it (the figure a
reader that does not work formulas out would show); the script fails on the first that
does not, and otherwise prints what each row of the recap comes to.

With --set, the unit cost of one line of the estimate is changed in the workbook's cells
first (nothing is written back to the file), so what the formulas come to after the
change can be read. Needs openpyxl.
"""
import math
import re
import sys

import openpyxl


def rounded(x, places):
    # Half away from nothing, as a workbook's ROUND has it; and 1.015 held as
    # 1.0149999 is the half it was meant to be.
    scale = 10 ** places
    return math.copysign(math.floor(abs(x) * scale + 0.5 + 1e-9), x) / scale


class Wrong:
    """What arithmetic on a word comes to (a workbook shows #VALUE!). An IF works out
    only the side it takes, so the side it does not take may be this and no harm done;
    a formula that comes to it is a fault."""

    def __repr__(self):
        return "#VALUE!"


WRONG = Wrong()


class Book:
    def __init__(self, path):
        self.formulas = openpyxl.load_workbook(path)                # cells as written: formulas as text
        self.written = openpyxl.load_workbook(path, data_only=True)  # the figure written with each formula
        self.memo = {}

    def cell(self, sheet, ref):
        key = (sheet, ref.replace("$", ""))
        if key in self.memo:
            return self.memo[key]
        value = self.formulas[sheet][key[1]].value
        if isinstance(value, str) and value.startswith("="):
            value = Formula(self, sheet, value[1:]).work()
        self.memo[key] = value
        return value

    def cells(self, sheet, a, b):
        a, b = a.replace("$", ""), b.replace("$", "")
        return [self.cell(sheet, c.coordinate) for row in self.formulas[sheet][f"{a}:{b}"] for c in row]


class Formula:
    TOKEN = re.compile(r"\s*(?:(\d+\.?\d*)|\"([^\"]*)\"|((?:[A-Za-z_][A-Za-z0-9_ ]*!)?\$?[A-Z]+\$?\d+(?::\$?[A-Z]+\$?\d+)?)|([A-Z]+(?=\())|(.))")

    def __init__(self, book, sheet, text):
        self.book, self.sheet, self.text = book, sheet, text
        self.tokens = [m.groups() for m in self.TOKEN.finditer(text) if m.group(0).strip()]
        self.at = 0

    def peek(self):
        return self.tokens[self.at] if self.at < len(self.tokens) else (None,) * 5

    def take(self):
        token = self.peek()
        self.at += 1
        return token

    def work(self):
        value = self.compare()
        if self.at != len(self.tokens):
            raise SystemExit(f"FAIL: cannot read the formula ={self.text}")
        return value

    @staticmethod
    def number(v):
        # An empty cell, or one whose formula came to nothing, is nothing in arithmetic;
        # a word is not a number at all.
        if v is None or v == "":
            return 0.0
        return WRONG if isinstance(v, (str, Wrong)) else v

    @classmethod
    def worked(cls, a, b, how):
        a, b = cls.number(a), cls.number(b)
        return WRONG if a is WRONG or b is WRONG else how(a, b)

    @staticmethod
    def same(a, b):
        blank = lambda v: v is None or v == ""
        return (blank(a) and blank(b)) or a == b

    def compare(self):
        value = self.sum()
        sign = self.peek()[4]
        if sign in ("=", "<", ">"):
            self.take()
            if self.peek()[4] in ("=", ">") and sign in ("<", ">"):
                sign += self.take()[4]
            other = self.sum()
            if sign == "=":
                return self.same(value, other)
            if sign == "<>":
                return not self.same(value, other)
            a, b = self.number(value), self.number(other)
            if a is WRONG or b is WRONG:
                return WRONG
            return {"<": a < b, ">": a > b, "<=": a <= b, ">=": a >= b}[sign]
        return value

    def sum(self):
        value = self.product()
        while self.peek()[4] in ("+", "-"):
            sign = self.take()[4]
            other = self.product()
            value = self.worked(value, other, (lambda a, b: a + b) if sign == "+" else (lambda a, b: a - b))
        return value

    def product(self):
        value = self.atom()
        while self.peek()[4] in ("*", "/"):
            sign = self.take()[4]
            other = self.atom()
            value = self.worked(value, other, (lambda a, b: a * b) if sign == "*" else (lambda a, b: a / b))
        return value

    def reference(self, text):
        sheet, _, ref = text.rpartition("!")
        return (sheet or self.sheet), ref

    def atom(self):
        number, quoted, ref, function, other = self.take()
        if number is not None:
            return float(number)
        if quoted is not None:
            return quoted
        if ref is not None:
            sheet, ref = self.reference(ref)
            if ":" in ref:
                return ("range", sheet, *ref.split(":"))
            return self.book.cell(sheet, ref)
        if function is not None:
            self.take()  # (
            args = [self.compare()]
            while self.peek()[4] == ",":
                self.take()
                args.append(self.compare())
            self.take()  # )
            return self.call(function, args)
        if other == "(":
            value = self.compare()
            self.take()
            return value
        if other == "-":
            return -self.atom()
        raise SystemExit(f"FAIL: cannot read the formula ={self.text}")

    def call(self, name, args):
        spread = lambda a: self.book.cells(*a[1:]) if isinstance(a, tuple) else [a]
        if name == "ROUND":
            x = self.number(args[0])
            return WRONG if x is WRONG else rounded(x, int(args[1]))
        if name == "IF":
            return args[1] if args[0] else args[2]
        if name == "OR":
            return any(args)
        if name == "COUNT":
            # How many of the cells hold a figure: a word ("none") and an empty cell do not count.
            return float(sum(1 for a in args for v in spread(a) if isinstance(v, (int, float)) and not isinstance(v, bool)))
        if name in ("COUNTIFS", "COUNTIF"):
            # Each pair is a range and what a cell of it has to be: a text it equals, or
            # ">=0" for a figure that is zero or more.
            def meets(v, wanted):
                if isinstance(wanted, str) and wanted.startswith(">="):
                    return isinstance(v, (int, float)) and not isinstance(v, bool) and v >= float(wanted[2:])
                return v == wanted
            columns = [(spread(args[i]), args[i + 1]) for i in range(0, len(args), 2)]
            return float(sum(1 for row in zip(*[c for c, _ in columns]) if all(meets(v, w) for v, (_, w) in zip(row, columns))))
        if name == "SUM":
            return sum(v for a in args for v in spread(a) if isinstance(v, (int, float)))
        if name == "SUMIF":
            keys, wanted, values = spread(args[0]), args[1], spread(args[2])
            return sum(v for k, v in zip(keys, values) if k == wanted and isinstance(v, (int, float)))
        raise SystemExit(f"FAIL: ={self.text} uses {name}, which this does not know")


def main():
    path = sys.argv[1]
    book = Book(path)
    if "--set" in sys.argv:
        sheet, element, line, new = sys.argv[sys.argv.index("--set") + 1:][:4]
        for row in book.formulas[sheet].iter_rows():
            if row[0].value == element and row[1].value == line:
                was = row[5].value
                amount_was = book.written[sheet][row[6].coordinate].value
                row[5].value = float(new)
                book.memo.clear()
                show = lambda v, f: "none" if v is None or v == "" else format(v, f)
                print(f"{sheet}: {element}, {line}: the price {show(was, 'g')} changed to {float(new):g}; its amount {show(amount_was, '.2f')} is now {book.cell(sheet, row[6].coordinate):.2f}")
                break
        else:
            raise SystemExit(f"FAIL: no line {line!r} on {element!r} in the sheet {sheet}")
    else:
        worked = 0
        for sheet in book.formulas.sheetnames:
            for row in book.formulas[sheet].iter_rows():
                for c in row:
                    if isinstance(c.value, str) and c.value.startswith("="):
                        value = book.cell(sheet, c.coordinate)
                        if value is WRONG:
                            raise SystemExit(f"FAIL: {sheet}!{c.coordinate} {c.value} does arithmetic on a word: a workbook would show #VALUE!")
                        written = book.written[sheet][c.coordinate].value
                        # A formula that comes to a word ("none") or to nothing has that written with it.
                        # What is written with a formula that comes to a word or to nothing is that
                        # word or nothing, never a 0 (which would show as $0.00).
                        if isinstance(value, str) or value is None:
                            same = (written is None or isinstance(written, str)) and (value or "") == (written or "")
                        else:
                            same = isinstance(written, (int, float)) and abs(value - float(written)) <= 0.004
                        if not same:
                            raise SystemExit(f"FAIL: {sheet}!{c.coordinate} {c.value} comes to {value!r}, and what is written with it is {written!r}")
                        worked += 1
        print(f"{len(book.formulas.sheetnames)} sheets: {', '.join(book.formulas.sheetnames)}")
        print(f"{worked} formulas worked out; each comes to what is written with it")
    for sheet in ("Recap", "Divisions"):
        if sheet not in book.formulas.sheetnames:
            continue
        for row in book.formulas[sheet].iter_rows(min_row=4):
            label, cell = row[0].value, row[1]
            if label is None:
                continue
            if cell.value is None:
                print(label)
            elif isinstance(cell.value, str) and not cell.value.startswith("="):
                print(f"{sheet}!{label}: {cell.value}")
            else:
                value = book.cell(sheet, cell.coordinate)
                if isinstance(value, str) or value is None:
                    print(f"{sheet}!{label}: {value or 'nothing'}")
                else:
                    print(f"{sheet}!{label} = {value:.2f}" if "percent" not in label else f"{sheet}!{label} = {value:g}")


if __name__ == "__main__":
    main()
