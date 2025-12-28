# Auto-Video 🎬

基于 Rust 开发的自动视频生成工具，使用阿里云千问大模型实现从文本到视频的全自动化生成。

## ✨ 功能特性

- 📝 **智能分镜**：使用千问大模型自动将文本内容分解为视频分镜脚本
- 🎨 **AI 图片生成**：每个分镜自动生成对应的 AI 图片（使用千问万相模型）
- 🎙️ **语音合成**：自动将文本转换为语音旁白（使用千问 CosyVoice 模型）
- 🎬 **视频合成**：自动合成图片、字幕和语音为完整视频
- ⚡ **高性能**：基于 Rust 开发，异步处理，性能优异

## 🚀 快速开始

### 前置要求

1. **Rust 环境**（1.70+）
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **FFmpeg**
   ```bash
   # macOS
   brew install ffmpeg
   
   # Ubuntu/Debian
   sudo apt-get install ffmpeg
   
   # Windows
   # 从 https://ffmpeg.org/download.html 下载并添加到 PATH
   ```

3. **阿里云 DashScope API Key**
   - 访问 [阿里云 DashScope](https://dashscope.aliyun.com/) 获取 API Key

### 安装

```bash
# 克隆仓库
git clone https://github.com/yourusername/auto-video.git
cd auto-video

# 编译项目
cargo build --release
```

### 配置

创建 `.env` 文件并配置 API Key：

```bash
cp .env.example .env
# 编辑 .env 文件，填入你的 API Key
```

或者通过命令行参数传入：

```bash
export DASHSCOPE_API_KEY=your_api_key_here
```

## 📖 使用方法

### 基础用法

```bash
# 从文本直接生成视频
./target/release/auto-video --text "这是一个关于春天的故事。万物复苏，生机勃勃。" --output spring.mp4

# 从文件生成视频
./target/release/auto-video --file story.txt --output story.mp4

# 指定工作目录
./target/release/auto-video --text "你的文本内容" --work-dir ./temp --output video.mp4
```

### 命令行参数

```
Options:
  -t, --text <TEXT>          输入文本内容
  -f, --file <FILE>          输入文本文件路径
  -o, --output <OUTPUT>      输出视频文件路径 [默认: output.mp4]
  -w, --work-dir <WORK_DIR>  临时文件工作目录 [默认: ./output]
      --api-key <API_KEY>    DashScope API Key（或设置 DASHSCOPE_API_KEY 环境变量）
  -h, --help                 显示帮助信息
  -V, --version              显示版本信息
```

## 🔧 工作流程

1. **文本分析**：使用千问大模型分析输入文本，生成分镜脚本
2. **图片生成**：为每个分镜调用万相模型生成对应图片
3. **语音合成**：使用 CosyVoice 将文本转换为语音
4. **视频合成**：使用 FFmpeg 将图片、字幕和语音合成为最终视频

## 📁 项目结构

```
auto-video/
├── src/
│   ├── main.rs           # 主程序入口
│   ├── error.rs          # 错误处理
│   ├── api/              # API 客户端
│   │   ├── mod.rs
│   │   └── qwen.rs       # 千问 API 封装
│   ├── scene/            # 场景/分镜处理
│   │   └── mod.rs
│   └── video/            # 视频生成
│       ├── mod.rs
│       └── generator.rs  # 视频合成逻辑
├── Cargo.toml            # 项目配置
├── .env.example          # 环境变量示例
├── .gitignore
└── README.md
```

## 🎯 示例

### 示例 1：生成旅行故事视频

创建 `travel.txt` 文件：
```
今天我们来到了美丽的杭州西湖。
湖面波光粼粼，远处的雷峰塔若隐若现。
漫步在苏堤上，感受着江南的诗情画意。
这里的每一处风景都让人流连忘返。
```

生成视频：
```bash
./target/release/auto-video --file travel.txt --output travel.mp4
```

### 示例 2：生成产品介绍视频

```bash
./target/release/auto-video \
  --text "我们的新产品采用最先进的AI技术，为用户带来前所未有的体验。简单易用，功能强大，是您的最佳选择。" \
  --output product.mp4
```

## ⚙️ 技术栈

- **语言**：Rust 2 **异步运行时**：Tokio
- **HTTP 客户端**：Reqwest
- **CLI 框架**：Clap
- **视频处理**：FFmpeg
- **AI 模型**：
  - 千问 Plus（文本分析）
  - 万相 wanx-v1（图片生成）
  - CosyVoice（语音合成）

## 🐛 故障排除

### FFmpeg 相关问题

如果遇到字幕显示问题，确保系统中有中文字体：

```bash
# macOS - 使用 PingFang 字体（已内置）
# Linux - 安装中文字体
sudo apt-get install fonts-wqy-zenhei

# 修改 src/video/generator.rs 中的字体路径
```

### API 调用失败

- 检查 API Key 是否正确
- 确认账户余额充足
- 检查网络连接

### 视频生成失败

- 确保 FFmpeg 已正确安装
- 检查工作目录的读写权限
- 查看日志输出的错误信息

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

MIT License

## 🙏 致谢

- [阿里云 DashScope](https://dashscope.aliyun.com/) - 提供强大的 AI 能力
- [FFmpeg](https://ffmpeg.org/) - 视频处理工具
- [Rust 社区](https://www.rust-lang.org/) - 优秀的编程语言和生态

## 📮 联系方式

如有问题或建议，请通过以下方式联系：

- 提交 [GitHub Issue](https://github.com/yourusername/auto-video/issues)
- 发送邮件至：your-email@example.com

---

⭐ 如果这个项目对您有帮助，请给个 Star！
