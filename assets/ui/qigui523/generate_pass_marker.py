"""Render the textured pass lettering from LeoCard's own UI font."""

from pathlib import Path
import subprocess

from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.ttLib import TTFont


ROOT = Path(__file__).resolve().parent
FONT = ROOT.parents[1] / "fonts/ChillRoundGothic-Medium.ttf"


def glyph_path(font: TTFont, char: str) -> str:
    glyph_set = font.getGlyphSet()
    glyph = glyph_set[font.getBestCmap()[ord(char)]]
    pen = SVGPathPen(glyph_set)
    glyph.draw(pen)
    return pen.getCommands()


def main() -> None:
    font = TTFont(FONT)
    first, second = (glyph_path(font, char) for char in "不出")
    source = ROOT / "pass-marker.svg"
    source.write_text(
        f'''<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="112" height="48" viewBox="0 0 112 48">
  <defs>
    <g id="letters" transform="translate(19 37) scale(.036 -.036)">
      <path d="{first}"/>
      <path d="{second}" transform="translate(1000 0)"/>
    </g>
    <linearGradient id="ink" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#fffdfd"/>
      <stop offset=".52" stop-color="#e8e2f2"/>
      <stop offset="1" stop-color="#b6a7d0"/>
    </linearGradient>
    <pattern id="grain" width="4" height="4" patternUnits="userSpaceOnUse">
      <path d="M0 4 4 0" fill="none" stroke="#fff" stroke-width=".35" opacity=".34"/>
    </pattern>
    <clipPath id="letter-shape"><use xlink:href="#letters"/></clipPath>
  </defs>
  <use xlink:href="#letters" transform="translate(1 1.5)" fill="#0b0a12" stroke="#0b0a12" stroke-width="55" stroke-linejoin="round" opacity=".68"/>
  <use xlink:href="#letters" fill="#66597c" stroke="#66597c" stroke-width="33" stroke-linejoin="round"/>
  <use xlink:href="#letters" fill="url(#ink)"/>
  <rect width="112" height="48" fill="url(#grain)" clip-path="url(#letter-shape)"/>
</svg>''',
        encoding="utf-8",
    )
    subprocess.run(
        ["rsvg-convert", "-w", "336", "-h", "144", "-o", str(ROOT / "pass-marker.png"), str(source)],
        check=True,
    )


if __name__ == "__main__":
    main()
