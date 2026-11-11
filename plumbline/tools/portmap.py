#!/usr/bin/env python3
"""The port map gate. It exists because the building model was once written without
opening the program it should have been ported from (DECISIONS.md call 28).

Every public item of the building model has to be accounted for in PORTMAP.md as one of:

  ported   taken from BIMCraft. The row names the file and line there, and that line has
           to hold the symbol the row names, in the clone at the pinned commit.
  ours     not in BIMCraft. The row says what was searched for and where, so the claim
           that it is not there can be checked by running the same search.

A row is:  | item | ported or ours | BIMCraft symbol | file:line | searched |

It fails when an item is not in the map, when a cited line does not hold the symbol, when
an "ours" row says nothing was searched, when a file holding ported code does not say so
in its header and in NOTICE, or when BIMCraft is not there to check against. It cannot be
passed from memory.

  tools/portmap.sh              check
  BIMCRAFT=/path tools/portmap.sh
"""
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PIN = "f202abd85d1cf165325e2f7b250da0b5e51f5b71"
ITEM = re.compile(r"^\s*pub (?:struct|enum|fn|const|type|trait) ([A-Za-z_][A-Za-z0-9_]*)", re.M)


def building_files():
    src = os.path.join(ROOT, "crates", "world", "src")
    out = []
    for dirpath, _, names in os.walk(src):
        for n in sorted(names):
            p = os.path.join(dirpath, n)
            rel = os.path.relpath(p, src)
            # The building model, and (since HANDOFF_DRAW.md) what draws and edits it.
            if n.endswith(".rs") and rel.startswith(("building", "editing")):
                out.append(p)
    return out


NOTHING = re.compile(r"`(grep [^`]+)`(?: over BIMCraft)?: nothing")
EACH = re.compile(r"`(grep -c [^`]+)` in each of ((?:[\w./-]+\.rs ?)+): 0")


def run_searches(bim, searched):
    """Run each search an `ours` row says found nothing, in BIMCraft's source. Two forms
    are run: `grep ...`: nothing   and   `grep -c ...` in each of a.rs b.rs: 0.
    A path that starts with crates/ (or is .) is from BIMCraft's root; any other is from
    its crates/core/src. Returns what is wrong."""
    import shlex
    out = []
    core = os.path.join(bim, "crates", "core", "src")

    def run(command, extra=()):
        try:
            words = shlex.split(command) + list(extra)
        except ValueError as e:
            return None, f"the search `{command}` cannot be read as a command: {e}"
        if not words or words[0] != "grep":
            return None, f"the search `{command}` is not a grep"
        from_root = any(w == "." or w.startswith("crates/") for w in words[1:])
        try:
            r = subprocess.run(words, cwd=bim if from_root else core, capture_output=True, text=True, timeout=120)
        except Exception as e:
            return None, f"the search `{command}` could not be run: {e}"
        if r.returncode > 1:
            return None, f"the search `{command}` failed: {r.stderr.strip()[:120]}"
        return r.stdout, None

    for command in NOTHING.findall(searched):
        got, why = run(command)
        if why:
            out.append(why)
        elif got.strip():
            first = got.strip().split("\n")[0][:110]
            out.append(f"the search `{command}` is said to find nothing and finds: {first}")
    for command, files in EACH.findall(searched):
        for f in files.split():
            got, why = run(command, [f])
            if why:
                out.append(why)
            elif got.strip() != "0":
                out.append(f"the search `{command}` is said to count 0 in {f} and counts {got.strip()}")
    return out


