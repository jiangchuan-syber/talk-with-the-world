# 划译

**用中文思考，英文直接出现在对话框里——不必先译成英文再重写。**

划译是 Windows 托盘小工具：在任意软件里**选中中文**，按 `Ctrl + Alt + T`，选区会被替换成地道英文。DeepSeek 负责翻译，你继续用母语组织内容即可。

### 适用场景

凡是要和外国友人、客户、同事**打字沟通**的地方，都可以开着划译：

| 场景 | 举例 |
| --- | --- |
| **即时聊天** | 微信、QQ、Telegram、WhatsApp、Slack、Discord、Teams |
| **外贸与商务** | 阿里国际站、邮件、询盘回复、报价说明、合同草稿 |
| **邮件与办公** | Outlook、网页邮箱、飞书、钉钉、Notion、Google Docs |
| **协作与客服** | Zendesk、在线客服、工单备注、跨境电商后台 |
| **社交与内容** | Twitter/X、LinkedIn、Reddit、论坛回帖、视频/直播字幕稿 |
| **学习与差旅** | 论文协作、选课邮件、酒店/签证沟通、线下扫码后的英文回复 |

不用切换「翻译软件 → 复制 → 粘贴」；**思维保持中文**，划译把当前选区变成英文，方便你专注在「说什么」，而不是「怎么写英文」。

### 为什么用划译 + DeepSeek

- **不转变思维模式**：脑子里是中文，选中即发；译的是你已经写好的句子，不是让你先憋英文。
- **DeepSeek 精准翻译**：默认对接 DeepSeek 对话模型，商务、口语、礼貌用语都更贴语境（也可换成其它 OpenAI 兼容接口）。
- **费用极低**：按 API 用量计费，日常划选翻译用量很小；**重度使用一般每天约 ¥0.5 量级**（视模型与字数而定，以 [DeepSeek 官网定价](https://platform.deepseek.com/) 为准）。

安装后从托盘打开设置，填入自己的 API Key 即可使用（密钥只保存在本机，不会打进安装包）。

## 下载（Windows）

| 版本 | 说明 |
| --- | --- |
| [**最新版**](https://github.com/jiangchuan-syber/moss/releases/latest) | 始终指向当前最新 Release |
| [**v0.1.0**](https://github.com/jiangchuan-syber/moss/releases/tag/v0.1.0) | 划译 0.1.0，x64 安装包 `划译_0.1.0_x64-setup.exe` |

直接下载 v0.1.0 安装包（Release 构建完成后可用）：

<https://github.com/jiangchuan-syber/moss/releases/download/v0.1.0/%E5%88%92%E8%AF%91_0.1.0_x64-setup.exe>

## 功能

- **全局快捷键**：选中中文后按 `Ctrl + Alt + T`，自动复制、翻译、粘贴英文结果
- **剪贴板友好**：翻译完成后会尝试恢复翻译前的剪贴板文本
- **系统托盘**：无前台主窗口，通过托盘图标打开设置或暂停/启用
- **可配置 API**：支持 DeepSeek 及任意 OpenAI 兼容的 `chat/completions` 接口
- **可调延迟**：复制/粘贴等待时间可在 80–800 ms 间调节，适配不同应用响应速度

## 环境要求

- **操作系统**：Windows（依赖 UI Automation、全局键盘钩子等 Windows API）
- **Node.js**：用于前端构建与 Tauri 开发脚本
- **Rust**：1.77.2 及以上（见 `src-tauri/Cargo.toml`）
- **API Key**：需在设置中自行填写；密钥不会打包进安装程序，仅保存在本机配置文件中

## 快速开始

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri:dev
```

会启动 Vite 开发服务器并运行 Tauri 应用。

### 构建发布版

```bash
npm run tauri:build
```

产物位于 `src-tauri/target/release/bundle/nsis/`，例如 `划译_0.1.0_x64-setup.exe`。安装包对外名称为 **划译**（`productName`）；当前仅生成 NSIS 安装包（中文产品名下 MSI 易失败）。

发布新版本：推送 `v*` 标签（如 `v0.1.1`）会触发 [`.github/workflows/release.yml`](.github/workflows/release.yml) 自动构建并上传到 [Releases](https://github.com/jiangchuan-syber/moss/releases)。

## 使用方法

1. 首次运行后，从系统托盘打开 **设置**，填写 API Key，并按需调整 API 地址、模型与复制/粘贴等待时间，点击保存。
2. 在 Word、浏览器、编辑器等任意窗口中**选中一段中文**。
3. 按下 **`Ctrl + Alt + T`**，等待翻译完成；选区会被替换为英文译文。
4. 可通过设置页或托盘菜单**暂停/启用**翻译服务。

### 默认 API 配置

| 项 | 默认值 |
| --- | --- |
| API 地址 | `https://api.deepseek.com/v1/chat/completions` |
| 模型 | `deepseek-v4-flash`（快速）/ `deepseek-v4-pro`（质量） |
| 复制/粘贴等待 | 180 ms（可在界面中调节） |

也可将 API 地址改为自建代理或其它兼容服务的完整 URL。

## 技术栈

- **前端**：React 19 + TypeScript + Vite + Tailwind CSS
- **桌面壳**：Tauri 2
- **后端**：Rust（`reqwest` 调用翻译 API，`uiautomation` / 键盘模拟处理选区与剪贴板）

## 项目结构（简要）

```
cn2en/
├── src/                 # React 设置界面
├── src-tauri/
│   ├── src/
│   │   ├── translate_service.rs   # 翻译 API 调用
│   │   ├── selection_translate.rs # 选区复制、粘贴与剪贴板恢复
│   │   ├── input_monitor.rs       # 输入与快捷键监听
│   │   └── tray.rs                # 系统托盘
│   └── tauri.conf.json
└── package.json
```

## 其它 npm 脚本

| 命令 | 说明 |
| --- | --- |
| `npm run dev` | 仅启动 Vite 前端（不启动 Tauri） |
| `npm run build` | 构建前端到 `dist/` |
| `npm run lint` | 运行 ESLint |

## 许可证

见仓库中的许可证文件（如有）。
