# LeoCard 接入位置速查

路径相对仓库根目录。先用 `rg --files`／`rg` 确认实际位置和符号，再阅读需要的文件；不要为接入一个游戏加载或修改所有路径。

## 选择参考游戏

| 需要的机制 | 优先参考 |
|---|---|
| 普通轮流出牌、多人跟牌 | `crates/games/qigui523/` 及 host/client 的 `qigui523` 模块 |
| 阶段较多、组队、亮主和结算 | `crates/games/shengji/` 及 `shengji` 模块 |
| 回合外响应、连锁效果、多模式 | `crates/games/uno/` 及 `uno` 模块 |
| 隐藏信息、响应窗口、多小局 | `crates/games/mahjong/` 及 `mahjong` 模块 |
| 筹码与底池、行动约束 | `crates/games/texas-holdem/` 及 `texas` 模块 |
| 词库、共用反馈、轮流与抢答 | `crates/games/mc-guess/` 及 `mc_guess` 模块 |

参考组织方式和公共接口，不复制已有游戏的特有规则、玩家数量或具体积分。

## 必须贯通的位置

| 内容 | 位置与检索符号 |
|---|---|
| Workspace 与依赖 | 根 `Cargo.toml`、新增游戏 crate，以及实际引用它的 crate 的 `Cargo.toml` |
| 对外规则入口 | `crates/games/<game>/src/lib.rs`；模块目录用 `mod.rs` 组织 |
| 游戏协议 | `crates/network/protocol/src/game.rs`：`GameKind`、`GameRules`、`GameCommand`、投影与转换；`messages.rs`：`GameSnapshot`；`events.rs`：`GameEvent`；`violations.rs`：`GameViolation`；相应游戏的快照模块与 `lib.rs` 导出 |
| 协议协商 | `crates/network/protocol/src/lib.rs`：`PROTOCOL_VERSION`；检索连接握手与版本拒绝逻辑，不绕过不兼容检查 |
| 房主后端 | `crates/host/src/session.rs`：`GameSetup`、`HostSession` 与后端分发；`lifecycle.rs`：`HostedGameLifecycle`；`host/src/<game>/`：生命周期、命令、快照与计时 |
| 通用房间能力 | `crates/host/src/room/`：身份、权限、座位、机器人、准备、互动与重连；`profiles.rs`：`settle_completed_match_profiles_once`、`records_game_profiles` |
| 客户端模型 | `crates/client/src/model/core.rs`、`access.rs`：快照／事件消费、游戏访问器和玩家元数据 |
| 游戏目录与页面 | `crates/client/src/app/games/catalog.rs`、`composition.rs`、`plugin.rs`、`mod.rs`、`violations.rs`；`screen_change.rs` 处理瞬态快照变化 |
| 本游戏客户端 | `crates/client/src/app/games/<game>/`：UI 状态、规则配置、操作分发、牌桌与表现系统 |

## 只在本次功能需要时检查

- **大厅与首页**：`crates/client/src/app/shell/lobby/`、首页入口资源及 `UiAssets` 加载。模式选择复用 `shell/lobby/mode/`；规则行复用 `presentation/ui/widgets/rules/`。
- **公共玩家组件**：`crates/client/src/app/presentation/ui/widgets/` 下的头像、托管、牌数、互动锚点、文本编辑及滚动组件，和 `crates/client/src/app/shell/chat/`。先找到组件负责的坐标与输入边界。
- **转场与结算**：`crates/client/src/app/presentation/seat_transition/`、`crates/client/src/app/shell/ui/transition/`、`crates/client/src/app/games/summary/`，以及各游戏结算描述器。防止结束按钮被动画可见性或布局遮住。
- **用户设置与档案**：各游戏长期统计定义在 `crates/games/<game>/src/profile.rs`；`crates/network/protocol/src/identity.rs` 只聚合公开档案。检查 `crates/client/app/src/player/preferences/`、`player/profile/` 及其 migration，以及 `crates/client/app/src/app/shell/profile/stats/` 的显示。客户端和房主直接从所属游戏 crate 导入统计类型，不经 protocol 重导出。
- **成就**：`crates/achievements/src/registry/`、`activity/`、`progress/`；房主事件路径与 `crates/client/src/app/achievements/`。成就定义和进度判断归成就 crate。
- **素材发布**：`assets/`、`crates/client/runtime-assets.txt`、`crates/client/build.rs`、`crates/client/src/app/runtime/platform.rs`。检查新资源确实进入生成的 `embedded_runtime_assets.rs`，并符合第三方素材来源要求。
- **验证入口**：游戏 crate 的独立测试模块、`host/src/<game>/tests/`、既有协议／素材检查，以及工作流实际规定的 gate。不要为复用而新增通用测试框架。

## 常用验证方式

按当前仓库的 target 和 feature 配置调整；并行改动时不要格式化或重写无关文件。

```sh
rustfmt --edition 2024 --config skip_children=true <本次修改的 Rust 文件>
cargo clippy -p leocard-<game> -p leocard-protocol -p leocard-host -p leocard-client --all-targets --all-features --locked -- -D warnings
```

仅在允许运行测试时，对核心改动选择相应 `cargo test -p ...`。若用户禁止运行测试，保留测试代码的编译／静态检查并明确没有执行测试；不把 Clippy 成功当成游戏运行验证。