def main():
    faults = []
    bim = os.environ.get("BIMCRAFT") or os.path.normpath(os.path.join(ROOT, "..", "..", "ipedrax", "bimcraft"))
    if not os.path.isdir(os.path.join(bim, "crates", "core", "src")):
        print(f"portmap: BIMCraft is not at {bim}. Clone https://github.com/IPedrax/bimcraft at {PIN}")
        print("         and set BIMCRAFT to it. The map cannot be checked, or written, without the source.")
        return 2
    head = subprocess.run(["git", "-C", bim, "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip()
    if head != PIN:
        faults.append(f"BIMCraft at {bim} is at {head[:12]}, not the pinned {PIN[:12]}: line numbers in the map are for the pin")

    map_path = os.path.join(ROOT, "PORTMAP.md")
    rows = {}
    if not os.path.exists(map_path):
        faults.append("PORTMAP.md is missing")
    else:
        for n, line in enumerate(open(map_path, encoding="utf-8"), 1):
            cells = [c.strip() for c in line.strip().strip("|").split("|")]
            if not line.startswith("|") or len(cells) < 5 or cells[0] in ("item", "") or set(cells[0]) <= set("-: "):
                continue
            item = cells[0].strip("`")
            if item in rows:
                faults.append(f"PORTMAP.md:{n}: {item} is listed twice")
            rows[item] = (n, cells[1].lower(), cells[2].strip("`"), cells[3].strip("`"), cells[4])

    # Every public item of the building model is in the map.
    files = building_files()
    seen = set()
    for path in files:
        text = open(path, encoding="utf-8").read()
        rel = os.path.relpath(path, ROOT)
        for name in ITEM.findall(text):
            seen.add(name)
            if name not in rows:
                faults.append(f"{rel}: pub item `{name}` is not in PORTMAP.md")
    for item, (n, *_rest) in rows.items():
        if item not in seen:
            faults.append(f"PORTMAP.md:{n}: `{item}` is in the map but is not a public item of the building model")

    ported_from = set()
    for item, (n, how, symbol, where, searched) in rows.items():
        if how == "ported":
            m = re.fullmatch(r"(crates/[\w./-]+\.rs):(\d+)", where)
            if not m:
                faults.append(f"PORTMAP.md:{n}: {item}: a ported row cites `crates/.../file.rs:LINE`; it has {where!r}")
                continue
            src = os.path.join(bim, m.group(1))
            if not os.path.exists(src):
                faults.append(f"PORTMAP.md:{n}: {item}: {m.group(1)} is not in BIMCraft")
                continue
            lines = open(src, encoding="utf-8", errors="replace").read().split("\n")
            at = int(m.group(2))
            if not symbol or at < 1 or at > len(lines) or not re.search(rf"\b{re.escape(symbol)}\b", lines[at - 1]):
                got = lines[at - 1].strip()[:90] if 1 <= at <= len(lines) else "no such line"
                faults.append(f"PORTMAP.md:{n}: {item}: {where} does not hold `{symbol}`; it holds: {got}")
            ported_from.add(m.group(1))
        elif how == "ours":
            if len(searched) < 12 or "grep" not in searched and "read" not in searched.lower():
                faults.append(f"PORTMAP.md:{n}: {item}: an `ours` row says what was searched for and where (a grep, or the files read); it has {searched!r}")
            # A search a row says came back with nothing is run, and has to come back with
            # nothing. (The ninth judgment found two rows whose search finds something; the
            # gate had only checked that a search was named.)
            for fault in run_searches(bim, searched):
                faults.append(f"PORTMAP.md:{n}: {item}: {fault}")
        else:
            faults.append(f"PORTMAP.md:{n}: {item}: is `ported` or `ours`, not {how!r}")

    # Ported code says so where it lives, and in NOTICE.
    if ported_from:
        notice = open(os.path.join(ROOT, "NOTICE"), encoding="utf-8").read()
        for path in files:
            rel = os.path.relpath(path, ROOT)
            text = open(path, encoding="utf-8").read()
            names = set(ITEM.findall(text))
            if any(rows.get(nm, (0, ""))[1] == "ported" for nm in names):
                if "BIMCraft" not in text[:3000] or PIN[:12] not in text[:3000]:
                    faults.append(f"{rel}: holds ported items but its header does not say it is ported from BIMCraft at {PIN[:12]}")
                if rel not in notice:
                    faults.append(f"NOTICE does not name {rel}, which holds code ported from BIMCraft")

    if faults:
        print(f"portmap: FAIL, {len(faults)} fault(s)")
        for f in faults[:80]:
            print("  " + f)
        if len(faults) > 80:
            print(f"  ... and {len(faults) - 80} more")
        return 1
    ported = sum(1 for r in rows.values() if r[1] == "ported")
    print(f"portmap: PASS. {len(rows)} public items: {ported} ported from BIMCraft at {PIN[:12]}, {len(rows) - ported} ours, each with its search.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
