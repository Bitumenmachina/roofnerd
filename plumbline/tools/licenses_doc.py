#!/usr/bin/env python3
"""Write, or check, the part of LICENSES.md that lists what is linked.

    licenses_doc.py <table file> <cargo metadata file> --write | --check

Called by tools/licenses.sh. The table of crates, its count and the list of direct
dependencies were kept by hand until 2026-11-26 and had fallen four crates and a hundred
rows behind what was linked (DECISIONS.md call 238). The rest of LICENSES.md (candidates not
linked, borrowed code, fonts) is prose and stays by hand.
"""
import collections
import json
import sys

START = "<!-- written by tools/licenses.sh --write; not edited by hand -->"
END = "<!-- end of what tools/licenses.sh writes -->"


def main() -> int:
    table = [line.rstrip("\n") for line in open(sys.argv[1]) if line.startswith("|")]
    rows = table[2:]
    names = collections.Counter(r.split("|")[1].strip() for r in rows)
    many = sorted(n for n, c in names.items() if c > 1)
    direct = []
    for p in sorted(json.load(open(sys.argv[2]))["packages"], key=lambda p: p["name"]):
        deps = sorted(
            {f"`{d['name']}` {d['req'].lstrip('^')}" for d in p["dependencies"]
             if d.get("kind") in (None, "build") and not d["name"].startswith("plumbline")}
        )
        if deps:
            direct.append(f"- `{p['name']}`: " + ", ".join(deps))
    count = f"The table has {len(rows)} rows for {len(names)} crates"
    if many:
        count += f"; {len(many)} are linked in more than one version (" + ", ".join(f"`{m}`" for m in many) + ")"
    block = [START, count + ".", "",
             "Direct dependencies, by crate of this workspace (the rest come in through these):", "",
             *direct, "", *table, END]
    doc = open("LICENSES.md").read()
    if START in doc:
        new = doc[:doc.index(START)] + "\n".join(block) + doc[doc.index(END) + len(END):]
    else:  # the first time: what was kept by hand runs from the count to the next section
        new = doc[:doc.index("The table has ")] + "\n".join(block) + "\n\n" + doc[doc.index("## Candidates checked and not linked"):]
    if sys.argv[3] == "--check":
        if new != doc:
            print("LICENSES.md does not hold what is linked today: run plumbline/tools/licenses.sh --write")
            return 1
        print(f"LICENSES.md holds what is linked today ({len(rows)} rows)")
        return 0
    open("LICENSES.md", "w").write(new)
    print(f"LICENSES.md: {len(rows)} rows for {len(names)} crates")
    return 0


if __name__ == "__main__":
    sys.exit(main())
