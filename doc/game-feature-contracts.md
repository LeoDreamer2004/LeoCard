# 游戏能力接口重构

## 目标与边界

覆盖十款游戏的房主会话、客户端页面、输入、结算、档案及可选表现能力。trait 用于约束统一的输入输出、生命周期和语义，即使各游戏的实现不同也应遵循同一接口。

保持现有规则、协议和存档布局。继续兼容已发布版本的存档，不兼容未发布 PR 的中间格式。能力接口独立实现，不引入运行时注册或把所有可选能力挂到基础生命周期的父 trait 上。各游戏拥有自己的实现，公共层负责接口和必要的调度。

## 重构前审查记录

- 十款游戏已实现 `HostedGameLifecycle`、`HostBackend`、`GameScreen` 和 `DomainUiAction`。
- 九款游戏使用 `UiActionHandler`，MC 竞猜仍手工遍历 UI 消息。
- 公共结算调度直接引用麻将、21 点和 MC 竞猜的私有播放资源，结算描述和播放门控需要统一。
- 房主的固有 `game_snapshot` 与生命周期同名方法返回不同类型，需区分具体快照和协议包装。
- 德州结算在 commands，掼蛋结算在 state，七鬼的 analysis 还承载动作执行。按职责调整模块及接口。
- 客户端 reconcile/clear 的输入、清理范围和返回值不同，应明确页面协调接口。
- 操作耗时、规则倒计时、动画时钟属于不同能力。UI 意图、网络命令和规则动作也应分层。
- 掼蛋顺位记录小局游次，德州等记录整场排名，档案接口必须明确统计周期。
- 记牌器、手牌选择、音频反馈适合作为可选能力。support、layout、cards 等名称不直接对应一个统一功能。

## 实施清单

| 项目 | 接口与范围 | 状态 |
| --- | --- | --- |
| 1 UI action | MC 竞猜接入现有 `UiActionHandler`，保留文本输入独立流程 | 已实现 |
| 2 页面生命周期 | 独立页面协调和行动提醒接口，页面组合继续使用 `GameScreen` | 已实现 |
| 3 客户端结算 | 统一结算描述和播放门控，公共调度不引用游戏私有资源 | 已实现 |
| 4 房主快照 | 统一具体快照接口，协议包装只在边界完成 | 已实现 |
| 5 房主命令 | 统一带具体命令类型的处理接口 | 已实现 |
| 6 决策计时 | 独立决策窗口接口，各游戏负责完成判定与档案写入 | 已实现 |
| 7 自动行动 | 统一游戏时间推进接口，保留各游戏机器人与托管策略 | 已实现 |
| 8 权威结算与模块职责 | 区分小局和整场结算接口，调整不符合职责的实现位置 | 已实现 |
| 9 档案能力 | 局数、增减、顺位、用时采用独立接口，统一档案展示入口 | 已实现 |
| 10 记牌器 | 统一计数投影和窗口状态接口，各游戏保留牌数计算 | 已实现 |
| 11 手牌选择 | 统一选择读写约定，各游戏保留合法性及交互处理 | 已实现 |
| 12 音频和反馈 | 明确事件/牌型到反馈的接口，保留播放顺序和去重 | 已实现 |
| 13 回归验证 | 格式、Clippy、相关测试、Android 和旧存档兼容验证 | 已通过 |
| 14 客户端 feature 分组 | 公共能力接口和对应公共调度集中到 `games/feature/` | 已实现 |
| 15 页面变化 | `GameScreenChange` 下发到支持局部更新的游戏，公共层只分派 | 已实现 |
| 16 游戏目录与大厅信息 | 名称、简介、人数约束和规则标签由各游戏实现 | 已实现 |

## 实现约束

