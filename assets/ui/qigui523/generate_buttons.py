"""Generate the in-game QiGui523 action button textures from editable SVG."""

from pathlib import Path
import subprocess


ROOT = Path(__file__).resolve().parent
PALETTES = {
    "play": (("#2e2c3c", "#1b1a27", "#a99ad7"), ("#423d58", "#29263b", "#d2c5ff")),
    "pass": (("#303534", "#1c2221", "#aab6b2"), ("#454d4a", "#2b3431", "#e1ebe6")),
    "hint": (("#263b36", "#172824", "#88b9a9"), ("#35544a", "#20392f", "#b4ecd4")),
    "warning": (("#443725", "#2a2118", "#c9a66d"), ("#604a2c", "#392a1b", "#f1c98a")),
}


def svg(top: str, bottom: str, accent: str, hovered: bool) -> str:
    edge_opacity = "0.94" if hovered else "0.68"
    sheen_opacity = "0.15" if hovered else "0.08"
    glow = (
        f'<path d="M9 2H155L162 9V45L155 52H9L2 45V9Z" '
        f'fill="none" stroke="{accent}" stroke-width="2" opacity="0.40"/>'
        if hovered
        else ""
    )
    return f'''<svg xmlns="http://www.w3.org/2000/svg" width="164" height="54" viewBox="0 0 164 54">
<defs>
  <linearGradient id="face" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0" stop-color="{top}"/>
    <stop offset="1" stop-color="{bottom}"/>
  </linearGradient>
  <pattern id="grain" width="7" height="7" patternUnits="userSpaceOnUse">
    <path d="M0 6L6 0" fill="none" stroke="#ffffff" stroke-width="0.4" opacity="0.075"/>
  </pattern>
  <clipPath id="shape"><path d="M9 2H155L162 9V45L155 52H9L2 45V9Z"/></clipPath>
</defs>
<path d="M9 4H155L162 11V47L155 54H9L2 47V11Z" fill="#000000" opacity="0.58"/>
<path d="M9 2H155L162 9V45L155 52H9L2 45V9Z" fill="url(#face)"/>
<path d="M9 2H155L162 9V45L155 52H9L2 45V9Z" fill="url(#grain)" clip-path="url(#shape)"/>
<path d="M10 5H154L159 10V13H5V10Z" fill="#ffffff" opacity="{sheen_opacity}"/>
<path d="M7 43L11 47H153L157 43" fill="none" stroke="#000000" stroke-width="1.3" opacity="0.43"/>
<path d="M10 5H154L159 10V44L154 49H10L5 44V10Z" fill="none" stroke="{accent}" stroke-width="1.3" opacity="{edge_opacity}"/>
{glow}
<path d="M9 2H155L162 9V45L155 52H9L2 45V9Z" fill="none" stroke="#080c0b" stroke-width="1.4"/>
</svg>'''


def main() -> None:
    for name, (normal, hovered) in PALETTES.items():
        for state, palette in (("normal", normal), ("hover", hovered)):
            stem = f"{name}-{state}"
            source = ROOT / f"{stem}.svg"
            target = ROOT / f"{stem}.png"
            source.write_text(svg(*palette, hovered=state == "hover"), encoding="utf-8")
            subprocess.run(
                ["rsvg-convert", "-w", "492", "-h", "162", "-o", str(target), str(source)],
                check=True,
            )


if __name__ == "__main__":
    main()
