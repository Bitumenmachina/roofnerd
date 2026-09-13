#!/usr/bin/env python3
"""The landing chain's own check of an exported roof, run against the industry's
reference toolkit rather than against this program's account of itself.

`packages/app/src/ifc/write.ts` writes the bytes; `expectedOf` in the same
file writes `<name>.expected.json` beside them, off the same solve — never off
the bytes. This script opens the `.ifc` with IfcOpenShell, counts entities by
class, reads every `Qto_*` quantity and every `Pset_*` property the expected
file names, and says PASS or FAIL for each one it can find and each one it
cannot. It also runs IfcOpenShell's own schema validator and reports what it
finds as findings, not as failures: a roof this program built and a table
this program did not write disagreeing about how many decimal places an
INTEGER prints is not a mismatched quantity.

No client data: everything this script prints is a name, a class, a quantity
name and two numbers, none of it read from `fixtures/`, `refs/` or
`evidence/`. No network: `ifcopenshell.open` reads the one path it is given
and nothing else.

Usage:
    ~/venvs/ifc/bin/python tools/verify-ifc.py <file.ifc> <expected.json>

Exit status: 0 if every count and every entry matched; 1 if any did not or
the file would not open; 2 for a usage error.
"""

from __future__ import annotations

import json
import logging
import math
import sys
from pathlib import Path
from typing import Any

import ifcopenshell
import ifcopenshell.util.element as element_util


def close_enough(got: Any, want: Any) -> bool:
    """Numbers within a thousandth (this writer's own coordinate rounding,
    `triangulate.ts`'s vertex-merge tolerance among them); everything else,
    including booleans, exactly."""
    if isinstance(want, bool) or isinstance(got, bool):
        return got == want
    if isinstance(want, (int, float)) and isinstance(got, (int, float)):
        return math.isclose(float(got), float(want), rel_tol=1e-6, abs_tol=1e-3)
    return got == want


def by_name_and_class(model: ifcopenshell.file, classes: set[str]) -> dict[tuple[str, str], Any]:
    """Every instance of the named classes, keyed by (class, its own Name) —
    the writer's own way of telling apart several elements sharing one
    condition's Tag (`ExpectedEntry`'s own doc comment in `write.ts`)."""
    out: dict[tuple[str, str], Any] = {}
    for cls in classes:
        for inst in model.by_type(cls):
            name = getattr(inst, "Name", None)
            if name is not None:
                out[(cls, name)] = inst
    return out


def check_counts(model: ifcopenshell.file, counts: dict[str, int]) -> bool:
    ok = True
    for cls, want in counts.items():
        try:
            got = len(model.by_type(cls))
        except RuntimeError as e:
            print(f"FAIL  count {cls}: {e}")
            ok = False
            continue
        status = "PASS" if got == want else "FAIL"
        if status == "FAIL":
            ok = False
        print(f"{status}  count {cls}: {got} (expected {want})")
    return ok


def check_entries(model: ifcopenshell.file, entries: list[dict[str, Any]]) -> bool:
    ok = True
    classes = {e["class"] for e in entries}
    index = by_name_and_class(model, classes)
    for e in entries:
        inst = index.get((e["class"], e["name"]))
        row = f"{e['name']} · {e['class']} · {e['set']}.{e['key']}"
        if inst is None:
            print(f"FAIL  {row}: no such element in the file")
            ok = False
            continue
        psets = element_util.get_psets(inst)
        found_set = psets.get(e["set"])
        if found_set is None:
            print(f"FAIL  {row}: no {e['set']} on this element")
            ok = False
            continue
        if e["key"] not in found_set:
            print(f"FAIL  {row}: {e['set']} carries no {e['key']}")
            ok = False
            continue
        got = found_set[e["key"]]
        want = e["value"]
        if close_enough(got, want):
            print(f"PASS  {row}: {got}")
        else:
            print(f"FAIL  {row}: file says {got!r}, expected.json says {want!r}")
            ok = False
    return ok


def run_schema_validation(model: ifcopenshell.file) -> list[str]:
    """IfcOpenShell's own validator, collected rather than printed as it goes —
    reported as findings, never folded into the PASS/FAIL count above. A
    finding here is a fact about how strictly the file matches the EXPRESS
    schema; it is not a claim about whether a quantity is the solve's own."""
    try:
        import ifcopenshell.validate as validate
    except ImportError:
        print("REPORT  this copy of IfcOpenShell has no validate module; skipped")
        return []

    records: list[str] = []

    class _Collect(logging.Handler):
        def emit(self, record: logging.LogRecord) -> None:
            records.append(self.format(record))

    logger = logging.getLogger("roofnerd.verify_ifc")
    logger.setLevel(logging.DEBUG)
    logger.handlers = [_Collect()]
    logger.propagate = False

    validate.validate(model, logger, express_rules=True)
    return records


def main(argv: list[str]) -> int:
    if len(argv) != 3:
        print(f"usage: {argv[0]} <file.ifc> <expected.json>", file=sys.stderr)
        return 2

    ifc_path = Path(argv[1])
    expected_path = Path(argv[2])

    try:
        model = ifcopenshell.open(str(ifc_path))
    except Exception as e:  # noqa: BLE001 - the reason a file will not open is the report
        print(f"FAIL  {ifc_path} would not open: {e}")
        return 1

    try:
        expected = json.loads(expected_path.read_text())
    except Exception as e:  # noqa: BLE001
        print(f"FAIL  {expected_path} would not read as JSON: {e}")
        return 1

    print(f"verifying {ifc_path.name} against {expected_path.name}")
    print(f"job: {expected.get('job', '?')}  schema: {model.schema}")
    print()

    counts_ok = check_counts(model, expected.get("counts", {}))
    print()
    entries_ok = check_entries(model, expected.get("entries", []))
    print()

    findings = run_schema_validation(model)
    if findings:
        print(f"schema validation: {len(findings)} finding(s) (reported, not failed on)")
        for f in findings:
            # The informative lines are the ones naming the rule or the
            # attribute and what it found; "For instance:" and the STEP line
            # under it are the same for every finding on one entity and say
            # nothing on their own.
            lines = [ln.strip() for ln in f.splitlines() if ln.strip()]
            at = next(
                (i for i, ln in enumerate(lines) if "attribute" in ln.lower() or ln.lower().startswith("rule")),
                None,
            )
            said = " ".join(lines[at:at + 2]) if at is not None else (lines[0] if lines else f)
            print(f"  • {said}")
    else:
        print("schema validation: no findings")

    ok = counts_ok and entries_ok
    print()
    print("PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
