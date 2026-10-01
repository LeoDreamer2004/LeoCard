"""Render the original LeoCard achievement emblems from compact SVG sources."""

from pathlib import Path
import subprocess


ROOT = Path(__file__).resolve().parent


def render(name: str, svg: str, width: int = 512, height: int = 512) -> None:
    source = ROOT / f"{name}.svg"
    source.write_text(svg, encoding="utf-8")
    subprocess.run(
        ["rsvg-convert", "-w", str(width), "-h", str(height), "-o", str(ROOT / f"{name}.png"), str(source)],
        check=True,
    )


FRAME = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256">
<defs>
 <linearGradient id="rim" x2="1" y2="1"><stop stop-color="#fff0bb"/><stop offset=".38" stop-color="#d4a968"/><stop offset=".72" stop-color="#805a43"/><stop offset="1" stop-color="#f5dba0"/></linearGradient>
 <linearGradient id="body" x2="0" y2="1"><stop stop-color="#403c62"/><stop offset=".56" stop-color="#252842"/><stop offset="1" stop-color="#111a2f"/></linearGradient>
 <linearGradient id="face" x2="0" y2="1"><stop stop-color="#ffeac0"/><stop offset="1" stop-color="#bd9862"/></linearGradient>
 <filter id="shadow" x="-.3" y="-.3" width="1.6" height="1.6"><feGaussianBlur stdDeviation="5"/></filter>
</defs>
<path d="M128 12 244 128 128 244 12 128Z" fill="#090c19" opacity=".65" filter="url(#shadow)"/>
<path d="M128 7 249 128 128 249 7 128Z" fill="url(#rim)"/>
<path d="M128 16 240 128 128 240 16 128Z" fill="url(#body)" stroke="#111529" stroke-width="3"/>
<path d="M128 27 229 128 128 229 27 128Z" fill="none" stroke="#b6996d" stroke-opacity=".6" stroke-width="2"/>
<path d="M128 37 219 128 128 219 37 128Z" fill="none" stroke="#837ba6" stroke-opacity=".42"/>
<path d="M44 128h20m128 0h20M128 44v20m0 128v20" stroke="#edc988" stroke-width="3" stroke-linecap="round"/>
<circle cx="128" cy="128" r="67" fill="#151d36" stroke="#756b7e" stroke-width="2"/>
{symbol}
</svg>"""

SYMBOLS = {
    "mahjong": """
<g transform="rotate(-14 99 124)">
 <rect x="75" y="91" width="48" height="71" rx="6" fill="#47786d" stroke="#283d48" stroke-width="2"/>
 <rect x="75" y="86" width="48" height="71" rx="6" fill="url(#face)" stroke="#735b54" stroke-width="2"/>
 <g fill="none" stroke="#397c69" stroke-width="4" stroke-linecap="round">
  <path d="M89 107v12m20-12v12M89 132v12m20-12v12"/>
  <path d="M85 107h8m12 0h8m-28 12h8m12 0h8M85 132h8m12 0h8m-28 12h8m12 0h8" stroke-width="2"/>
 </g>
</g>
<g transform="rotate(14 157 124)">
 <rect x="133" y="91" width="48" height="71" rx="6" fill="#47786d" stroke="#283d48" stroke-width="2"/>
 <rect x="133" y="86" width="48" height="71" rx="6" fill="url(#face)" stroke="#735b54" stroke-width="2"/>
 <g fill="none" stroke="#397c69" stroke-width="3">
  <circle cx="148" cy="108" r="5"/><circle cx="166" cy="108" r="5"/>
  <circle cx="148" cy="134" r="5"/><circle cx="166" cy="134" r="5"/>
 </g>
</g>
<rect x="103" y="82" width="50" height="81" rx="6" fill="#47786d" stroke="#283d48" stroke-width="2"/>
<rect x="103" y="76" width="50" height="81" rx="6" fill="url(#face)" stroke="#735b54" stroke-width="2"/>
<path d="M113 103h30v25h-30ZM128 90v53" fill="none" stroke="#a6444b" stroke-width="6" stroke-linejoin="round"/>
<path d="M91 172c23 10 51 10 74 0" fill="none" stroke="#d5ae73" stroke-width="5" stroke-linecap="round"/>
""",
    "qigui": """
