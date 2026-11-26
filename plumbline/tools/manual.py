#!/usr/bin/env python3
"""The BuildOS manual: one page, changed in place (DECISIONS.md call 237, point 3).

    tools/manual.py take <folder of pictures> <commit> [demonstration ...]
        Copy the pictures the manual names (manual/pictures.tsv) out of a run's pictures
        (a demos/captured folder), each to its fixed name in manual/pictures/, and note the
        commit and the day it was taken on (manual/taken.tsv). With demonstrations named,
        only their pictures. A picture is replaced, never added beside. With the word
        `committed` for the commit, each picture is stamped with the commit that last
        changed it in the repository (for pictures kept there before 2026-11-26).
    tools/manual.py build
        Write manual/index.html from manual/manual.md (how each task is done), STATUS.md
        (the state: it is not written twice) and the pictures. The page opens with no
        connection.
    tools/manual.py build --check
        Fail if manual/index.html is not what build would write. The commit hook runs it.
    tools/manual.py page <file>
        The same page without its outer wrapper, for publishing as an artifact.

Run from anywhere. Needs Python's Pillow for `take` only.
"""
import datetime
import html
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MAN = os.path.join(ROOT, "manual")
E = html.escape


def rows(name):
    path = os.path.join(MAN, name)
    if not os.path.exists(path):
        return []
    return [line.rstrip("\n").split("\t") for line in open(path) if line.strip() and not line.startswith("#")]


def take(folder, commit, only):
    from PIL import Image

    taken = {r[0]: r for r in rows("taken.tsv")}
    day = datetime.date.today().isoformat()
    os.makedirs(os.path.join(MAN, "pictures"), exist_ok=True)
    done = 0
    for name, source, kind in rows("pictures.tsv"):
        if only and source.split("/")[0] not in only:
            continue
        if kind == "pair":
            parts = [os.path.join(folder, source + "-model.png"), os.path.join(folder, source + "-sheet.png")]
        else:
            parts = [os.path.join(folder, source)]
        if not all(os.path.exists(p) for p in parts):
            print(f"  no picture for {name}: {source} is not in {folder}")
            continue
        ims = [Image.open(p).convert("RGB") for p in parts]
        if len(ims) == 2:  # the two windows as they stand on a screen: side by side, a gap between
            h = max(i.height for i in ims)
            both = Image.new("RGB", (ims[0].width + 24 + ims[1].width, h), (24, 28, 31))
            both.paste(ims[0], (0, 0))
            both.paste(ims[1], (ims[0].width + 24, 0))
            im = both
        else:
            im = ims[0]
        if im.width > 1800:
            im = im.resize((1800, round(im.height * 1800 / im.width)), Image.LANCZOS)
        im.save(os.path.join(MAN, "pictures", name + ".jpg"), quality=84, optimize=True)
        if commit == "committed":  # a picture already in the repository: the commit and the day it was last changed
            import subprocess
            stamp = subprocess.run(["git", "log", "-1", "--format=%h %cs", "--", *parts], capture_output=True, text=True, cwd=ROOT).stdout.split()
            taken[name] = [name, stamp[0], stamp[1], f"{im.width}x{im.height}"]
        else:
            taken[name] = [name, commit, day, f"{im.width}x{im.height}"]
        done += 1
    with open(os.path.join(MAN, "taken.tsv"), "w") as out:
        out.write("# picture\tcommit it was taken on\tday\tsize. Written by tools/manual.py take.\n")
        for name in sorted(taken):
            out.write("\t".join(taken[name]) + "\n")
    print(f"{done} pictures taken into manual/pictures/ from {folder}, on {commit}")


def inline(text):
    text = E(text)
    text = re.sub(r"`([^`]+)`", r"<code>\1</code>", text)
    text = re.sub(r"\*\*([^*]+)\*\*", r"<b>\1</b>", text)
    return text


def slug(text):
    return re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-")


