#!/bin/bash

# PF8 Android JNI 构建脚本

set -e

echo "开始构建 PF8 Android JNI 库..."

# 检查是否安装了 cargo-ndk
if ! command -v cargo-ndk &>/dev/null; then
  echo "错误: cargo-ndk 未安装"
  echo "请运行: cargo install cargo-ndk"
  exit 1
fi

# 进入 rust 目录
cd "$(dirname "$0")/rust"

# 定义目标架构
TARGETS=(
  "aarch64-linux-android" # arm64-v8a
  # "armv7-linux-androideabi"  # armeabi-v7a
  # "i686-linux-android"       # x86
  "x86_64-linux-android" # x86_64
)

# 检查是否安装了目标
echo "检查 Rust 目标..."
for target in "${TARGETS[@]}"; do
  if ! rustup target list | grep -q "$target (installed)"; then
    echo "安装目标: $target"
    rustup target add "$target"
  fi
done

# 构建类型 (debug 或 release)
BUILD_TYPE="${1:-release}"

if [ "$BUILD_TYPE" = "release" ]; then
  BUILD_FLAG="--release"
  BUILD_DIR="release"
  echo "构建模式: Release"
else
  BUILD_FLAG=""
  BUILD_DIR="debug"
  echo "构建模式: Debug"
fi

# 输出目录
OUTPUT_DIR="../app/src/main/jniLibs"

# 清理旧的库文件
echo "清理旧的库文件..."
rm -rf "$OUTPUT_DIR"

# 构建所有目标
echo "开始构建..."
cargo ndk \
  -t arm64-v8a \
  -t x86_64 \
  -o "$OUTPUT_DIR" \
  build $BUILD_FLAG
# -t armeabi-v7a \
# -t x86 \

echo "✅ 构建完成!"
echo ""
echo "库文件已生成到:"
find "$OUTPUT_DIR" -name "*.so" -type f

echo ""
echo "文件大小:"
du -h "$OUTPUT_DIR"/*/*.so

echo ""
echo "现在可以在 Android Studio 中构建和运行应用了。"