- 必选接口应有实际方法，不用空标记代替约定。不用默认空实现掩盖遗漏。
- 具体命令、快照和输出使用关联类型，上下文可用关联类型或泛型参数，避免向通用层暴露游戏内部结构。
- 自动行动不计作人类耗时，拒绝及重放不重复计数，重连重新开始等待窗口。
- 结算只提交一次，区分小局演出和整场档案、钱包结算。
- 页面切换清理、事件去重和动画暂停语义保持原状，局部交互不扩大成全局 UI 重建。
- traits 和实现分层，UI trait 留在客户端，房主 trait 留在 host，纯档案接口可放在游戏公共库。
- trait 实现直接放在业务所属模块中，由 trait 方法承载实际逻辑。不保留只负责调用原函数的转发方法，也不为沿用原调用方式再包装一次 trait 方法。
- `use` 统一放在所属模块顶部，不放入函数体。测试专用导入放在测试模块顶部，平台专用导入保留相应的 `cfg` 条件。
- 客户端公共能力统一放在 `games/feature/`，游戏根目录保留游戏组合、插件和目录顺序。具体实现仍放在每个游戏自己的业务模块。

## 实现对照

### 1 至 3 客户端页面与动作

公共接口位于 `crates/client/app/src/app/games/feature/`：

- `screen.rs` 的 `GameScreen` 只负责大厅和牌桌渲染。
- `ui_lifecycle.rs` 的 `GameUiLifecycle` 负责快照协调、拒绝回滚和离开页面时的清理。
- `action_prompt.rs` 的 `GameActionPrompt` 判断当前是否等待本人必须执行的决策。
- `summary.rs` 的 `GameSummary` 提供结算描述及播放暂停位置。

十款游戏的页面渲染、协调和行动提醒实现在各自 `screen.rs`。`GameSummary` 实现直接放在各游戏的 `summary.rs` 或 `settlement` 模块内，方法中直接构造描述和处理播放门控，不在 `screen.rs` 添加转发。`composition.rs` 仅做游戏类型分派，`feature/summary.rs` 中的 `sync_summary_playback` 通过页面接口获取结果，不再读取麻将、21 点、MC 竞猜的私有资源。升级保留独立结算演出，其公共结算描述明确返回 `None`。

MC 竞猜的 `actions.rs` 采用已有 `DomainUiAction`、`UiActionHandler` 和 `dispatch_domain_actions`。输入框、提交事件、内容同步保留在独立 `input.rs`，离开 MC 竞猜时的模式菜单清理由页面生命周期承担。

### 4 至 8 房主能力与职责

公共接口位于 `crates/host/src/features/`，各游戏自己实现：

| 接口 | 覆盖 | 实际调用入口 |
| --- | --- | --- |
| `GameSnapshotView` | 十款游戏 | `HostedGameLifecycle::game_snapshot` 将具体视图包装为协议快照 |
| `GameCommandHandler` | 十款游戏 | `HostedGameLifecycle::handle_game_command` 解析后调用具体命令处理 |
| `DecisionTiming` | 十款游戏 | 各游戏 `timing.rs` 的请求计时和窗口刷新 |
| `GameAutomation` | 十款游戏 | 各游戏 `advance_time` 驱动机器人、托管、规则截止时间和自动演出 |
| `MatchSettlement` | 除麻将连连看、升级外八款游戏 | 游戏结束流程提交整场档案和钱包结算 |
| `HandSettlement` | 斗地主、德州、升级 | 现有独立小局完成入口，升级按小局提交档案和钱包结算 |

`HostedGameLifecycle` 只要求命令和快照这两个基本协议能力，不要求计时、自动行动或结算能力。没有运行时注册。`DecisionTiming` 只约定决策窗口和命令分类，耗时存储、请求接受判定和写入仍在每个游戏内部。

具体快照直接在 `GameSnapshotView::game_view` 中构造，协议层 `game_snapshot` 只负责包装，不保留会话内的 `build_snapshot` 转发。MC 竞猜新增自己的 `snapshot.rs`，掼蛋命令从生命周期移到 `commands.rs`。

德州、掼蛋、麻将和斗地主的结算移到各自 `settlement.rs`，其他已有结算模块沿用。原有的 `settle_match`、`apply_finished_reference_points`、`finish_hand_if_needed` 等纯转发方法移除，由调用方直接调用结算 trait。七鬼原来的 `analysis.rs` 拆回 `commands.rs` 的动作执行、`publication.rs` 的事件广播和 `statistics.rs` 的档案投影。

