#!/bin/bash
# WASM 插件构建脚本

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
OUTPUT_DIR="$PROJECT_ROOT/plugins/wasm"
WASM_FILE="$PROJECT_ROOT/target/wasm32-unknown-unknown/release/wasm_plugin.wasm"

echo "🔨 构建 WASM 插件..."

# 创建输出目录
mkdir -p "$OUTPUT_DIR"

# 进入插件目录
cd "$SCRIPT_DIR"

# 清理旧的构建
echo "🧹 清理旧的 WASM 文件..."
rm -f "$WASM_FILE" 2>/dev/null || true

# 使用 cargo 直接构建
echo "📦 编译 WASM..."
cargo build --release --lib --target wasm32-unknown-unknown

# 检查 WASM 文件是否生成
if [ ! -f "$WASM_FILE" ]; then
    echo "❌ 错误: WASM 文件未生成"
    echo "📁 查找路径: $WASM_FILE"
    exit 1
fi

# 使用 wasm-bindgen 生成绑定
echo "🔗 生成 JavaScript 绑定..."
wasm-bindgen \
    --target web \
    --out-dir "$OUTPUT_DIR" \
    --out-name wasm_plugin \
    "$WASM_FILE"

# 复制 WASM 文件到输出目录（作为备份）
cp "$WASM_FILE" "$OUTPUT_DIR/wasm_plugin_bg.wasm"

echo ""
echo "✅ 构建完成！"
echo "📁 输出文件:"
ls -lh "$OUTPUT_DIR"/*.wasm "$OUTPUT_DIR"/*.js 2>/dev/null

echo ""
echo "📊 WASM 文件大小:"
du -h "$OUTPUT_DIR"/*.wasm

echo ""
echo "🎉 WASM 插件已准备好，可以在应用中加载！"
