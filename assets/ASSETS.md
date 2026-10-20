# 运行时素材使用与许可声明

本文声明 LeoCard 当前发布物实际使用的图像、音频、字体素材及其许可来源。
发布构建会按 `crates/client/runtime-assets.txt` 选取运行时文件，并默认内嵌到可执行文件；
未被该清单选中的原始压缩包、预览图和备用素材不属于运行时发布内容。需要保留的第三方
原始文件位于 `assets/vendor/`；仅保留加工成品的例外由对应条目记录来源和许可。

除另有说明外，项目自有的运行时素材随 LeoCard 的 GPL-3.0 许可证提供。用户在游戏中
自行选择的桌布仅在本地读取，不随 LeoCard 发布物分发，相关使用权由用户自行确认。

## CC0 素材

下列素材以 CC0 1.0 或等同公共领域声明提供，可随程序使用、修改和再分发，无署名
义务；本项目仍保留来源以便追溯。

| 来源 | 运行时用途 | 本地位置 | 许可证与来源 |
| --- | --- | --- | --- |
| Kenney Boardgame Pack 2 | 普通牌面、牌背与德州扑克筹码 | `vendor/kenney/boardgame/` | CC0 1.0；<https://kenney.nl/assets/boardgame-pack> |
| Kenney UI Pack 2.0 | 部分操作音效 | `vendor/kenney/ui/Sounds/` | CC0 1.0；<https://kenney.nl/assets/ui-pack> |
| Kenney Interface Sounds 1.0 | 通用界面提示和德州、UNO、升级操作音效 | `vendor/kenney/interface-sounds/` | CC0 1.0；<https://kenney.nl/assets/interface-sounds> |
| Kenney Casino Audio 1.1 | 发牌、出牌、推牌、筹码和结算音效 | `vendor/kenney/casino-audio/` | CC0 1.0；<https://kenney.nl/assets/casino-audio> |
| VerzatileDev 4 Colour Cards | UNO 牌面、选色状态和 UNO 牌背 | `cards/uno/` | CC0 1.0；<https://verzatiledev.itch.io/4colour>，许可声明见 <https://verzatiledev.itch.io/4colour/devlog/1447489/now-cc0-license> |
| n4 / OpenGameArt | 内置深绿色桌布纹理 | `vendor/opengameart/green-textile/table_felt_dark_green.png` | CC0；<https://opengameart.org/content/seamless-pattern-pack-greentextilepng> |
| BigSoundBank / Joseph SARDIN | 升级游戏的断电、通电音效 | `audio/shengji/power-off.ogg`、`power-on.ogg` | CC0 / public-domain equivalent；<https://bigsoundbank.com/electric-switch-s0026.html> |

`table_felt_dark_green.png` 是 OpenGameArt 原纹理的项目内衍生版本，降低了亮度和饱和度；
其来源与加工说明见同目录的 `SOURCE.md`。`power-off.ogg` 与 `power-on.ogg` 由
Electric switch #0026 的瞬态片段裁剪并做响度规范化而成；原始文件校验值与加工记录见
`vendor/bigsoundbank/electric-switch/SOURCE.md`。

UNO 运行时牌面由 64 张 PNG 组成；实体牌的重复副本共用相同纹理。原始包、作者、下载
日期和 SHA-256 记录在 `vendor/verzatiledev/4-colour-cards/SOURCE.md`。

## 需保留署名或许可证的素材

### Qwen3-TTS 生成语音

