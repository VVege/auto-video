#!/bin/bash

# 测试脚本 - 使用短文本快速测试

echo "🎬 Auto-Video 测试脚本"
echo "======================="
echo ""

# 检查环境
if [ -z "$DASHSCOPE_API_KEY" ]; then
    echo "❌ 错误: DASHSCOPE_API_KEY 环境变量未设置"
    echo "请运行: export DASHSCOPE_API_KEY=your_key"
    exit 1
fi

echo "✅ API Key 已设置"

# 检查 FFmpeg
if ! command -v ffmpeg &> /dev/null; then
    echo "❌ 错误: FFmpeg 未安装"
    echo "请运行: brew install ffmpeg"
    exit 1
fi

echo "✅ FFmpeg 已安装"
echo ""

# 创建测试文本
TEST_TEXT="春天来了，万物复苏。花儿竞相开放，美丽极了。"

echo "📝 测试文本: $TEST_TEXT"
echo ""

# 运行测试
echo "🚀 开始生成视频..."
echo ""

./target/release/auto-video \
    --text "$TEST_TEXT" \
    --output test-output.mp4 \
    --work-dir ./test-temp

EXIT_CODE=$?

echo ""
if [ $EXIT_CODE -eq 0 ]; then
    echo "✅ 视频生成成功！"
    echo "📹 输出文件: test-output.mp4"
    
    if [ -f "test-output.mp4" ]; then
        SIZE=$(ls -lh test-output.mp4 | awk '{print $5}')
        echo "📊 文件大小: $SIZE"
        echo ""
        echo "🎥 播放视频:"
        echo "   macOS: open test-output.mp4"
        echo "   Linux: xdg-open test-output.mp4"
    fi
else
    echo "❌ 视频生成失败 (退出码: $EXIT_CODE)"
    echo ""
    echo "请查看上方错误信息"
fi

echo ""
echo "🗑️  清理临时文件"
echo "   rm -rf test-temp"
echo "   rm -f test-output.mp4"