def to_html(md, taken, prefix=""):
    """A small part of Markdown: headings, paragraphs, lists, tables, and a picture line."""
    out, toc = [], []
    lines = md.split("\n")
    i = 0

    def para(buf):
        if buf:
            out.append("<p>" + inline(" ".join(s.strip() for s in buf)) + "</p>")

    buf = []
    while i < len(lines):
        line = lines[i]
        m = re.match(r"^(#{1,3}) (.*)$", line)
        pic = re.match(r"^\[picture: ([a-z0-9-]+) \| (.*)\]$", line)
        item = re.match(r"^(\s*)(-|\d+\.) (.*)$", line)
        if m:
            para(buf); buf = []
            level, text = len(m.group(1)), m.group(2)
            if level == 1:
                i += 1
                continue  # the page has its own title
            ident = prefix + slug(text)
            if level == 2:
                toc.append((ident, text))
            out.append(f'<h{level} id="{ident}">{inline(text)}</h{level}>')
        elif pic:
            para(buf); buf = []
            name, caption = pic.group(1), pic.group(2)
            t = taken.get(name)
            if t and os.path.exists(os.path.join(MAN, "pictures", name + ".jpg")):
                w, h = t[3].split("x")
                out.append(
                    f'<figure><a href="pictures/{name}.jpg"><img src="pictures/{name}.jpg" width="{w}" height="{h}" loading="lazy" alt="{E(caption)}"></a>'
                    f'<figcaption>{inline(caption)} <span class="stamp">Picture of commit <code>{E(t[1])}</code>, {E(t[2])}.</span></figcaption></figure>'
                )
            else:
                out.append(f'<figure class="none"><div class="nopic">No picture of the present build yet.</div><figcaption>{inline(caption)}</figcaption></figure>')
        elif line.startswith("|"):
            para(buf); buf = []
            table = []
            while i < len(lines) and lines[i].startswith("|"):
                table.append([c.strip() for c in lines[i].strip().strip("|").split("|")])
                i += 1
            head, body = table[0], [r for r in table[2:]]
            out.append('<div class="scroll"><table><thead><tr>' + "".join(f"<th>{inline(c)}</th>" for c in head) + "</tr></thead><tbody>")
            for r in body:
                out.append("<tr>" + "".join(f"<td>{inline(c)}</td>" for c in r) + "</tr>")
            out.append("</tbody></table></div>")
            continue
        elif item:
            para(buf); buf = []
            tag = "ul" if item.group(2) == "-" else "ol"
            items = []
            while i < len(lines):
                it = re.match(r"^(\s*)(-|\d+\.) (.*)$", lines[i])
                if it:
                    items.append(it.group(3))
                elif lines[i].startswith("  ") and lines[i].strip() and items:
                    items[-1] += " " + lines[i].strip()
                else:
                    break
                i += 1
            out.append(f"<{tag}>" + "".join(f"<li>{inline(t)}</li>" for t in items) + f"</{tag}>")
            continue
        elif not line.strip():
            para(buf); buf = []
        else:
            buf.append(line)
        i += 1
    para(buf)
    return "\n".join(out), toc


