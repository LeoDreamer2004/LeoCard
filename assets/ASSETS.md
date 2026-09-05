# 运行时素材使用与许可声明

本文声明 LeoCard 当前发布物实际使用的图像、音频、字体和着色器素材及其许可来源。
发布构建会按 `crates/client/runtime-assets.txt` 选取运行时文件，并默认内嵌到可执行文件；
未被该清单选中的原始压缩包、预览图和备用素材不属于运行时发布内容。需要保留的第三方
原始文件位于 `assets/vendor/`；仅保留加工成品的例外由对应条目记录来源和许可。项目加工
后的素材位于 `assets/cards/`、`assets/ui/`、`assets/audio/`、`assets/icons/` 与
`assets/fonts/`。

除另有说明外，项目自有的运行时素材随 LeoCard 的 GPL-3.0 许可证提供。用户在游戏中
自行选择的桌布仅在本地读取，不随 LeoCard 发布物分发，相关使用权由用户自行确认。

## CC0 素材

下列素材以 CC0 1.0 或等同公共领域声明提供，可随程序使用、修改和再分发，无署名
义务；本项目仍保留来源以便追溯。

| 来源 | 运行时用途 | 本地位置 | 许可证与来源 |
| --- | --- | --- | --- |
| Kenney Boardgame Pack 2 | 普通牌面、牌背与德州扑克筹码 | `vendor/kenney/boardgame/` | CC0 1.0；<https://kenney.nl/assets/boardgame-pack> |
| Kenney UI Pack 2.0 | 按钮底图、方向箭头与部分操作音效 | `vendor/kenney/ui/` | CC0 1.0；<https://kenney.nl/assets/ui-pack> |
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
继续遵循 M+ 字体许可证。高度图
由刻纹透明度及笔画内部距离计算生成，供 `shaders/mahjong_tile.wgsl` 实时绘制凹刻深度、
象牙牌体、绿色侧边和表面反光，不包含新的第三方图形。

### Game-icons 射箭图标（CC BY 3.0）

- 作者：Lorc（`Archery target`）、Delapouite（`Dart`）
- 运行时文件：`ui/effects/shengji_target.png`、`shengji_dart.png`
- 原始文件与完整署名：`vendor/game-icons/archery/SOURCE.md`
- 来源：<https://game-icons.net/>
- 许可证：<https://creativecommons.org/licenses/by/3.0/>

两张运行时 PNG 由原 SVG 栅格化后按游戏主题着色，用于升级的目标与飞镖效果。再分发时
须保留上述作者、来源和 CC BY 3.0 署名信息。

### GitHub Mark（MIT；受商标规范约束）

- 运行时文件：`icons/github-mark.svg`、`icons/github-mark.png`
- 来源：GitHub Primer Octicons 的 `mark-github`，<https://github.com/primer/octicons>
- 许可证：MIT
- 用途：游戏设置中打开 LeoCard GitHub 仓库的按钮

GitHub 名称与标志同时受 GitHub 商标规范约束；本项目仅将其用于指向对应仓库的识别性
链接。

### 无名杀互动与快捷语音（GPL-3.0）

- 来源：<https://github.com/libnoname/noname>
- 运行时目录：`vendor/noname/interactions/`、`vendor/noname/audio/effect/`、
  `vendor/noname/voice/male/`、`vendor/noname/damage_fire2.mp3`
- 上游目录：`apps/core/image/emotion/throw_emotion`、`apps/core/audio/effect`、
  `apps/core/audio/voice/male`
- 许可证：GPL-3.0

互动素材提供鲜花、鸡蛋、酒杯、拖鞋的飞行/命中图片与音效，用于桌内互动和资料页的
鲜花、鸡蛋累计展示；酒杯按 10 朵鲜花、拖鞋按 10 个鸡蛋计入目标玩家资料。快捷语音
为编号 0 至 22 的男性语音。`interactions/NOTICE.md` 与 `voice/NOTICE.md` 保留了更具体
的素材范围和来源说明。

这些文件随 GPL-3.0 项目分发。分发包含它们的二进制或素材包时，必须同时满足项目及
上游 GPL-3.0 的许可证保留和对应源代码提供义务。

## 项目内素材

以下文件由项目维护，用于界面与效果：

- `icons/app-icon.png`：由用户提供并整理为透明方形画布的 LeoCard 应用图标；运行时
  直接编译进客户端，Windows 发布物另使用由它生成的多尺寸 ICO 文件。
- `ui/panel_*.png`、`ui/player_panel_*.png`：窗口、分区和玩家框底图。
- `shaders/table_background.wgsl`、`shaders/turn_border.wgsl`、`shaders/uno_palette.wgsl`：
  桌布、回合边框与 UNO 调色盘着色器。
- `ui/effects/sequence_airplane.svg` 与其 PNG：七鬼五二三顺子效果；该图为项目原创，
  说明见 `ui/effects/README.md`。
- `icons/host-crown.*`、`icons/list-menu.*`、`icons/robot-2-fill.*`：房主、快捷语音和
  机器人界面图标；SVG 为可编辑源文件，PNG 为运行时副本。
- `cards/uno-extension/uno-flip/`：UNO FLIP 亮暗双面牌纹理。
- `cards/uno-extension-source/uno-flip/dark-bases/`：以默认 UNO 牌框为母版制作的暗面底图；暗面
  外框使用纯黑描边并沿用原牌框透明圆角。
- `cards/uno-extension-source/uno-flip/flip-glyph.png`：项目绘制的翻牌透明纹理，亮暗两面共享同一
  几何结构并分别着色，不复用转向牌图案。
- `cards/uno-extension-source/uno-flip/skip-everyone-glyph.png`：项目绘制的全员禁手双禁止透明纹理。
- `cards/uno-extension-source/uno-flip/plus-one-glyph.png`、`plus-two-glyph.png`、
  `plus-five-glyph.png`：整串绘制的摸牌
  角标透明纹理，合成时不再分别拼接加号和数字。
- `cards/uno-extension-source/uno-flip/dark-glyphs/wild-draw-color.png`：暗面指定颜色摸牌的透明
  合成层。
- `mahjong/demo/height/`：由香港麻将牌纹理内部距离场生成的刻印高度图；粗笔画中心更深，
  用于计算连续凹槽法线、内阴影与反射高光。

亮面摸一和 Wild 摸二、暗面摸五均直接以默认牌框及对应摸牌牌型的比例合成，不保留
容易产生色边的中间抠图。

上述项目内文件按 LeoCard 的 GPL-3.0 许可证提供。

## 可追溯性记录

Kenney 原始包均保留各自随包的 `License.txt` 或 `license.txt`。已记录的下载包
SHA-256 如下：

| 素材包 | SHA-256 |
| --- | --- |
| Kenney Boardgame Pack 2 | `3b6a7dd5af658d1ffa0071429d5dde10ff63121e88bd06d4904e721e60b2c398` |
| Kenney UI Pack 2.0 | `a8a14a234911eb648c062622915c93e79e94e97cb7f9f375a70f6617f1174318` |
| Kenney Interface Sounds 1.0 | `f2193d072726d6758a5f7871b2dcc54dcce0d5c35c6f0a62f92549b327c81232` |
| Kenney Casino Audio 1.1 | `f36250766ac5bc378c13708ddf12a23a8e54a3251f8d482c7536e51b5dbafa18` |
| VerzatileDev 4 Colour Cards | `ee20da4f717ef44be9ee6f9c48561f169aebb672222d39aea2c254603be5d337` |

本声明对应仓库中当前的运行时清单与随附来源记录。
