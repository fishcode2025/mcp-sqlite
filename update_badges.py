#!/usr/bin/env python3
# Python脚本用于更新README.md中的徽章

import re
import os

# 设置仓库信息
REPO = "fishcode2025/mcp-sqlite"
BRANCH = "main"
PACKAGE_NAME = "mcp-sqlite"

# 创建徽章链接
BUILD_BADGE = f"[![构建状态](https://github.com/{REPO}/actions/workflows/rust.yml/badge.svg?branch={BRANCH})](https://github.com/{REPO}/actions/workflows/rust.yml)"
VERSION_BADGE = f"[![版本](https://img.shields.io/github/v/release/{REPO}?include_prereleases)](https://github.com/{REPO}/releases/latest)"
CRATES_BADGE = f"[![Crates.io](https://img.shields.io/crates/v/{PACKAGE_NAME})](https://crates.io/crates/{PACKAGE_NAME})"
DOCS_BADGE = f"[![文档](https://docs.rs/{PACKAGE_NAME}/badge.svg)](https://docs.rs/{PACKAGE_NAME})"
LICENSE_BADGE = f"[![许可证](https://img.shields.io/badge/许可证-MIT-green)](https://github.com/{REPO}/blob/main/LICENSE)"

# 组合所有徽章
BADGES = f"{BUILD_BADGE} {VERSION_BADGE} {CRATES_BADGE} {DOCS_BADGE} {LICENSE_BADGE}"

# 读取README.md文件
with open("README.md", "r", encoding="utf-8") as f:
    content = f.read()

# 使用正则表达式替换徽章行
pattern = r"(# SQLite MCP服务器\n\n)(\[!\[.*?\n\n)"
replacement = f"\\1{BADGES}\n\n"
new_content = re.sub(pattern, replacement, content, flags=re.DOTALL)

# 写回README.md文件
with open("README.md", "w", encoding="utf-8") as f:
    f.write(new_content)

print("README.md徽章已更新")

# 如果在Git环境中，可以取消注释以下行来提交更改
# os.system("git add README.md")
# os.system("git commit -m \"更新README徽章 [skip ci]\"")
# os.system("git push") 