CSS = """
:root{--bg:#f3f1ea;--panel:#fbfaf6;--ink:#1c2226;--soft:#5b6469;--line:#d8d2c4;--line2:#e9e4d8;--blue:#1f4e79;--blue-bg:#e3ecf5;--warn:#8a5a00;--warn-bg:#f8ecc8;
--mono:ui-monospace,"SF Mono","Cascadia Mono",Menlo,Consolas,"Liberation Mono",monospace;--sans:system-ui,-apple-system,"Segoe UI",Roboto,"Helvetica Neue",Arial,sans-serif}
@media (prefers-color-scheme:dark){:root:not([data-theme="light"]){--bg:#14181b;--panel:#1b2125;--ink:#e7e4dc;--soft:#9aa3a8;--line:#333c42;--line2:#272e33;--blue:#8fbbe6;--blue-bg:#1d2f41;--warn:#e8bf5e;--warn-bg:#3a2e10;color-scheme:dark}}
:root[data-theme="dark"]{--bg:#14181b;--panel:#1b2125;--ink:#e7e4dc;--soft:#9aa3a8;--line:#333c42;--line2:#272e33;--blue:#8fbbe6;--blue-bg:#1d2f41;--warn:#e8bf5e;--warn-bg:#3a2e10;color-scheme:dark}
*{box-sizing:border-box}
body{margin:0;background:var(--bg);color:var(--ink);font:16px/1.55 var(--sans)}
.wrap{max-width:1180px;margin:0 auto;padding-block:26px 80px;padding-left:max(16px,env(safe-area-inset-left,0px));padding-right:max(16px,env(safe-area-inset-right,0px));display:grid;grid-template-columns:230px minmax(0,1fr);gap:34px}
header.sheet{grid-column:1/-1;border:2px solid var(--ink);background:var(--panel);display:grid;grid-template-columns:1fr auto}
header.sheet .t{padding:16px 20px}
header.sheet h1{margin:0 0 4px;font-size:28px;line-height:1.15;letter-spacing:-.01em}
header.sheet p{margin:0;color:var(--soft);max-width:60ch}
.block{border-left:2px solid var(--ink);display:grid;grid-template-columns:auto auto;font:12px/1.35 var(--mono);align-content:stretch}
.block div{padding:6px 12px;border-bottom:1px solid var(--line)}
.block div:nth-child(odd){color:var(--soft);text-transform:uppercase;letter-spacing:.06em;border-right:1px solid var(--line)}
.block div:nth-last-child(-n+2){border-bottom:0}
.block .flag{color:var(--warn);background:var(--warn-bg);font-weight:700}
nav{position:sticky;top:14px;align-self:start;font-size:14px;max-height:calc(100vh - 28px);overflow:auto}
nav b{display:block;font:11.5px var(--mono);text-transform:uppercase;letter-spacing:.07em;color:var(--soft);margin:14px 0 4px}
nav b:first-child{margin-top:0}
nav a{display:block;color:var(--ink);text-decoration:none;padding:3px 8px;border-left:2px solid var(--line)}
nav a:hover{border-left-color:var(--blue);color:var(--blue)}
main{min-width:0}
h2{font-size:21px;margin:40px 0 8px;padding-top:10px;border-top:2px solid var(--ink);text-wrap:balance}
main>h2:first-child{margin-top:0}
h3{font-size:16px;margin:22px 0 6px}
p,li{max-width:70ch}
p{margin:8px 0}
ol,ul{padding-left:24px;margin:8px 0}
li{margin:4px 0}
code{font:.9em var(--mono);background:var(--line2);padding:1px 4px;border-radius:3px;overflow-wrap:anywhere}
.scroll{overflow-x:auto;margin:12px 0}
table{border-collapse:collapse;width:100%;background:var(--panel);border:1px solid var(--line);font-size:14.5px}
th,td{text-align:left;vertical-align:top;padding:7px 10px;border-bottom:1px solid var(--line2)}
th{font:11.5px var(--mono);text-transform:uppercase;letter-spacing:.06em;color:var(--soft);border-bottom:1px solid var(--line)}
tr:last-child td{border-bottom:0}
figure{margin:16px 0;background:var(--panel);border:1px solid var(--line)}
figure img{display:block;width:100%;height:auto;max-width:100%}
figcaption{padding:8px 12px;font-size:14px;border-top:1px solid var(--line)}
.stamp{color:var(--soft);white-space:nowrap}
.nopic{padding:26px 12px;text-align:center;color:var(--warn);background:var(--warn-bg);font:13px var(--mono)}
.part{margin-top:56px;padding:10px 14px;background:var(--blue-bg);color:var(--blue);font:700 13px var(--mono);letter-spacing:.06em;text-transform:uppercase}
footer{grid-column:1/-1;border-top:1px solid var(--line);padding-top:10px;color:var(--soft);font-size:13px}
@media (max-width:860px){.wrap{grid-template-columns:minmax(0,1fr);gap:18px}nav{position:static;max-height:none;display:flex;flex-wrap:wrap;gap:2px 4px}nav b{flex-basis:100%}nav a{border-left:0;border-bottom:1px solid var(--line);padding:2px 6px}header.sheet{grid-template-columns:1fr}.block{border-left:0;border-top:2px solid var(--ink)}}
@media print{nav{display:none}.wrap{display:block}body{background:#fff}}
"""


