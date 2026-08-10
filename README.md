# LeoCard

使用 Rust、Bevy 开发的局域网棋牌游戏。目前包含七鬼五二三、德州扑克和升级（拖拉机）。
**本项目完全由 AI 辅助开发，不保证代码质量和可维护性。**

运行当前 2–6 人 TCP 联机客户端：

```bash
cargo run -p leocard-client
```

开发时客户端会从工作区根目录的 `assets` 加载素材。Release 构建会自动把运行时所需
的图片、字体、音频和着色器嵌入可执行文件，因此分享时只需分发 `leocard`（Windows
下为 `leocard.exe`），不需要附带 `assets` 目录：

```bash
cargo build -p leocard-client --release
```

内嵌文件列表由 `crates/client/runtime-assets.txt` 管理

## 许可证

LeoCard 的原创代码以 [GNU GPL v3.0（仅此版本）](LICENSE)发布。分发修改版时须按
GPL-3.0 提供相应源代码和许可证声明。第三方字体、图片、音频等素材继续使用各自的
许可证，具体来源、署名和再分发要求见 [ASSETS.md](ASSETS.md)。
