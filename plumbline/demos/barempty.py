# A message put away is put away (call 178 item 4): the bar along the foot of the model window
# says nothing of it. The bar's words are lettered in a tone's colour; with the card put away (or
# up) none of that colour is left in the bar. Usage: barempty.py <picture>... ; each must have no
# coloured ink (largest channel less smallest over 60) in the bar, x 20 to 640, y 925 to 950.
import sys
from PIL import Image
ok = True
for path in sys.argv[1:]:
    im = Image.open(path).convert("RGB").crop((20, 925, 640, 950))
    n = sum(1 for p in im.getdata() if max(p) - min(p) > 60)
    good = n < 6
    ok = ok and good
    print("    bar of %s: %d coloured pixels, wanted none: %s" % (path.split("/")[-2] + "/" + path.split("/")[-1], n, "as wanted" if good else "FAIL"))
sys.exit(0 if ok else 1)