麻将连连看原本没有整场档案和钱包结算，因此不补空结算实现。升级原本每小局生成独立结算标识、提交档案和参考分，采用 `HandSettlement`，不误称为整场。其他游戏的小局推进已由规则层或自己的结算流程管理，不为凑齐覆盖添加无意义的 `HandSettlement` 包装。

### 9 档案能力

纯数据接口定义在 `crates/games/common/src/profile.rs`，实际实现保留在每个游戏库的 `profile.rs`：

| 接口 | 覆盖 | 语义 |
| --- | --- | --- |
| `ActionTimeProfile` | 十款游戏 | 读取、写入该游戏档案中的操作耗时 |
| `GameCountStats` | 除麻将连连看外九款游戏 | 已完成对局数及统计周期 |
| `ReferenceStats` | 除麻将连连看外九款游戏 | 累计参考分增减 |
| `PlacementStats` | 掼蛋、21 点、麻将、MC 竞猜、七鬼、德州、UNO | 顺位计数及统计周期 |

`ResultScope` 明确计数周期。`COMPLETION_SCOPE` 在升级为 `Hand`，其他已有对局数统计为 `Match`。`PLACEMENT_SCOPE` 在掼蛋为 `Hand`，表示小局游次，其他已有顺位统计为 `Match`。斗地主和升级没有原有顺位字段，因此不虚构该能力。

客户端 `shell/profile/stats/view.rs` 定义 `GameProfileRows`，十款游戏各自生成档案行，档案页只做类型分派。平均操作用时继续由每款游戏自己排列，空值沿用 `--`。没有恢复统一追加时间行的函数，也没有恢复 `ActionTimedGame`。

### 10 至 12 可选交互与反馈

| 接口 | 覆盖 | 边界 |
| --- | --- | --- |
| `CardCounter` | 斗地主、掼蛋、七鬼、升级 | 统一窗口状态与计数显示投影，牌数计算和顺序仍归游戏 |
| `HandSelection` | 斗地主、掼蛋、七鬼、升级、UNO | 统一实体牌选择集合的读取、修改、清空和替换，合法性与提示不进入公共层 |
| `GameFeedback` | 除麻将连连看外九款游戏 | 将游戏事件、动作或牌型映射为自己的反馈输出 |

接口文件分别为 `games/feature/card_counter.rs`、`games/feature/hand_selection.rs` 和 `games/feature/feedback.rs`。所有接口都有现有渲染或动作流程调用。

反馈输出保留具体类型，可返回语音、音频计划、静态提示切片或 MC 音符迭代器。音色、强弱、时序、资源选择、重放去重仍在游戏内。MC 音符和 21 点动作提示不新增堆分配。

反馈映射直接写在 `GameFeedback::feedback` 中，不再经过 `pattern_voice`、动作音频计划等同义转发函数。麻将的动作语音映射放在素材模块内实现，直接访问该模块自己的资源。真正被多处复用的时间轴、音频片段组合及渲染工具继续留在所属模块。

`support`、`layout`、`cards` 等同名模块可能分别处理协议支持、布局细节、牌面资源或规则牌型，不强行按文件名统一。规则倒计时、操作耗时和动画时钟也不合并为一个计时能力。

### 14 至 16 公共 feature 分组、页面变化与目录信息

`games/feature/` 按能力分文件，`mod.rs` 只声明模块和导出接口。`GameScreen`、生命周期、行动提醒、结算、记牌器、选牌、反馈、页面变化、元信息和大厅规则集中在这里。`games/mod.rs` 统一导出常用接口，调用方不依赖其文件位置。

`feature/screen_change.rs` 定义 `GameScreenChange::is_transient_change`。实际比较逻辑直接实现在五款游戏自己的模块：

