# Python 环境初始化助手

基于 **Rust + Tauri + Vue 3** 打造的 Windows Python3 环境变量检测与初始化工具。应用遵循知攻系统统一设计标准，提供离线密钥认证、主题切换以及自定义标题栏等能力。

## 功能特性

- ⚙️ **环境检测**：自动扫描当前目录下的 `python.exe`/`python3.exe`，并检查 `python3` 环境变量、`PATH` 中的 Python 目录与 Scripts 目录。
- 🚀 **一键初始化**：在 Windows 上自动创建/更新 `python3` 环境变量、补全 PATH 配置、写入全局 `pip.ini` 并添加国内镜像源。
- 🔐 **离线密钥认证**：启动时读取 `keyzhigongfile` 指向的密钥文件，使用 RSA + AES 组合加密完成设备与时间验证。
- 🎨 **主题系统**：内置浅色、深色与跟随系统三种模式，通过自定义标题栏按钮即可切换。
- 🪟 **自定义窗口**：实现无边框窗口、拖拽、最小化/最大化/关闭等控制逻辑。
- 🖼️ **构建时图标拉取**：遵循国内网络策略，在编译阶段自动从国内镜像拉取应用图标，无需提交二进制资源。

## 开发指引

### 先决条件

- Rust 1.74+ 与 Tauri 构建依赖
- Node.js 18+ 与 npm

### 本地运行

```bash
npm install
npm run tauri dev
```

### 构建发布

```bash
npm install
npm run tauri build
```

构建脚本会自动从国内镜像下载图标文件 `icons/app-icon.ico`，请确保网络能够访问阿里云或清华 TUNA 镜像。

## 目录结构

- `src/`：Vue 3 + Tailwind 前端工程
- `src-tauri/`：Tauri 后端（Rust）
  - `offline_key.rs`：离线密钥认证实现
  - `python_env.rs`：Python 环境检测与初始化逻辑
  - `config.rs`：主题配置持久化
  - `build.rs`：构建期自动拉取图标

## 许可证

本项目示例代码仅供内部评审与演示使用，具体授权以实际合同为准。
