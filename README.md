# LeoCard

[![CI and Release](https://github.com/LeoDreamer2004/LeoCard/actions/workflows/ci.yml/badge.svg)](https://github.com/LeoDreamer2004/LeoCard/actions/workflows/ci.yml)

使用 Rust、Bevy 开发的跨平台局域网棋牌游戏。目前包含七鬼五二三、德州扑克、升级（拖拉机）、UNO 牌及变种和麻将。

**本项目完全由 AI 开发，不保证代码质量和可维护性。**——当然，本人也会对质量做整体把关，当明显代码质量不佳时会使用 AI 进行大批量重构 (可以参考 [大幅重构记录](https://github.com/LeoDreamer2004/LeoCard/commit/96c51687c46f9fdf608d6e751f81da2259dce13f))。

运行当前联机客户端：

```bash
cargo run -p leocard-client
```

开发时客户端会从工作区根目录的 `assets` 加载素材。Release 构建会自动把运行时所需的图片、字体、音频和着色器嵌入可执行文件，因此分享时不需要附带 `assets` 目录：

```bash
cargo build -p leocard-client --release
```

内嵌文件列表由 `crates/client/runtime-assets.txt` 管理

## 许可证

LeoCard 的原创代码以 [GPL v3.0](LICENSE) 发布。分发修改版时须按
GPL-3.0 提供相应源代码和许可证声明。第三方字体、图片、音频等素材继续使用各自的许可证，具体来源、署名和再分发要求见 [ASSETS.md](./assets/ASSETS.md)。