<defs><linearGradient id="seven" x2="0" y2="1"><stop stop-color="#d77571"/><stop offset="1" stop-color="#a2444c"/></linearGradient></defs>
<path d="M94 83h73v15l-45 77h-23l46-73H94Z" fill="url(#seven)" stroke="#e3bc89" stroke-width="3" stroke-linejoin="round"/>
<path d="M98 88h63" stroke="#efae96" stroke-opacity=".65" stroke-width="2"/>
""",
    "texas": """
<circle cx="128" cy="125" r="56" fill="#d7b579" stroke="#5f5063" stroke-width="3"/>
<circle cx="128" cy="125" r="45" fill="#273652" stroke="#f2d9a3" stroke-width="5" stroke-dasharray="8 7"/>
<path d="M128 82c-10 15-40 33-40 54 0 13 9 23 22 23 9 0 15-6 18-12 3 6 9 12 18 12 13 0 22-10 22-23 0-21-30-39-40-54Z" fill="url(#face)"/>
<path d="M123 147c0 14-3 21-10 27h30c-7-6-10-13-10-27Z" fill="url(#face)"/>
<path d="M108 155c6 0 11-4 14-9m26 9c-6 0-11-4-14-9" fill="none" stroke="#755a5b" stroke-width="2"/>
""",
    "shengji": """
<path d="M88 90h45v50H88Z" fill="url(#face)" stroke="#786058" stroke-width="3" stroke-linejoin="round"/>
<path d="M95 98h28v27H95Z" fill="#395568" stroke="#a68d6a" stroke-width="2"/>
<path d="m99 100 20 22" stroke="#9eb4ba" stroke-width="3" stroke-opacity=".6"/>
<path d="M83 90h54l-6-9H88Z" fill="#e6c792" stroke="#786058" stroke-width="3" stroke-linejoin="round"/>
<path d="M128 118h41c9 0 14 6 14 15v16h-59Z" fill="url(#face)" stroke="#786058" stroke-width="3" stroke-linejoin="round"/>
<path d="M156 115V98h7" fill="none" stroke="#d1ac78" stroke-width="6" stroke-linecap="round"/>
<path d="M168 127v12m6-11v10" stroke="#6e626a" stroke-width="3" stroke-linecap="round"/>
<path d="M82 143h102m-48 3v11h11" fill="none" stroke="#d1ac78" stroke-width="5" stroke-linejoin="round"/>
<path d="M76 143c0-15 10-26 24-26s25 11 25 26" fill="none" stroke="#ebce97" stroke-width="7" stroke-linecap="round"/>
<g fill="#18233a" stroke="#e6c792" stroke-width="4">
 <circle cx="100" cy="150" r="22"/>
 <circle cx="166" cy="156" r="15"/>
</g>
<g fill="#b48f62" stroke="#f0d8a7" stroke-width="2">
 <circle cx="100" cy="150" r="10"/>
 <circle cx="166" cy="156" r="6"/>
</g>
<path d="M100 135v5m0 20v5m-15-15h5m20 0h5" stroke="#b48f62" stroke-width="3" stroke-linecap="round"/>
""",
    "uno": """
<g transform="rotate(-17 128 128)">
 <rect x="92" y="72" width="72" height="108" rx="12" fill="#d7b989" stroke="#805f63" stroke-width="3"/>
 <rect x="99" y="80" width="58" height="92" rx="8" fill="#28314d"/>
 <path d="M128 84c22 13 28 35 16 46-11 10-26 2-29 16-2 8 5 16 13 22-24-5-37-20-29-36 5-10 19-10 22-19 3-10-5-19-10-24 5-2 11-4 17-5Z" fill="#d55854"/>
 <path d="M130 87c-12 18-7 28 6 37 13 10 14 22-4 42 26-12 34-35 20-50-8-9-17-17-22-29Z" fill="#e8bd59"/>
