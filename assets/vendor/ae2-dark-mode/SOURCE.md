# AE2 Dark Mode GUI textures

- Author: Ridanisaurus.
- Based on Applied Energistics 2 GUI assets by Ridanisaurus Rid and AlgorithmX2 et al.
- Resource pack: AE2DarkMode V1.0.6, Minecraft 1.21.
- Author's release page: https://www.curseforge.com/minecraft/texture-packs/ae2-dark-mode/files/5520011
- Download: https://edge.forgecdn.net/files/5520/011/AE2DarkMode-V1.0.6.zip
- License: CC BY-NC-SA 4.0, https://creativecommons.org/licenses/by-nc-sa/4.0/
- License copy: `LICENSE.txt`.

Runtime files in `../../ui/minecraft_guess/widgets/`:

- `table-frame.png`: `assets/ae2/textures/guis/background.png`, with the two dark bottom-shadow rows (253–254) removed while retaining the outermost border row (255), giving a symmetric two-pixel frame.
- `popup-frame.png`: the same symmetric `table-frame.png` reduced to a 5×5 nine-slice sprite by retaining its two-pixel corner borders and one center pixel. This prevents Bevy from shrinking the border with the original 256×254 source size when displaying a compact reveal popup.
- `table-inset.png`: the recessed panel from `assets/ae2/textures/guis/cpu_selector.png`, keeping the top three rows `(7, 18, 69, 3)`, a plain middle row `(7, 22, 69, 1)` and the bottom border `(7, 156, 69, 1)`; these are joined vertically into a 69×5 nine-slice sprite so the original baked row separators do not stretch inside the table.
- `table-surface.png`: uncolored recessed fill cropped from `cpu_selector.png`, rectangle `(8, 22, 8, 8)`, for headings and empty rows.
- `scroll-track.png`: recessed scrollbar track cropped from `cpu_selector.png`, rectangle `(80, 18, 6, 139)`.
- `table-cell.png`: background fill cropped from `background.png`, rectangle `(8, 8, 8, 8)`; tinted at runtime for guess feedback.
- `button.png` and `button-disabled.png`: unchanged `assets/ae2/textures/gui/sprites/button.png` and `button_disabled.png`, using the upstream three-pixel nine-slice border.
- `input.png` and `input-focused.png`: cropped from `assets/ae2/textures/guis/text_field.png`, rectangles `(0, 0, 128, 12)` and `(0, 12, 128, 12)`.
- `scroll-thumb.png` and `scroll-thumb-disabled.png`: unchanged `assets/ae2/textures/gui/sprites/big_scroller.png` and `big_scroller_disabled.png`.
- `close.png`: close icon cropped from `assets/ae2/textures/guis/states.png`, rectangle `(96, 0, 16, 16)`; displayed on the existing OreUI button textures.

The crops retain the upstream pixels. These files remain under CC BY-NC-SA 4.0.

Only the runtime files, license and this source record are retained. The original sprites,
unused network-status texture and Minecraft resource-pack metadata have been removed.

OreUI assets use nearest-neighbor sampling to preserve pixel edges and avoid blending white RGB from transparent pixels.
