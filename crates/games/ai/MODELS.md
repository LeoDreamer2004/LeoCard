# 外部游戏模型

模型权重通过原生客户端的“游戏设置 / AI”页面单独下载，不放入游戏素材包。
转换、校验和 CPU/GPU 推理都在 Rust 中执行，无须安装 Python 或 PyTorch。
网页客户端不编译 `leocard-ai` 和各游戏的模型适配模块，也不显示 AI 设置。
网页玩家仍可加入由原生房主运行模型的房间。

## 已接入

| 游戏 | 模型 | 权重下载量 | 许可证 | 来源 |
| --- | --- | --- | --- | --- |
| 三人斗地主 | DouZero-WP 的三个角色 | 约 18 MB | Apache-2.0 | [DouZero](https://github.com/kwai/DouZero)，[固定版本权重](https://huggingface.co/palemoky/douzero-baselines/tree/57b3914046c2a0877016b8b8830fd07cf5b0ba08/checkpoints/douzero_WP) |
| 国标麻将 | kdens3 三学生模型 | 约 80 MB | MIT | [发布者模型卡](https://huggingface.co/Dannibal/ijcai-mahjong-ckpts-2026/blob/6964d80b9f8a6670dda44f14b664c61112f61009/README.md) |
| 掼蛋 | Guandan Actor 的 master 检查点 | 约 8 MB | MIT | [固定版本源码和权重](https://github.com/746505972/guandan/tree/7d37216369d8b003e94a2779c82f497a621f6f7b) |

下载地址、版本、字节数和 SHA-256 集中在 `src/catalog.rs`。
下载后转换为带版本头的 f16 数据缓存，读取时展开为 f32 计算。
缓存放在玩家配置目录的 `models/<模型>/<版本>`，写入权重后最后原子写入清单。
缓存校验失败时可重新下载。来源版本发生变化时使用新的目录。

## 配置生效范围

每个游戏单独选择是否启用，推理设备可选自动、CPU 或 GPU。
下载代理端口可指定本机 `127.0.0.1` 的 HTTP 代理，留空沿用系统代理设置。
自动模式优先使用支持计算的硬件 GPU，初始化或执行失败时改用 CPU。
强制 GPU 需要检测到可用设备。
只有房主的设置生效，准备大厅中的机器人同步改名为“觉悟人机 + 编号”。
开局时保留权重和设备句柄，游戏中切换设置或删除本地模型不替换正在进行的对局。
未下载、未启用或不支持的游戏继续使用原有策略。三人斗地主默认使用纯规则策略。

模型定义、特征和合法动作映射在每个游戏内部，共享 crate 只负责格式、缓存句柄和推理算子。
DouZero 的 LSTM 历史编码使用 CPU，全连接头支持 CPU/GPU。
kdens3 支持卷积、残差和全连接，在时间预算内合并学生模型的合法动作 softmax。
掼蛋保留发布模型的输入布局，九张、十张炸弹等超出检查点动作空间的局面交回原有策略。
本项目的报听、补花、进贡等扩展仍由本地规则约束，模型输出只能选择当前合法动作。
接入并不代表这些模型在本项目规则下均已证明胜率高于原有策略。

## 其余游戏的检索结论

本次未找到能够直接接入且同时具备公开权重、可分发许可和匹配玩法的双升、UNO、
德州、七鬼五二三、二十一点、推麻将连连看、Minecraft 猜词、你画我猜模型。
双升的 ShengJi+ 有公开检查点，但许可和规则适配尚不能确认。
部分 UNO 项目只有训练代码、无许可证，或作者自己的评估弱于规则策略。
RLCard 的 Leduc 模型不能作为多人无限注德州策略直接使用。
这些游戏不显示占位的模型下载条目。

## 验证方法

普通测试不联网、不要求下载权重或存在硬件 GPU。
需要复核真实模型时，开发者可使用本目录 `tools/prepare_reference.py` 下载固定检查点，
并用 NumPy/PyTorch 生成独立数值参考。这个脚本仅用于开发验证。

```sh
python crates/games/ai/tools/prepare_reference.py /tmp/leocard-ai-models
LEOCARD_AI_TEST_MODELS=/tmp/leocard-ai-models cargo test -p leocard-doudizhu -p leocard-mahjong -p leocard-guandan native_inference_matches_pytorch -- --ignored
LEOCARD_AI_TEST_MODELS=/tmp/leocard-ai-models LEOCARD_AI_TEST_GPU=1 cargo test -p leocard-doudizhu -p leocard-mahjong -p leocard-guandan native_inference_matches_pytorch -- --ignored
```

GPU 校验必须在能够访问真实 GPU 的环境中显式运行。
