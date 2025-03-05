# PowerShell脚本用于更新README.md中的徽章

# 获取仓库信息
$REPO = "fishcode2025/mcp-sqlite"
$BRANCH = "main"
$PACKAGE_NAME = "mcp-sqlite"

# 创建徽章链接
$BUILD_BADGE = "[![构建状态](https://github.com/$REPO/actions/workflows/rust.yml/badge.svg?branch=$BRANCH)](https://github.com/$REPO/actions/workflows/rust.yml)"
$VERSION_BADGE = "[![版本](https://img.shields.io/github/v/release/$REPO?include_prereleases)](https://github.com/$REPO/releases/latest)"
$CRATES_BADGE = "[![Crates.io](https://img.shields.io/crates/v/$PACKAGE_NAME)](https://crates.io/crates/$PACKAGE_NAME)"
$DOCS_BADGE = "[![文档](https://docs.rs/$PACKAGE_NAME/badge.svg)](https://docs.rs/$PACKAGE_NAME)"
$LICENSE_BADGE = "[![许可证](https://img.shields.io/badge/许可证-MIT-green)](https://github.com/$REPO/blob/main/LICENSE)"

# 读取README.md文件
$readmeContent = Get-Content -Path "README.md" -Raw

# 创建新的徽章行
$badgeLine = "$BUILD_BADGE $VERSION_BADGE $CRATES_BADGE $DOCS_BADGE $LICENSE_BADGE"

# 使用正则表达式替换第一行后的徽章行
$newContent = $readmeContent -replace "(?<=# SQLite MCP服务器\r?\n\r?\n).*?(?=\r?\n\r?\n)", $badgeLine

# 写回文件
Set-Content -Path "README.md" -Value $newContent

Write-Host "README.md徽章已更新"

# 如果在Git环境中，可以取消注释以下行来提交更改
# git add README.md
# git commit -m "更新README徽章 [skip ci]"
# git push 