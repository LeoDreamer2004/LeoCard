# Minecraft 新版本素材更新

目录：`icons/` 存放分类图集，`widgets/` 存放界面纹理，`background/` 存放密铺和全景。

- **词库**：更新 `assets/data/minecraft/` 对应 CSV；核对 Java 正式版新增内容、属性和两段加入版本号。
- **图标**：方块与物品共用 `icons/entries.png`；其余六类各自一张。同步 `assets/ui/minecraft_guess/icons/` 中的分类索引与 `layouts.tsv`，各图集可独立调整分辨率。
- **背景**：更新 `background/block-fronts.png`、`block-fronts.tsv`，保留排除和正面选取规则；新增全景后补齐 `versions.tsv`。
- **音效与来源**：需要时更新 `assets/audio/minecraft_guess/note_block/`、`tools/minecraft/sources/background.json`（不提交） 与 `assets/ASSETS.md`。

景观中的“任意群系”限定在其生成维度的自然群系内；虚空需显式标注“（含虚空）”。

生成工具、采集来源和校验记录放在忽略提交的 `tools/minecraft/`，原图只缓存在 `target/minecraft-audit/`。