| 游戏 | 实现位置 | 允许局部更新的变化 |
| --- | --- | --- |
| 21 点 | `blackjack/state.rs` | 决策倒计时 |
| MC 竞猜 | `mc_guess/turn_timer.rs` | 竞猜倒计时 |
| 德州 | `texas/spectator/state.rs` | 异步观战胜率 |
| 七鬼 | `qigui523/turn_timer.rs` | 回合倒计时 |
| 升级 | `shengji/snapshot.rs` | 发牌进度及亮主宽限、抄底倒计时 |

公共 `ScreenChangeCapture` 只捕获和分派这五类快照，保留原先的克隆范围。切换游戏、缺少快照以及其他字段变化仍按原逻辑处理，不为其他游戏补空实现，也不新增运行时注册。原来的 `only_*_changed` 独立函数及导出移除。

`feature/metadata.rs` 的 `GameMetadata` 提供 `GameDescriptor`，包含游戏类型、名称和简介。`feature/lobby.rs` 的 `GameLobbyRules` 提供座位上限、开始人数校验和三个大厅标签。十款游戏分别在自己的 `catalog.rs` 实现这两个接口，公共 `GameRules` 实现只按枚举类型分派。

根目录 `catalog.rs` 只按原顺序引用各游戏的元信息。大厅直接调用规则能力，删除 `game_seat_count`、`game_accepts_player_count`、`lobby_rule_labels` 等旧入口。游戏列表顺序、名称、简介、人数限制和标签文案均保持原状，包括 UNO 各模式的标签优先级。

## 验证记录

完成项必须有调用方，不能只声明 trait 或保留无人调用的适配层。复用现有回归测试，本次没有新增测试。

- 桌面工作区 Clippy：`cargo clippy --workspace --exclude leocard-android --all-targets --all-features --offline -- -D warnings` 通过。
- Android arm64 Clippy：API 31、NDK 29、全部 feature，`-D warnings` 通过。
- 工作区测试：`cargo test --workspace --exclude leocard-android --all-features --offline`，642 项通过，0 失败、0 忽略。
- 升级统计周期接口调整后补跑：`cargo test -p leocard-host -p leocard-client --all-features --offline`，261 项通过，0 失败、0 忽略。包含操作计时、结算、观战、页面资源借用及档案迁移回归。
- `cargo fmt --all -- --check` 和 `git diff --check` 通过。
- 十款游戏的档案结构字段与顺序均未改变，没有修改协议或迁移格式。
- 发布版兼容：使用临时独立工具，依据 `v0.12.0` 的字段声明生成档案 v8、钱包 v6 样本，再通过当前构建的公共加载接口读取并重复加载。身份、九款旧游戏档案、评分、结算凭据、互动数据、余额、道具权益、拥有和装备的头像框均保留，文件分别迁移为档案 v9 和钱包 v7。
- 没有启动图形客户端，未进行手工画面、音频或触屏验收。

### 实现位置与转发清理后的补充验证

- 十款游戏的 `GameSummary` 均放回各自结算或 summary 模块，房主快照和音频映射也由 trait 方法直接实现，共移除 42 个冗余函数。
- 逐项对照九款既有快照构造、九款既有结算描述和六组音频映射，除方法接收者和局部变量名称外，主体逻辑保持一致。
- 桌面工作区全部目标、全部 feature 的 Clippy 及 Android arm64 Clippy 均通过，`-D warnings` 无警告。
- 房主和客户端现有测试重新运行，261 项通过，0 失败、0 忽略，没有新增测试。
- 本次修改文件的 rustfmt 检查和 `git diff --check` 通过，函数体内 `use` 无残留。

### feature 分组、页面变化和目录能力的补充验证

- 桌面工作区全部目标、全部 feature 的 Clippy，以及 Android arm64 Clippy 均通过，`-D warnings` 无警告。
- 客户端现有测试 130 项全部通过，没有新增测试。
- 工作区 rustfmt 和 `git diff --check` 通过。
- 五款游戏的局部更新比较主体与迁移前一致，名称和简介逐字保留。目录只保留原有顺序，原有大厅人数判定和标签逻辑移入各游戏实现。
