#!/usr/bin/env python3
"""Regenerates the logo assets from assets/logo/colourways.json.

  python3 scripts/gen-logo.py [--slug lavender] [--icon-slug lavender]

Writes assets/logo/gitpoulpi.svg (+ .png at 256 and 512, nearest-neighbour) for the
README and the docs, and assets/logo/gitpoulpi.ico (16/32/48/64/128/256) for the Windows
executable. The colourway is picked by slug; the default is the file's "default". Needs
rsvg-convert and ImageMagick (magick). Run from anywhere.
"""
import argparse, json, pathlib, subprocess, tempfile

root = pathlib.Path(__file__).resolve().parent.parent
logo = root / "assets" / "logo"
data = json.loads((logo / "colourways.json").read_text())

def way(slug):
    for w in data["ways"]:
        if w["slug"] == slug:
            return w
    raise SystemExit(f"unknown colourway {slug!r}")

def svg(w):
    pal = {"O": w["ink"], "P": w["body"], "L": w["light"], "W": "#FFFFFF", "K": w["ink"],
           "C": w["cheek"], "T": w["tips"][0], "R": w["tips"][1], "S": w["tips"][2], "B": w["tips"][3]}
    rects = "".join(f'<rect x="{x}" y="{y}" width="1" height="1" fill="{pal[ch]}"/>'
                    for y, row in enumerate(data["grid"]) for x, ch in enumerate(row) if ch in pal)
    return ('<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" shape-rendering="crispEdges" '
            f'role="img" aria-label="GitPoulpi">{rects}</svg>\n')

def png(svg_path, out, size):
    subprocess.run(["rsvg-convert", "-w", str(size), "-h", str(size), "-o", str(out), str(svg_path)], check=True)

ap = argparse.ArgumentParser()
ap.add_argument("--slug", default=data["default"])
ap.add_argument("--icon-slug", default=None, help="colourway of the .ico (default: --slug)")
a = ap.parse_args()

(logo / "gitpoulpi.svg").write_text(svg(way(a.slug)))
png(logo / "gitpoulpi.svg", logo / "gitpoulpi.png", 256)
png(logo / "gitpoulpi.svg", logo / "gitpoulpi-512.png", 512)
with tempfile.TemporaryDirectory() as tmp:
    icon_svg = pathlib.Path(tmp) / "icon.svg"
    icon_svg.write_text(svg(way(a.icon_slug or a.slug)))
    sizes = [16, 32, 48, 64, 128, 256]
    files = []
    for s in sizes:
        f = pathlib.Path(tmp) / f"{s}.png"
        png(icon_svg, f, s)
        files.append(str(f))
    subprocess.run(["magick", *files, str(logo / "gitpoulpi.ico")], check=True)
print("logo assets written for", a.slug)
