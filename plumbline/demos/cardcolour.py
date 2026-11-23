# What a message card's colour says (call 178 item 5): the colour of its bar and first line, read
# off the picture the window saved. Green: done. Blue: a question, or the next thing to type.
# Amber: a caution. Red: refused or wrong. Usage: cardcolour.py <picture> <word> [<picture> <word> ...]
import sys
from PIL import Image
INKS = {"green": (0x1F, 0x7A, 0x4D), "blue": (0x1D, 0x4E, 0xD8), "amber": (0xA9, 0x69, 0x0A), "red": (0xB9, 0x32, 0x27)}
def read(path):
    # The card's bar is a column of its colour, 4 pixels wide and as tall as the card: the longest
    # vertical run of each ink in one column is read (a button's blue edge, or the red of a dashed
    # wall on the plan below, is not a run that long).
    im = Image.open(path).convert("RGB").crop((270, 110, 1230, 330))
    w, h = im.size
    px = im.load()
    best = {k: 0 for k in INKS}
    for x in range(w):
        run = {k: 0 for k in INKS}
        for y in range(h):
            for k, c in INKS.items():
                if all(abs(px[x, y][i] - c[i]) <= 14 for i in range(3)):
                    run[k] += 1
                    best[k] = max(best[k], run[k])
                else:
                    run[k] = 0
    top = max(best, key=best.get)
    return (top if best[top] >= 20 else "none"), best
ok = True
a = sys.argv[1:]
for path, want in zip(a[::2], a[1::2]):
    got, n = read(path)
    good = got == want
    ok = ok and good
    print("    card in %s: %s, wanted %s %s (longest vertical run of its ink, in pixels: %s)" % (path.split("/")[-2] + "/" + path.split("/")[-1], got, want, "as wanted" if good else "FAIL", n))
sys.exit(0 if ok else 1)
