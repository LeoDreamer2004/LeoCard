# 4 Colour Cards (CC0)

- 作者：VerzatileDev
- 素材页：<https://verzatiledev.itch.io/4colour>
- CC0 声明：<https://verzatiledev.itch.io/4colour/devlog/1447489/now-cc0-license>
- 许可证：Creative Commons Zero 1.0 Universal（CC0 1.0）
- 许可证全文：<https://creativecommons.org/publicdomain/zero/1.0/legalcode>
- 下载日期：2026-08-14
- 原始包：`4ColourCardsByVerzatileDev.zip`
- 原始包 SHA-256：`ee20da4f717ef44be9ee6f9c48561f169aebb672222d39aea2c254603be5d337`

作者的下载页将素材包标记为 CC0，后续公告也明确说明其已进入公共领域。原始压缩包
不含独立的许可证文本，因此在此保留作者的许可声明链接和 CC0 法律文本链接。

## 运行时加工

`assets/cards/uno` 中的 64 张牌面从压缩包的 `Individual` 目录选取。加工仅包括：

- 选取经典四种颜色的数字牌与摸二、反转、跳过牌；
- 选取万能牌、万能摸四牌及其四种选色状态；
- 选取空白万能牌和一张牌背；
- 从 965×1507、16 位 PNG 缩放为 256×400、8 位 sRGB PNG；
- 去除非图像元数据并重新进行无损 PNG 压缩。

一副牌中重复出现的实体牌应复用同一张纹理，不需要复制图片文件。
