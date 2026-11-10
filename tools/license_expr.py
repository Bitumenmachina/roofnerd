#!/usr/bin/env python3
"""Decide whether a license expression can be met with permissive licenses alone.

An expression such as "(MIT OR Apache-2.0) AND Unicode-3.0" is parsed properly:
brackets first, then AND, then OR. "A OR B" passes if either side passes. "A AND B"
passes only if both do. So "GPL-3.0 AND (MIT OR Apache-2.0)" fails, because the GPL
part is not optional. The old "MIT/Apache-2.0" spelling means OR.

Reads lines of name|version|license on standard input, prints a table, and exits 1 if
any crate fails. With --self-test it checks itself instead.
"""
import re
import sys

PERMISSIVE = {
    "MIT", "MIT-0", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Zlib", "0BSD",
    "Unicode-3.0", "Unicode-DFS-2016", "CC0-1.0", "BSL-1.0", "Unlicense",
}


def tokens(expr):
    return re.findall(r"\(|\)|[^\s()]+", expr.replace("/", " OR "))


def permissive(expr):
    toks = tokens(expr)
    pos = 0

    def peek():
        return toks[pos] if pos < len(toks) else None

    def take():
        nonlocal pos
        pos += 1
        return toks[pos - 1]

    def either():  # a OR b OR c
        ok = both()
        while peek() == "OR":
            take()
            ok = both() or ok
        return ok

    def both():  # a AND b AND c
        ok = one()
        while peek() == "AND":
            take()
            ok = one() and ok
        return ok

    def one():
        t = take() if peek() is not None else None
        if t is None:
            raise ValueError("expression ends early")
        if t == "(":
            ok = either()
            if peek() != ")":
                raise ValueError("unclosed bracket")
            take()
            return ok
        if t in (")", "AND", "OR", "WITH"):
            raise ValueError(f"unexpected {t}")
        ok = t in PERMISSIVE
        if peek() == "WITH":  # an exception clause does not make a license permissive
            take()
            take()
        return ok

    try:
        ok = either()
        return ok and pos == len(toks)
    except (ValueError, IndexError):
        return False


def self_test():
    cases = {
        "MIT": True,
        "MIT OR Apache-2.0": True,
        "MIT/Apache-2.0": True,
        "(MIT OR Apache-2.0) AND Unicode-3.0": True,
        "Unlicense OR MIT": True,
        "GPL-3.0 OR MIT": True,  # the MIT option can be taken
        "GPL-3.0": False,
        "AGPL-3.0-only": False,
        "LGPL-2.1": False,
        "MPL-2.0": False,
        "GPL-3.0 AND (MIT OR Apache-2.0)": False,
        "(MIT OR Apache-2.0) AND GPL-3.0-only": False,
        "LGPL-3.0-or-later AND (MIT OR Apache-2.0)": False,
        "MIT AND GPL-2.0-or-later": False,
        "Apache-2.0 WITH LLVM-exception": True,
        "GPL-2.0 WITH Classpath-exception-2.0": False,
        "": False,
        "License": False,
        "(MIT": False,
    }
    bad = [(e, want, permissive(e)) for e, want in cases.items() if permissive(e) != want]
    for e, want, got in bad:
        print(f"SELF-TEST FAIL: {e!r}: expected {want}, got {got}")
    print(f"self-test: {len(cases) - len(bad)} of {len(cases)} cases correct")
    return 1 if bad else 0


def main():
    if "--self-test" in sys.argv:
        return self_test()
    failed = 0
    print("| Crate | Version | License |")
    print("|---|---|---|")
    for line in sys.stdin:
        line = line.rstrip("\n")
        if not line:
            continue
        name, version, lic = (line.split("|") + ["", ""])[:3]
        print(f"| {name} | {version} | {lic} |")
        if not permissive(lic):
            print(f"NOT PERMISSIVE: {name} {version} ({lic})", file=sys.stderr)
            failed = 1
    return failed


if __name__ == "__main__":
    sys.exit(main())
