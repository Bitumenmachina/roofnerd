#!/usr/bin/env python3
"""An IFC model checked by IfcOpenShell, which is not part of this program.

    tools/ifc-check.py <model.ifc>

Validates the file against its schema, and against the schema's own rules where
IfcOpenShell can run them here (that needs pytest), and counts what a building has.
Exits 1 on any fault, printing each. IfcOpenShell is used as a checker only; nothing of
it is linked into the program.
"""
import collections
import sys

import ifcopenshell
import ifcopenshell.validate


def main():
    model = ifcopenshell.open(sys.argv[1])
    said = "against the schema and its rules"
    logger = ifcopenshell.validate.json_logger()
    try:
        ifcopenshell.validate.validate(model, logger, express_rules=True)
    except ModuleNotFoundError:
        said = "against the schema (its rules could not be run here: pytest is not installed)"
        logger = ifcopenshell.validate.json_logger()
        ifcopenshell.validate.validate(model, logger)
    if logger.statements:
        faults = collections.Counter(str(s.get("message", "")).split("\n")[0][:200] for s in logger.statements)
        print(f"IfcOpenShell {ifcopenshell.version}: {model.schema}: {len(logger.statements)} faults {said}")
        for fault, n in faults.most_common():
            print(f"  {n} x {fault}")
        sys.exit(1)
    print(f"IfcOpenShell {ifcopenshell.version}: {model.schema}: no fault {said}")
    count = lambda cls: len(model.by_type(cls))
    floors = len([s for s in model.by_type("IfcSlab") if s.PredefinedType == "FLOOR"])
    print(f"{count('IfcBuildingStorey')} IfcBuildingStorey, {count('IfcWall')} IfcWall, {count('IfcDoor')} IfcDoor, {count('IfcWindow')} IfcWindow, {floors} IfcSlab that are floors, {count('IfcSpace')} IfcSpace, {count('IfcRoof')} IfcRoof")
    runs = sum(count(c) for c in ("IfcDuctSegment", "IfcPipeSegment", "IfcCableCarrierSegment", "IfcCableSegment"))
    print(f"{runs} runs, {count('IfcDistributionSystem')} systems, {count('IfcPropertySet')} sets of values")
    for unit in model.by_type("IfcUnitAssignment")[0].Units:
        if unit.UnitType == "LENGTHUNIT":
            print(f"lengths in {unit.Name.lower()}")


if __name__ == "__main__":
    main()