</g>
""",
    "personal": """
<path d="M79 173v-12c0-25 18-41 49-41s49 16 49 41v12Z" fill="url(#face)" stroke="#725959" stroke-width="3" stroke-linejoin="round"/>
<path d="m107 128 21 25 21-25m-21 25v20" fill="none" stroke="#66546f" stroke-width="4" stroke-linejoin="round"/>
<path d="M91 159v14m74-14v14" stroke="#ac8c69" stroke-width="3" stroke-linecap="round"/>
<path d="M114 117v10c7 10 21 10 28 0v-10" fill="url(#face)" stroke="#725959" stroke-width="3"/>
<ellipse cx="128" cy="97" rx="24" ry="28" fill="url(#face)" stroke="#725959" stroke-width="3"/>
<path d="M105 94c-1-16 9-27 23-27s25 11 23 27c-8-1-14-5-18-11-7 7-17 10-28 11Z" fill="#63536e" stroke="#b49a7c" stroke-width="2" stroke-linejoin="round"/>
<g fill="#83676b"><circle cx="117" cy="100" r="2.2"/><circle cx="139" cy="100" r="2.2"/></g>
<path d="M125 101v10h6" fill="none" stroke="#ac8c69" stroke-width="2" stroke-linecap="round"/>
<path d="M88 178h80" stroke="#d3ad76" stroke-width="4" stroke-linecap="round"/>
""",
}

for name, symbol in SYMBOLS.items():
    render(f"emblem-{name}", FRAME.format(symbol=symbol))

render(
    "scroll-chevron",
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40 24">'
    '<path d="m5 19 15-13 15 13" fill="none" stroke="#e8c58d" '
    'stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/></svg>',
    80,
    48,
)


MEDAL = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256">
<defs>
 <linearGradient id="metal" x2="1" y2="1"><stop stop-color="{light}"/><stop offset=".46" stop-color="{base}"/><stop offset="1" stop-color="{dark}"/></linearGradient>
 <linearGradient id="inset" x2="0" y2="1"><stop stop-color="{dark}"/><stop offset="1" stop-color="{base}"/></linearGradient>
</defs>
<path d="M128 12 220 54 241 151 175 232H81L15 151 36 54Z" fill="#0c1020" opacity=".5"/>
<path d="M128 14 216 56 237 150 172 227H84L19 150 40 56Z" fill="url(#metal)" stroke="{light}" stroke-width="5"/>
<path d="M128 32 201 67 219 145 164 209H92L37 145 55 67Z" fill="url(#inset)" stroke="{dark}" stroke-width="5"/>
<path d="M128 47 189 76 203 140 157 194H99L53 140 67 76Z" fill="none" stroke="{light}" stroke-opacity=".7" stroke-width="3"/>
<g fill="none" stroke="{light}" stroke-width="6" stroke-linecap="round">
 <path d="M91 99H77v15c0 15 11 24 24 24m64-39h14v15c0 15-11 24-24 24"/>
</g>
<path d="M91 87h74v26c0 26-13 43-37 43s-37-17-37-43Z" fill="{light}" stroke="{dark}" stroke-width="3"/>
<path d="M120 153h16v22h-16Z" fill="{light}" stroke="{dark}" stroke-width="3"/>
<path d="M107 175h42l7 12h-56Z" fill="{light}" stroke="{dark}" stroke-width="3" stroke-linejoin="round"/>
<path d="m128 101 5 10 11 2-8 8 2 12-10-5-10 5 2-12-8-8 11-2Z" fill="{base}" stroke="{dark}" stroke-width="2"/>
</svg>"""

for name, light, base, dark in [
    ("gold", "#ffe9a4", "#d6a652", "#705025"),
    ("silver", "#f1f4ff", "#abb7cf", "#5c647a"),
    ("bronze", "#ffd0a5", "#bb795b", "#70443e"),
]:
    render(f"medal-{name}", MEDAL.format(light=light, base=base, dark=dark))
