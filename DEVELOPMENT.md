# 开发文档

## 项目架构

### 模块划分

```
src/
├── main.rs           # 程序入口，CLI参数解析和主流程控制
├── error.rs          # 统一错误处理
├── api/              # API客户端模块
│   ├── mod.rs        # API模块导出
│   └── qwen.rs       # 千问API封装（文本、图片、语音生成）
├── scene/            # 场景/分镜数据结构
│   └── mod.rs        # Scene结构定义
└── video/            # 视频生成模块
    ├── mod.rs        # 视频模块导出
    └── generator.rs  # 视频合成逻辑（FFmpeg封装）
```

### 核心流程

1. **文本输入** → CLI 参数解析
2. **分镜生成** → 调用千问大模型分析文本
3. **图片生成** → 并行调用万相API生成图片
4. **语音合成** → 调用CosyVoice生成语音
5. **视频合成** → 使用FFmpeg合成最终视频

### API集成

#### 千问文本生成 (qwen-plus)
- 用途：分析输入文本，生成分镜脚本
- 输入：原始文本
- 输出：JSON格式的分镜列表

#### 万相图片生成 (wanx-v1)
- 用途：根据分镜描述生成图片
- 模式：异步任务，需轮询结果
- 参数：style=photography, size=1280*720

#### CosyVoice语音合成
- 用途：将文本转换为语音
- 音色：longxiaochun
- 格式：MP3

## 开发指南

### 环境设置

```bash
# 安装Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装FFmpeg
brew install ffmpeg  # macOS
# 或 apt-get install ffmpeg  # Linux

# 克隆项目
git clone <your-repo>
cd auto-video

# 配置环境变量
cp .env.example .env
# 编辑 .env，填入API Key
```

### 构建和测试

```bash
# 开发模式构建
cargo build

# 发布模式构建
cargo build --release

# 运行测试
cargo test

# 代码检查
cargo check

# 格式化代码
cargo fmt

# Lint检查
cargo clippy
```

### 调试技巧

1. **启用详细日志**
   ```bash
   RUST_LOG=debug ./target/debug/auto-video --text "测试"
   ```

2. **保留临时文件**
   - 在 `work-dir` 中查看中间生成的图片和音频
   - 检查FFmpeg生成的视频片段

3. **API调试**
   - 使用 `reqwest` 的日志功能查看HTTP请求
   - 检查API响应JSON

### 添加新功能

#### 添加新的AI服务

1. 在 `src/api/` 创建新的客户端模块
2. 实现统一的接口
3. 在 `main.rs` 中集成

#### 扩展视频效果

1. 修改 `src/video/generator.rs`
2. 添加新的FFmpeg滤镜和效果
3. 更新Scene结构以支持新参数

## 性能优化

### 当前实现

- 异步API调用（Tokio运行时）
- 图片生成串行（避免API限流）
- 语音生成一次性（整段文本）

### 可优化项

1. **并行图片生成**
   - 使用 `tokio::spawn` 并行生成多个图片
   - 注意API速率限制

2. **缓存机制**
   - 对相同描述的图片进行缓存
   - 避免重复API调用

3. **流式处理**
   - 边生成边合成视频
   - 减少等待时间

## API限制和处理

### 速率限制

- 千问API有QPS限制
- 图片生成有并发限制
- 实现了重试和退避机制

### 错误处理

所有API调用都包装在 `Result<T, VideoError>` 中：

```rust
pub enum VideoError {
    ApiError(String),
    SceneError(String),
    VideoGenerationError(String),
    IoError(std::io::Error),
    HttpError(reqwest::Error),
    JsonError(serde_json::Error),
}
```

## 测试

### 单元测试

```bash
cargo test
```

### 集成测试

```bash
# 测试完整流程
./target/release/auto-video \
  --text "简短的测试文本" \
  --output test.mp4 \
  --api-key $DASHSCOPE_API_KEY
```

### 性能测试

```bash
time ./target/release/auto-video --file long_text.txt
```

## 发布

### 构建发布版本

```bash
cargo build --release
strip target/release/auto-video  # 减小二进制大小
```

### 打包

```bash
# 创建发布包
tar -czf auto-video-v0.1.0-macos.tar.gz \
  -C target/release auto-video \
  -C ../.. README.md LICENSE

## 贡献指南

1. Fork项目
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启Pull Request

### 代码规范

- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 添加必要的注释和文档
- 编写测试用例

## 常见问题

### Q: 如何更换其他AI服务？

A: 在 `src/api/` 下创建新的客户端，实现相同的接口即可。

### Q: 如何自定义视频效果？

A: 修改 `src/video/generator.rs` 中的FFmpeg参数，添加自定义滤镜。

### Q: 如何提高生成速度？

A: 
1. 使用更快的AI模型
2. 并行处理图片生成
3. 优化FFmpeg参数
4. 使用SSD存储临时文件

### Q: 支持哪些视频格式？

A: 当前输出MP4格式，可通过修改FFmpeg参数支持其他格式。

## 路线图

- [ ] 支持更多AI服务（OpenAI, Midjourney等）
- [ ] Web界面
- [ ] 批量处理
- [ ] 模板系统
- [ ] 音乐背景
- [ ] 转场效果
- [ ] 多语言字幕
- [ ] 云端部署
