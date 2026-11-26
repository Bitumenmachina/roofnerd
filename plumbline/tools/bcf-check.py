#!/usr/bin/env python3
"""An issue list (BCF 2.1) checked against buildingSMART's own published schema files, by a program that is not part of this one.

    tools/bcf-check.py <issues.bcfzip>

Checks that the zip has `bcf.version`, that each issue's folder has the markup and each viewpoint it names, and that every file is
valid against buildingSMART's version.xsd, markup.xsd and visinfo.xsd (release 2.1 of its BCF-XML repository). The schema files are
buildingSMART's, published under CC BY-ND 4.0 (no derivatives): they are fetched here once into a cache folder
(`$BCF_SCHEMAS`, else `~/.cache/bcf-2.1`) and are never kept in this repository. lxml is the checker; nothing of it is linked into the
program, and the program opens no connection: this script does the fetching.

Exit 0: valid. Exit 1: a fault, each printed. Exit 2: it could not be checked (no lxml, or the schema files could not be fetched and
are not in the cache): said, and that is not a pass.
"""
import os
import re
import sys
import urllib.request
import zipfile

BASE = "https://raw.githubusercontent.com/buildingSMART/BCF-XML/release_2_1/Schemas/"
FILES = ["version.xsd", "markup.xsd", "visinfo.xsd"]
GUID = re.compile(r"^[a-fA-F0-9]{8}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{12}$")


def cannot(why):
    print(f"NOT CHECKED: {why}")
    sys.exit(2)


def schemas():
    folder = os.environ.get("BCF_SCHEMAS") or os.path.expanduser("~/.cache/bcf-2.1")
    os.makedirs(folder, exist_ok=True)
    for name in FILES:
        path = os.path.join(folder, name)
        if not os.path.exists(path):
            try:
                with urllib.request.urlopen(BASE + name, timeout=30) as r:
                    data = r.read()
            except Exception as e:  # noqa: BLE001
                cannot(f"{name} is not in {folder} and could not be fetched ({e})")
            with open(path, "wb") as f:
                f.write(data)
    return folder


def main():
    try:
        from lxml import etree
    except ModuleNotFoundError:
        cannot("lxml is not installed (pip install lxml)")
    folder = schemas()
    load = lambda n: etree.XMLSchema(etree.parse(os.path.join(folder, n)))  # noqa: E731
    version, markup, visinfo = load("version.xsd"), load("markup.xsd"), load("visinfo.xsd")
    faults = []
    z = zipfile.ZipFile(sys.argv[1])
    names = z.namelist()
    if "bcf.version" not in names:
        faults.append("no bcf.version at the root")
    topics = {}
    for n in names:
        if n == "bcf.version":
            schema, what = version, n
        elif n.endswith("/markup.bcf"):
            schema, what = markup, n
            topics[n.split("/")[0]] = n
        elif n.endswith(".bcfv"):
            schema, what = visinfo, n
        else:
            faults.append(f"{n}: a file the format does not have")
            continue
        try:
            doc = etree.fromstring(z.read(n))
        except etree.XMLSyntaxError as e:
            faults.append(f"{n}: not well-formed: {e}")
            continue
        if not schema.validate(doc):
            for e in schema.error_log:
                faults.append(f"{what}: line {e.line}: {e.message}")
        if n == "bcf.version" and doc.get("VersionId") != "2.1":
            faults.append(f"bcf.version says {doc.get('VersionId')!r}, not 2.1")
        if n.endswith("/markup.bcf"):
            if not GUID.match(n.split("/")[0]):
                faults.append(f"{n}: the folder is not named by the issue's identifier")
            if doc.find("Topic").get("Guid") != n.split("/")[0]:
                faults.append(f"{n}: the folder is not the topic's Guid")
            for vp in doc.findall("Viewpoints"):
                want = f"{n.split('/')[0]}/{vp.findtext('Viewpoint')}"
                if want not in names:
                    faults.append(f"{n}: names {vp.findtext('Viewpoint')} and it is not in the zip")
    if faults:
        print(f"{len(faults)} faults against buildingSMART's BCF 2.1 schema")
        for f in faults:
            print(f"  {f}")
        sys.exit(1)
    print(f"lxml {etree.__version__}: {len(topics)} issues, {len(names)} files: no fault against buildingSMART's BCF 2.1 schema (version.xsd, markup.xsd, visinfo.xsd)")


if __name__ == "__main__":
    main()
