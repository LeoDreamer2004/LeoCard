# LeoCard

[![CI and Release](https://github.com/LeoDreamer2004/LeoCard/actions/workflows/ci.yml/badge.svg)](https://github.com/LeoDreamer2004/LeoCard/actions/workflows/ci.yml)

使用 Rust、Bevy 开发的跨平台局域网棋牌游戏。目前包含七鬼五二三、德州扑克、升级（拖拉机）、UNO 牌及变种、麻将和 MC 竞猜。

**本项目完全由 AI 开发，不保证代码质量和可维护性。**——当然，本人也会对质量做整体把关，当明显代码质量不佳时会使用 AI 进行大批量重构 (可以参考 [大幅重构记录](https://github.com/LeoDreamer2004/LeoCard/commit/96c51687c46f9fdf608d6e751f81da2259dce13f))。

运行当前联机客户端：

```bash
cargo run -p leocard-client
```

开发时客户端会从工作区根目录的 `assets` 加载素材。内嵌文件列表由 `crates/client/runtime-assets.txt` 管理。

## Minecraft 启动器

桌面端首页顶栏提供「MC 启动器」。它可以从 Mojang 下载 Java 版正式版和快照，安装 Fabric、Forge 和 NeoForge，并使用离线玩家名或微软账号启动已安装的版本。资源下载可选择 Modrinth 或 CurseForge，支持 Mod、资源包、光影和数据包。下载 Mod 时会安装平台标记为必需的依赖。游戏文件默认保存在系统数据目录的 `leocard/minecraft` 下，可通过 `LEOCARD_MINECRAFT_DIR` 指定其他位置。

启动前需要在电脑上安装适合该游戏版本的 Java。启动器优先使用 `LEOCARD_JAVA_PATH`，其次使用 `JAVA_HOME`，最后查找系统路径中的 Java。Forge 和 NeoForge 安装也需要 Java。每个版本的存档和配置放在独立的 `instances/<版本>` 目录，启动日志写入该目录的 `launcher.log`。Android 端暂不提供该入口。

微软正版登录需要先注册支持个人微软账号及公共客户端设备码流的 Azure 应用，并将其应用（客户端）ID 设为 `LEOCARD_MICROSOFT_CLIENT_ID`。点击「微软账号正版登录」后，按界面给出的地址和代码完成授权。登录会验证 Xbox 授权、Minecraft Java 版购买资格和玩家资料；账号会话只保留在当前运行的客户端内。CurseForge 需要将官方 API 密钥设为 `LEOCARD_CURSEFORGE_API_KEY`。若作者禁止第三方下载某个文件，客户端会提示改用其他资源。

自 0.7.0 版本起，本游戏支持 ARM64 的 Android 12 及以上客户端。

## 许可证

LeoCard 的原创代码以 [GPL v3.0](LICENSE) 发布。分发修改版时须按
GPL-3.0 提供相应源代码和许可证声明。第三方字体、图片、音频等素材继续使用各自的许可证，具体来源、署名和再分发要求见 [ASSETS.md](./assets/ASSETS.md)。