生成模型：[Qwen3-TTS VoiceDesign](https://huggingface.co/Qwen/Qwen3-TTS-12Hz-1.7B-VoiceDesign) 与 [Qwen3-TTS Base](https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-Base)，模型卡标注 Apache-2.0。参考音保存在 `audio/reference`，不进入运行时资源包。

- 国标麻将番种报读语音：`audio/mahjong/fans/` 下的男女声各 81 条语音。
- 麻将操作语音：`audio/mahjong/actions/` 下的男女声吃、碰、明杠、暗杠、补杠、胡、自摸和补花语音。
- 德州扑克行动语音：`audio/texas_holdem/voices/` 下的 14 条男女声行动语音。

### Minecraft 等级物品图标（Mojang）

- 运行时文件：`ui/profile/` 下的 10 张 PNG，分别用于下界合金锭、钻石、金锭、红石粉、铁锭、铜锭、圆石、橡木原木、泥土和堆肥桶等级。
- 来源：[Minecraft Wiki 的 Invicon 图标](https://minecraft.wiki/)。
- Minecraft 物品图像的权利归 Mojang Studios 所有，不适用 LeoCard 的 GPL-3.0 素材授权；使用须遵守 [Minecraft 使用规范](https://www.minecraft.net/en-us/usage-guidelines)。

### MC 竞猜方块与物品图标（Mojang）

- 运行时文件：`ui/minecraft_guess/icons/entries.png`，方块和物品共用，用于猜测记录、候选词以及揭晓后的答案。
- 来源：[Minecraft Wiki 库存图标](https://minecraft.wiki/w/Category:InvSprite_files)和对应的方块原始渲染图。按当前词库的方块／物品标识分别映射，外观相同的图标共用图集位置；动画取第一帧，保持长宽比装入 64×64 像素图格。物品保留原生像素图，三维方块采用较高分辨率的原始渲染图，客户端使用最近邻采样。
- `ui/minecraft_guess/icons/entries.tsv` 是客户端使用的图集索引，`layouts.tsv` 记录各图集的图格尺寸和列数。
- 关系谜图类别图集 `ui/minecraft_guess/graph/node-symbols.png` 使用 128×128 像素图格，复用草方块、金锭、僵尸正面头像、附魔书、药水、绿宝石和海底神殿图标；草方块采用 300×300 原始渲染图缩制，其余保留原生像素。绿色山脉剪影使用原版草方块顶部纹理。
- Minecraft 图像的权利归 Mojang Studios 所有，授权要求与上述等级物品图标一致。

### MC 竞猜生物、附魔、状态效果、职业、世界景观、生物群系

- 独立图集：`ui/minecraft_guess/icons/{mobs,enchantments,effects,villager-professions,landscapes,biomes}.png`，各自使用相同名称的索引，位于 `ui/minecraft_guess/icons/`。原版生物头像来自 [Minecraft Wiki 生物列表](https://minecraft.wiki/w/Mob)，职业图像来自 [村民职业外观](https://minecraft.wiki/w/Villager#Professions)，地物图标来自 [Minecraft Wiki 地物](https://zh.minecraft.wiki/w/地物)，生成结构图标来自 [Minecraft Wiki 生成结构](https://zh.minecraft.wiki/w/生成结构)，生物群系图标来自 [Minecraft Wiki 生物群系](https://zh.minecraft.wiki/w/生物群系)。状态效果纹理由 Mojang 官方 Java 26.3 客户端提取，附魔图标复用附魔书原图。
- 对应 CSV 属性依据官方注册表、[Minecraft Wiki](https://minecraft.wiki/)及[中文状态效果查论编](https://zh.minecraft.wiki/w/Template:Navbox_effects)整理；Wiki 文本采用 [CC BY-NC-SA 3.0](https://creativecommons.org/licenses/by-nc-sa/3.0/)。
- Minecraft 图像权利归 Mojang Studios 所有，使用规范同上。

### MC 竞猜游戏背景（Mojang）

- 本地位置：`ui/minecraft_guess/background/`。每小局从词库完整方块的正面中随机选一种密铺；关闭客户端随机背景时固定使用 Java 版 1.12.2 的原版泥土菜单背景。猜中加入版本后切换为该版本的全景。
- `block-fronts.png` 从 Java 版 26.3 客户端的默认物品模型、正面纹理和默认染色提取，按 `block-fronts.tsv` 打包为 16×16 像素图格；动态纹理取首帧。全景来自 [Mojang 官方版本清单](https://piston-meta.mojang.com/mc/game/version_manifest_v2.json)所列客户端归档与资源索引中的六面图片，保持原始分辨率，按原版立方体贴图顺序排列并垂直翻转后打包。
- 来源与打包文件的 SHA-1 留存在 `tools/minecraft/sources/background.json`（不提交）；`background/versions.tsv` 记录运行时版本映射。

### MC 竞猜音符盒反馈音效（Mojang）

- 运行时文件：`audio/minecraft_guess/note_block/{guitar,harp,pling,bass,bell}.ogg`。
- 来源：Java 版 26.3 官方资源索引中的 `minecraft/sounds/note/`，保留原始单声道 Vorbis 文件。原文件未重新合成或转码；音高、音量和多音节奏由播放层控制，音高采用原版音符盒的 25 档调音规则，整体音量跟随游戏全局音量。

### CozyUI+ 首页纹理（GPL-3.0）

- 作者：零雾〇五 Fogg05；来源：<https://github.com/Fogg05/CozyUI-Plus>
- `cozy-button-purple-plain.png` 和 `cozy-button-arrows.png` 从原版 `button_highlighted.png` 分离而来。
- 两张 `compact` 按钮图保留原图的两端与中段，统一短按钮的圆角和亮边尺寸。
- 两张红色 `compact` 按钮图分别从灰色常态和紫色高亮按钮的颜色层次转换而来；悬停态保留普通高亮贴图的原始白边，仅将彩色部分转为较暗的红色，两张图的透明轮廓一致。
- 两张冷绿色 `compact` 按钮图同样由灰色常态和紫色高亮按钮变色而来；悬停态保留普通高亮贴图的原始白边，两张图的透明轮廓一致。
- `cozy-game-card.png` 与 `cozy-game-card-hover.png` 分别取自 `hud/effect_background.png` 和 `hud/effect_background_ambient.png`，用于设置标签等卡片的常态与悬停态；只清除了全透明区域的白色 RGB，可见像素未改动。
- `cozy-settings-page.png` 取自 `recipe_book/slot_craftable.png`，用于设置窗口右侧内容页；左侧标签复用 `cozy-game-card.png` 与其高亮变种。
- 两张输入框 `compact` 图使用同样的切片方式，保留原版 `text_field.png` 和 `text_field_highlighted.png` 的边框与底色。
- 四张 `rule-*.png` 箭头图保留原版箭头，将与画面不协调的连通灰色背景转为透明像素。
- `cozy-help-question.png` 清除了透明区域残留的白色底色，避免缩放时在问号边缘出现白边；可见像素未改动。
- 许可证副本：`vendor/cozyui-plus/LICENSE.txt`
- 成就通知使用 `ui/achievements/toast-normal.png` 和 `toast-gold.png`：提取自同版本 `assets/minecraft/textures/gui/advancements/widgets.png` 的蓝色与橙色横条，裁掉周围空白，将白色底转为透明；运行时保留边角，按九宫格拉伸。

这些纹理用于首页、顶栏、设置、资料页、游戏内聊天抽屉和警示提示；按钮与滑块轨道按原纹理的九宫格边距绘制，箭头和滑块柄使用原比例动画帧；其余资源包内容未加入发布物。

### Microsoft Fluent Emoji（MIT）

- 运行时文件：`ui/fluent-emoji/` 下的 30 张 3D PNG
- 来源：<https://github.com/microsoft/fluentui-emoji>
- 许可证：MIT

这些 3D PNG 用于聊天表情；客户端通过上浮和淡出动画呈现动态气泡。

### 寒蝉圆黑体（SIL Open Font License 1.1）

- 运行时文件：`fonts/ChillRoundGothic-Medium.ttf`
- 许可证副本：`fonts/OFL-ChillRoundGothic.txt`
- 来源：<https://github.com/Warren2060/ChillRoundGothic>
- 字体文件 SHA-256：`7aaf168681f168b9f550734d9e25058ffe73213db07d7a50152d5446dd9e7094`

程序使用该字体显示中文界面。任何再分发均应同时提供 OFL 许可证；修改字体时还应遵守
OFL 关于保留名称与再分发的条件。

### I.Mahjong-HK 香港麻将牌字形（M+ 字体许可证）

- 加工文件：`cards/mahjong/hong-kong/` 下的 43 张 600×800 透明 PNG
- 运行时高度图：`cards/mahjong/hong-kong-height/` 下的 42 张 600×800 灰度 PNG
- 来源：<https://github.com/SyaoranHinata/I.Mahjong> 的 `I.MahjongHK.otf`
- 许可证：M+ 字体许可证，允许商业或非商业使用、复制、修改与再分发
- 用途：香港样式的 34 张基础牌、8 张花牌和牌背

项目从字体中栅格化牌面，移除字体自带的牌框，仅保留内部刻纹；万子采用黑色数字、红色
“万”字，筒子采用蓝、红分色，索子采用绿、红分色（一索另含黑色鸟纹），风牌使用黑色，
三元牌、花牌和牌背按牌义着色。原字体和下载仓库不随项目保留，来源与许可由本节记录；加工后的 PNG
继续遵循 M+ 字体许可证。

### GitHub Mark（MIT；受商标规范约束）

- 运行时文件：`icons/github-mark.png`
- 来源：GitHub Primer Octicons 的 `mark-github`，<https://github.com/primer/octicons>
- 许可证：MIT
- 用途：游戏设置中打开 LeoCard GitHub 仓库的按钮

GitHub 名称与标志同时受 GitHub 商标规范约束；本项目仅将其用于指向对应仓库的识别性
链接。

### 无名杀互动与快捷语音（GPL-3.0）

- 来源：<https://github.com/libnoname/noname>
- 运行时目录：`vendor/noname/interactions/`、`vendor/noname/audio/effect/`、
  `vendor/noname/voice/{male,female}/`、`vendor/noname/damage_fire2.mp3`
- 上游目录：`apps/core/image/emotion/throw_emotion`、`apps/core/audio/effect`、
  `apps/core/audio/voice/{male,female}`
- 许可证：GPL-3.0

互动素材提供鲜花、鸡蛋、酒杯、拖鞋的飞行/命中图片与音效，用于桌内互动和资料页的
鲜花、鸡蛋累计展示。快捷语音为男女各一套编号 0 至 22 的语音。

这些文件随 GPL-3.0 项目分发。分发包含它们的二进制或素材包时，必须同时满足项目及
上游 GPL-3.0 的许可证保留和对应源代码提供义务。

### 百度贴吧经典表情

- 运行时文件：`ui/tieba-emoji/` 下的 50 张透明 PNG，保留原始 90×90 尺寸。
- 来源：<https://github.com/microlong666/Tieba_mobile_emotions>，从百度贴吧 Android 11.6.8.2 提取的默认表情，取编号 1–50。
- 原始表情归百度及对应权利人所有；来源仓库声明仅供学习交流与个人非营利性使用，未提供开源授权。

### AE2 Dark Mode GUI 贴图（CC BY-NC-SA 4.0）

- 作者：Ridanisaurus；基于 Applied Energistics 2（Ridanisaurus Rid、AlgorithmX2 et al.）的 GUI。
- Minecraft 1.21，AE2DarkMode V1.0.6：[作者发布页](https://www.curseforge.com/minecraft/texture-packs/ae2-dark-mode/files/5520011)。
- 用于 MC竞猜表格的外框、内凹底面、单元格底纹，以及输入框和操作、候选词按钮。按钮直接采用原图；外框裁去底部多出的两行阴影以统一上下厚度，内凹底面及滚动条轨道取自 CPU 选择器，其余仅裁切，反馈颜色在运行时着色。
- 许可副本：`vendor/ae2-dark-mode/LICENSE.txt`；来源和裁切记录：`vendor/ae2-dark-mode/SOURCE.md`；运行时贴图：`ui/minecraft_guess/widgets/`。原图及资源包元数据已移除。
- 图片素材按 [CC BY-NC-SA 4.0](https://creativecommons.org/licenses/by-nc-sa/4.0/) 提供，保留署名，限非商业使用，衍生素材使用相同许可。

### AE2 蓝色按钮高亮贴图（CC BY-NC-SA 3.0）

- 作者：Ridanisaurus Rid、AlgorithmX2 et al.；AE2 19.2.18（Minecraft 1.21.1）。
- 来源：[AE2 官方发布](https://github.com/AppliedEnergistics/Applied-Energistics-2/releases/tag/neoforge/v19.2.18)。
- 原样采用 `button_highlighted.png`，用于 MC竞猜操作及候选词按钮的蓝色悬停高亮；暗色材质包没有覆盖该素材。
- 运行时文件：`ui/minecraft_guess/widgets/button-highlighted.png`；许可副本：`vendor/ae2/LICENSE.txt`；来源、九宫格边距及上游授权说明：`vendor/ae2/SOURCE.md`。原图及资源包元数据已移除。
- 图片素材按 [CC BY-NC-SA 3.0](https://creativecommons.org/licenses/by-nc-sa/3.0/) 提供。
