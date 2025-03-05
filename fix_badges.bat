@echo off
setlocal enabledelayedexpansion
REM Windows批处理文件用于更新README.md中的徽章

REM 设置仓库信息
set REPO=fishcode2025/mcp-sqlite
set BRANCH=main
set PACKAGE_NAME=mcp-sqlite

REM 创建临时文件
set TEMP_FILE=%TEMP%\readme_temp.md

REM 读取第一行并写入临时文件
for /f "delims=" %%a in (README.md) do (
    echo %%a > %TEMP_FILE%
    goto :continue
)

:continue
echo. >> %TEMP_FILE%
echo [![构建状态](https://github.com/%REPO%/actions/workflows/rust.yml/badge.svg?branch=%BRANCH%)](https://github.com/%REPO%/actions/workflows/rust.yml) [![版本](https://img.shields.io/github/v/release/%REPO%?include_prereleases)](https://github.com/%REPO%/releases/latest) [![Crates.io](https://img.shields.io/crates/v/%PACKAGE_NAME%)](https://crates.io/crates/%PACKAGE_NAME%) [![文档](https://docs.rs/%PACKAGE_NAME%/badge.svg)](https://docs.rs/%PACKAGE_NAME%) [![许可证](https://img.shields.io/badge/许可证-MIT-green)](https://github.com/%REPO%/blob/main/LICENSE) >> %TEMP_FILE%

REM 跳过原始文件中的徽章行并添加剩余内容
set SKIP_LINES=3
set LINE_COUNT=0
for /f "skip=1 delims=" %%a in (README.md) do (
    set /a LINE_COUNT+=1
    if !LINE_COUNT! GTR %SKIP_LINES% (
        echo %%a >> %TEMP_FILE%
    )
)

REM 替换原始文件
copy /y %TEMP_FILE% README.md > nul

echo README.md徽章已更新

REM 如果在Git环境中，可以取消注释以下行来提交更改
REM git add README.md
REM git commit -m "更新README徽章 [skip ci]"
REM git push 