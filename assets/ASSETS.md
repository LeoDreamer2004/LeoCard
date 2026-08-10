# 素材清单

所有第三方原始素材均放在 `assets/vendor` 下，运行时挑选、重命名或二次加工后的
文件应放在 `assets/cards`、`assets/ui` 和 `assets/audio`，不要直接修改原始副本。

## Kenney（CC0 1.0）

下载日期：2026-07-31。每个素材包都保留了压缩包自带的 `License.txt` 或
`license.txt`。

| 素材包 | 版本 | 本地目录 | 官方来源 | 下载包 SHA-256 |
| --- | --- | --- | --- | --- |
| Playing Cards Pack | 1.0 | `assets/vendor/kenney/playing-cards` | <https://kenney.nl/assets/playing-cards-pack> | `b93b0313818e9c8f6acc24b008efdec1d8a6360b13afe953beed23881588be3c` |
| Boardgame Pack | 2 | `assets/vendor/kenney/boardgame` | <https://kenney.nl/assets/boardgame-pack> | `3b6a7dd5af658d1ffa0071429d5dde10ff63121e88bd06d4904e721e60b2c398` |
| UI Pack | 2.0 | `assets/vendor/kenney/ui` | <https://kenney.nl/assets/ui-pack> | `a8a14a234911eb648c062622915c93e79e94e97cb7f9f375a70f6617f1174318` |
| Interface Sounds | 1.0 | `assets/vendor/kenney/interface-sounds` | <https://kenney.nl/assets/interface-sounds> | `f2193d072726d6758a5f7871b2dcc54dcce0d5c35c6f0a62f92549b327c81232` |
| Casino Audio | 1.1 | `assets/vendor/kenney/casino-audio` | <https://kenney.nl/assets/casino-audio> | `f36250766ac5bc378c13708ddf12a23a8e54a3251f8d482c7536e51b5dbafa18` |

这些素材使用 Creative Commons Zero 1.0，可以用于个人和商业项目且不强制署名。
项目仍可在致谢页面标注 Kenney，以便玩家找到原作者。

## BigSoundBank 电气开关（CC0）

- 文件：`assets/vendor/bigsoundbank/electric-switch/electric-switch-0026.ogg`
- 名称：Electric switch #0026
- 作者：Joseph SARDIN
- 来源：<https://bigsoundbank.com/electric-switch-s0026.html>
- 下载日期：2026-08-10
- 原始文件 SHA-256：`c97384d1cdcc5834220d54daed0659dd04b8e19068ca12281137e39646ed2253`
- 许可证：Creative Commons Zero / public domain equivalent
- 加工说明：截取开关断开与吸合瞬态，响度规范化后生成
  `assets/audio/shengji/power-off.ogg` 和 `power-on.ogg`。

来源页明确允许编辑并随个人或商业项目重新分发，且不强制署名；项目仍保留作者与
来源信息以便追溯。

## Game-icons 射箭图标（CC BY 3.0）

- `Archery target`：Lorc
- `Dart`：Delapouite
- 来源：<https://game-icons.net/>
- 许可证：<https://creativecommons.org/licenses/by/3.0/>
- 原始文件：`assets/vendor/game-icons/archery/`
- 运行时文件：`assets/ui/effects/shengji_target.png`、`shengji_dart.png`

运行时版本由原 SVG 栅格化为透明 128×128 PNG，并由游戏按主题动态着色；完整来源、
作者、下载日期和修改说明见同目录的 `SOURCE.md`。

## 寒蝉圆黑体（SIL Open Font License 1.1）

- 文件：`assets/fonts/ChillRoundGothic-Medium.ttf`
- 来源：<https://github.com/Warren2060/ChillRoundGothic>
- SHA-256：`7aaf168681f168b9f550734d9e25058ffe73213db07d7a50152d5446dd9e7094`
- 许可证：`assets/fonts/OFL-ChillRoundGothic.txt`

游戏统一使用 Medium 字重的简体中文 TTF。发布游戏时必须连同 OFL 许可证一起分发；
若修改字体文件，还需遵守 OFL 中关于字体名称和再分发的要求。

## 建议的首批运行时选材

- 手牌：`boardgame/PNG/Cards` 中的 140×190 经典牌面；多副牌复用纹理。
- 牌背：`boardgame/PNG/Cards/cardBack_blue2.png`。
- 出牌和发牌声：`casino-audio/Audio/card-place-*.ogg`、`card-slide-*.ogg`。
- 按钮声：`interface-sounds/Audio/click_*.ogg`、`confirmation_*.ogg`、`error_*.ogg`。
- 按钮和面板：从 `ui/PNG` 下按颜色挑选一套，避免混用多个主题。

同一张物理牌的 `deck` 编号只存在于游戏状态中；使用多副牌时可复用相同牌面纹理，
无需复制四份图片。

## 无名杀玩家互动素材（GPL-3.0）

- 目录：`assets/vendor/noname/interactions`
- 来源：<https://github.com/libnoname/noname>
- 原目录：`apps/core/image/emotion/throw_emotion`、`apps/core/audio/effect`
- 内容：鲜花、鸡蛋、酒杯、拖鞋的飞行/命中图片，以及每种互动的两段音效

当前仅用于用户所述的私人构建。该素材不是 CC0；若今后分发包含这些素材的构建，需先
单独确认 GPL-3.0 的源代码提供、许可证保留等合规要求，或替换成许可更宽松的自制素材。

## 无名杀男性快捷语音（GPL-3.0）

- 目录：`assets/vendor/noname/voice/male`
- 来源：<https://github.com/libnoname/noname>
- 原目录：`apps/core/audio/voice/male`
- 内容：编号 0 至 22 的 23 条男性快捷语音
- 说明：`assets/vendor/noname/voice/NOTICE.md`

这些语音与玩家互动素材采用相同的私人使用前提；若今后分发，需要重新处理 GPL-3.0
合规或更换为可独立分发的语音素材。

## 未纳入的备选素材

没有下载先前提到的 OpenGameArt “Playing Cards (Vector & PNG)”。其页面虽然标记为
CC0，但页面评论对人头牌、黑桃 A 图案的原始出处提出了疑问。现有 Kenney 牌组已经
覆盖完整需求，因此暂不把这个存在来源争议的备选包放进可发布素材中。
