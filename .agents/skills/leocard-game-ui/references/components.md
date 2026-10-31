# 游戏 UI 公共组件入口

路径相对仓库根目录。只读本次相关入口及调用方，接口调整后通过 `rg` 查当前定义。

## 基础层与页面组合

| 用途 | 入口与参考 |
|---|---|
| 场景、纹理按钮、文本、输入、滚动条 | `crates/client/ui/src/`，查 `UiComposer`、`UiSceneKey`、`TextureButton`、`TextInput`、`TexturedScrollbar`、`sliced_image` |
| 锚定气泡、浮窗边缘避让 | `crates/client/ui/src/bubble/`、`placement.rs`、`anchored_popup.rs`；聊天层将玩家解析为锚点 Entity，UI 层不依赖协议 |
| 抽屉滑动与滚动范围 | `crates/client/ui/src/drawer.rs` 的 `DrawerSlide`、`animate_drawers`，以及 `scrollbar/scroll.rs` 的 `scroll_viewport_y`；应用保留重建时的进度和滚动位置 |
| 尺寸、语义色与 Android 倍率 | `crates/client/app/src/app/presentation/ui/constants.rs` |
| 普通页面皮肤 | `crates/client/app/src/app/shell/ui/cozy.rs` |
| OreUI 皮肤 | `crates/client/app/src/app/games/mc_guess/assets/skin.rs`，结合 `table/`、`graph/guide/` 调用查看 |
| 画布缩放与窗口适配 | `crates/client/app/src/app/shell/input/ui.rs` |
| 顶栏、菜单和安全边距 | `crates/client/app/src/app/shell/ui/header/` |

## 大厅、玩家与结算

| 用途 | 入口与参考 |
|---|---|
| 默认大厅与标题 | `crates/client/app/src/app/shell/lobby/view/`，查 `LobbyPage`、`LobbyPageConfig` |
| 模式选择案例 | `crates/client/app/src/app/games/uno/lobby/`、`crates/client/app/src/app/games/mc_guess/lobby/` |
| 配置行与帮助 | `crates/client/app/src/app/presentation/ui/widgets/rules/`、`crates/client/ui/src/rules/` |
| 各人数座位及牌桌方向 | `crates/client/app/src/app/presentation/seats/` |
| 头像、互动、准备和房主标记 | `crates/client/app/src/app/presentation/ui/widgets/players.rs`，查 `add_avatar`、`add_ready_avatar`、`attach_player_interactions`、`PlayerMenuProfile` |
| 剩余牌数 | `crates/client/app/src/app/presentation/ui/widgets/hand_count.rs` |
| 公共结算及动效 | `crates/client/app/src/app/presentation/summary/`，查 `RankedSummary`、`SummaryPlayer`、`add_ranked_summary` |
| 结算接入案例 | `crates/client/app/src/app/games/doudizhu/summary.rs`、`crates/client/app/src/app/games/texas/settlement.rs` |

## 交互和动效

| 用途 | 入口与参考 |
|---|---|
| 牌桌按钮 | `crates/client/app/src/app/presentation/ui/widgets/game_button.rs`，查 `GameButtonSpec`、`GameButtonTone`、`scaled` |
| 手牌尺寸、布局、拖选 | `crates/client/app/src/app/presentation/hand.rs`、`crates/client/ui/src/selection.rs` |
| 选中高亮 | `crates/client/app/src/app/presentation/hand_style.rs`；接入参考 `crates/client/app/src/app/games/doudizhu/hand_interaction.rs` |
| UNO 牌面和飞牌几何 | `crates/client/app/src/app/games/uno/hand.rs`、`crates/client/app/src/app/games/uno/view.rs`、`crates/client/app/src/app/games/uno/presentation/flight.rs` |
| 后台行动提醒及名称彩蛋 | `crates/client/app/src/app/runtime/notifications/`、`crates/client/app/src/platform/notifications/`、各游戏 `screen.rs` 的 `GameScreen::requires_action`；公共身份见 `crates/client/app/src/player/persona.rs` 的 `PlayerPersona` |
| 分数反馈 | `crates/client/app/src/app/presentation/score_motion.rs` |
| 缓动及座位过渡 | `crates/client/app/src/app/presentation/motion.rs`、`crates/client/app/src/app/presentation/seat_transition/` |
| 页面转场 | `crates/client/app/src/app/shell/ui/transition/` |
| 二择弹窗与普通提示 | `crates/client/app/src/app/shell/ui/confirmation/`、`crates/client/app/src/app/shell/ui/modal/` |

跨游戏功能放在 app 公共表现层；不依赖游戏和业务状态的通用控件才下沉到 `leocard-ui`。素材通过现有 assets crate、`UiAssets` 和 `crates/client/assets/runtime-assets.txt` 注册，第三方来源与许可沿用项目约定。
