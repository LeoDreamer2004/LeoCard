# Android 发布

CI 使用 JDK 21、ARM64 Android 12 及以上目标。普通提交和 PR 检查原生代码并验证
APK 打包；推送与 Cargo 版本一致的 `v*` 标签时构建、签名并发布正式 APK。

GitHub Release 同时包含 Linux、Windows 和以下 Android 文件：

- `leocard-android-arm64.apk`
- `leocard-android-arm64.apk.sha256`

发布依赖这四个仓库 Actions Secrets，缺失时 CI 会停止发布：

- `ANDROID_KEYSTORE_BASE64`：PKCS12 发布密钥文件的 Base64 内容。
- `ANDROID_KEYSTORE_PASSWORD`：密钥库密码。
- `ANDROID_KEY_ALIAS`：签名密钥别名。
- `ANDROID_KEY_PASSWORD`：签名密钥密码。

本机签名备份位于被 Git 忽略的 `secrets/android-release.p12` 和
`secrets/android-release.env`。请另外备份这两个文件；后续版本必须沿用同一签名
才能覆盖升级。不要提交或公开密钥、密码文件。调试 APK 使用另一份调试签名，
安装正式版前需要卸载已有调试版。

本地构建正式版可在准备 Android 工具后，加载 `secrets/android-release.env`，
使用 `cargo ndk -t arm64-v8a -P 31 -o target/android/jniLibs build -p leocard-client --lib --release --locked`
编译原生库，再用 Gradle 执行 `:app:assembleRelease`。
