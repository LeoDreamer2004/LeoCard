# Minecraft 新版本素材更新

- **词库**：更新 `assets/data/minecraft/blocks.csv`、`items.csv`，核对新增内容、属性和加入版本。
- **物品图标**：补齐来源，重新生成 `entries.png` 和 `assets/data/minecraft/{icons.tsv,icon-sources.json}`。
- **密铺背景**：重新生成 `background/block-fronts.png`、`block-fronts.tsv`，保持现有排除、去重和正面选取规则。
- **主菜单全景**：有新背景时更新 `background/panoramas/`；新增版本必须补上 `background/versions.tsv` 映射，沿用旧背景也要映射。
- **音效与来源**：音符盒原音变化时更新 `assets/audio/minecraft_guess/note_block/`；同步 `background/sources.json` 和 `assets/ASSETS.md`。
