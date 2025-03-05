#!/bin/bash
# 修复版本的脚本用于更新README.md中的徽章

# 获取仓库信息
REPO="fishcode2025/mcp-sqlite"
BRANCH="main"
PACKAGE_NAME="mcp-sqlite"

# 创建徽章链接 - 使用不同的分隔符来避免斜杠问题
BUILD_BADGE="[![构建状态](https://github.com/${REPO}/actions/workflows/rust.yml/badge.svg?branch=${BRANCH})](https://github.com/${REPO}/actions/workflows/rust.yml)"
VERSION_BADGE="[![版本](https://img.shields.io/github/v/release/${REPO}?include_prereleases)](https://github.com/${REPO}/releases/latest)"
CRATES_BADGE="[![Crates.io](https://img.shields.io/crates/v/${PACKAGE_NAME})](https://crates.io/crates/${PACKAGE_NAME})"
DOCS_BADGE="[![文档](https://docs.rs/${PACKAGE_NAME}/badge.svg)](https://docs.rs/${PACKAGE_NAME})"
LICENSE_BADGE="[![许可证](https://img.shields.io/badge/许可证-MIT-green)](https://github.com/${REPO}/blob/main/LICENSE)"

# 创建临时文件
TEMP_FILE=$(mktemp)

# 读取第一行
read -r FIRST_LINE < README.md

# 写入第一行到临时文件
echo "$FIRST_LINE" > "$TEMP_FILE"
echo "" >> "$TEMP_FILE"
echo "$BUILD_BADGE $VERSION_BADGE $CRATES_BADGE $DOCS_BADGE $LICENSE_BADGE" >> "$TEMP_FILE"

# 跳过原始文件中的徽章行
sed -n '1d;/^$/,$p' README.md | sed '1d' >> "$TEMP_FILE"

# 替换原始文件
mv "$TEMP_FILE" README.md

echo "README.md徽章已更新"

# 如果在Git环境中，可以取消注释以下行来提交更改
# git add README.md
# git commit -m "更新README徽章 [skip ci]"
# git push 