def page():
    taken = {r[0]: r for r in rows("taken.tsv")}
    text = open(os.path.join(MAN, "manual.md")).read()
    head, rest = text.split("\n\n", 2)[1], text.split("\n\n", 2)[2]  # the first paragraph is the page's own heading
    manual, toc_a = to_html(rest, taken)
    status_md = open(os.path.join(ROOT, "STATUS.md")).read()
    status, toc_b = to_html(status_md, taken, prefix="state-")
    lead = re.search(r"^\*\*(As of [^*]+)\*\*", status_md, re.M)
    commits = sorted({t[1] for t in taken.values()})
    days = sorted({t[2] for t in taken.values()})
    pictures = (f"{len(taken)}, taken from {days[0]} to {days[-1]}" if len(days) > 1 else f"{len(taken)}, taken on {days[0]}") if taken else "none yet"
    intro = head
    nav = "<b>How each task is done</b>" + "".join(f'<a href="#{i}">{E(t)}</a>' for i, t in toc_a)
    nav += "<b>The state of the program</b>" + "".join(f'<a href="#{i}">{E(t)}</a>' for i, t in toc_b)
    body = f"""<div class="wrap">
<header class="sheet">
  <div class="t"><h1>BuildOS manual</h1><p>{inline(' '.join(intro.split()))}</p></div>
  <div class="block">
    <div>Program</div><div>BuildOS</div>
    <div>State</div><div class="flag">Not a release candidate</div>
    <div>As of</div><div>{E(lead.group(1)[6:].split(',')[0] if lead else '')}</div>
    <div>Pictures</div><div>{E(pictures)}</div>
  </div>
</header>
<nav>{nav}</nav>
<main>
{manual}
<div class="part">The state of the program (from STATUS.md)</div>
{status}
</main>
<footer>Written by <code>tools/manual.py build</code> from <code>manual/manual.md</code>, <code>STATUS.md</code> and <code>manual/pictures/</code>. It opens with no connection. No job, client or supplier data is on this page: every house and every price in a picture is made up.</footer>
</div>"""
    return f"<title>BuildOS Manual</title>\n<meta name=\"claude:viewport-fit-sides\" content=\"cover\">\n<style>{CSS}</style>\n{body}\n"


def full(inner):
    return '<!doctype html>\n<html lang="en">\n<head>\n<meta charset="utf-8">\n<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">\n' + inner.split("<div class=\"wrap\">")[0] + "</head>\n<body>\n<div class=\"wrap\">" + inner.split("<div class=\"wrap\">", 1)[1] + "</body>\n</html>\n"


def main(argv):
    if len(argv) >= 4 and argv[1] == "take":
        take(argv[2], argv[3], set(argv[4:]))
        return 0
    if len(argv) >= 2 and argv[1] == "build":
        doc = full(page())
        path = os.path.join(MAN, "index.html")
        if "--check" in argv:
            if not os.path.exists(path) or open(path).read() != doc:
                print("manual/index.html is not what the manual's text, STATUS.md and the pictures give: run plumbline/tools/manual.py build")
                return 1
            return 0
        open(path, "w").write(doc)
        print(f"manual/index.html: {len(doc):,} bytes, {len(rows('taken.tsv'))} pictures")
        return 0
    if len(argv) == 3 and argv[1] == "page":
        open(argv[2], "w").write(page())
        return 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
