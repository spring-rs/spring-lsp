#!/bin/bash

# 清理构建产物
echo "Cleaning build artifacts..."

# 删除编译输出
rm -rf out
rm -rf dist

# 删除打包文件
rm -f *.vsix

echo "Clean complete!"
