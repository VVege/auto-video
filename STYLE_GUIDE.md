# 🎨 图片风格配置说明

## 可用的图片风格

根据千问万相 API，支持以下图片风格（必须使用尖括号格式）：

| 风格代码 | 说明 | 适用场景 |
|---------|------|---------|
| `<photography>` | 摄影风格 | ✅ 默认，真实感照片 |
| `<portrait>` | 人像风格 | 人物特写、肖像 |
| `<3d cartoon>` | 3D卡通 | 卡通人物、动画场景 |
| `<anime>` | 动漫风格 | 日系动漫、二次元 |
| `<oil painting>` | 油画风格 | 艺术画作、经典油画 |
| `<watercolor>` | 水彩画 | 柔和画面、水彩艺术 |
| `<sketch>` | 素描风格 | 黑白素描、线条艺术 |
| `<chinese painting>` | 国画风格 | 中国传统绘画 |
| `<flat illustration>` | 扁平插画 | 现代简约、扁平设计 |
| `<auto>` | 自动选择 | 让 AI 自动选择最佳风格 |

## 当前配置

**默认风格**: `<photography>` (摄影风格)

这是最适合一般视频内容的风格，生成的图片真实感强，适合大多数场景。

## 如何自定义风格

### 方法 1: 修改代码中的默认风格

编辑 `src/api/qwen.rs` 第 180 行左右：

```rust
"parameters": {
    "style": "<photography>",  // 修改这里
    "size": "1280*720",
    "n": 1
}
```

可选值：
- `<photography>` - 摄影（推荐）
- `<anime>` - 动漫
- `<3d cartoon>` - 3D 卡通
- `<oil painting>` - 油画
- `<watercolor>` - 水彩
- 等等...

### 方法 2: 添加命令行参数支持（未来功能）

```bash
# 未来可能的用法
./auto-video --text "..." --style anime --output video.mp4
```

## 示例效果

### 摄影风格 (当前默认)
```bash
./target/release/auto-video \
    --text "美丽的日落，金色的天空" \
    --output sunset.mp4
```
生成效果：真实的摄影照片风格

### 动漫风格
修改代码中的 style 为 `<anime>`，然后：
```bash
./target/release/auto-video \
    --text "可爱的小猫在玩耍" \
    --output cat-anime.mp4
```
生成效果：日系动漫风格

### 油画风格
修改代码中的 style 为 `<oil painting>`，然后：
```bash
./target/release/auto-video \
    --text "宁静的乡村风景" \
    --output village.mp4
```
生成效果：经典油画风格

## 注意事项

### ⚠️ 格式要求

1. **必须使用尖括号**: `<photography>` ✅  而不是 `photography` ❌
2. **大小写敏感**: 必须使用小写
3. **空格处理**: `<3d cartoon>` 中间有空格是正确的

### 💡 风格选择建议

| 视频类型 | 推荐风格 |
|---------|---------|
| 新闻报道、纪录片 | `<photography>` |
| 儿童故事 | `<3d cartoon>` 或 `<anime>` |
| 艺术展示 | `<oil painting>` 或 `<watercolor>` |
| 产品介绍 | `<photography>` |
| 动画短片 | `<anime>` 或 `<flat illustration>` |
| 教育内容 | `<flat illustration>` 或 `<photography>` |

## 常见错误

### ❌ 错误 1: 缺少尖括号
```json
"style": "photography"  // 错误！
```
**错误信息**: `Value error, style photography not in ['<photography>', ...]`

**正确方式**:
```json
"style": "<photography>"  // 正确！
```

### ❌ 错误 2: 使用了不支持的风格
```json
"style": "<realistic>"  // 不在支持列表中
```

**解决方法**: 使用支持列表中的风格

### ❌ 错误 3: 大小写错误
```json
"style": "<Photography>"  // 大写 P
```

**正确方式**:
```json
"style": "<photography>"  // 全小写
```

## 批量测试不同风格

创建测试脚本 `test-styles.sh`:

```bash
#!/bin/bash

STYLES=("<photography>" "<anime>" "<oil painting>" "<watercolor>")
TEXT="美丽的春天，花儿盛开"

for style in "${STYLES[@]}"; do
    # 修改代码中的风格
    sed -i '' "s/\"style\": \"<[^>]*>\"/\"style\": \"$style\"/" src/api/qwen.rs
    
    # 重新编译
    cargo build --release
    
    # 生成视频
    ./target/release/auto-video \
        --text "$TEXT" \
        --output "spring_${style//[<> ]/_}.mp4"
done
```

## 高级配置

### 其他可配置参数

除了 style，还可以配置：

```rust
"parameters": {
    "style": "<photography>",
    "size": "1280*720",      // 图片尺寸
    "n": 1,                  // 生成数量
    // 可能的其他参数:
    // "seed": 42,           // 随机种子（可复现）
    // "ref_img": "url",     // 参考图片
}
```

### 支持的图片尺寸

- `1024*1024` - 正方形
- `720*1280` - 竖屏
- `1280*720` - 横屏（当前默认，适合视频）
- `1920*1080` - 全高清

## 快速修复指南

如果遇到风格相关错误：

1. **检查格式**: 确保使用 `<photography>` 格式
2. **检查拼写**: 对照支持列表检查拼写
3. **重新编译**: 修改后必须重新编译
4. **查看日志**: 错误信息会显示支持的风格列表

---

**更新时间**: 2025-12-28  
**当前版本**: v0.1.2  
**默认风格**: `<photography>`
