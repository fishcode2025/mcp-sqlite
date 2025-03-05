# 发布指南

本文档描述了如何发布新版本的SQLite MCP服务器。

## 发布流程

1. 更新版本号

   在`Cargo.toml`文件中更新版本号：

   ```toml
   [package]
   name = "mcp-sqlite"
   version = "x.y.z"  # 更新此处的版本号
   ```

2. 更新CHANGELOG.md（如果存在）

   记录自上一个版本以来的所有更改。

3. 提交更改

   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "准备发布 vx.y.z"
   ```

4. 创建标签

   ```bash
   git tag -a vx.y.z -m "版本 x.y.z"
   ```

5. 推送更改和标签

   ```bash
   git push origin main
   git push origin vx.y.z
   ```

6. 等待GitHub Actions完成

   推送标签后，GitHub Actions将自动执行以下操作：

   - 构建和测试代码
   - 为不同平台创建二进制文件
   - 创建GitHub Release
   - 发布到crates.io
   - 更新文档
   - 更新README中的徽章

7. 验证发布

   - 检查[GitHub Releases](https://github.com/fishcode2025/mcp-sqlite/releases)页面
   - 检查[crates.io](https://crates.io/crates/mcp-sqlite)页面
   - 检查[docs.rs](https://docs.rs/mcp-sqlite)页面

## GitHub Actions工作流

本项目使用以下GitHub Actions工作流：

### 1. Rust CI/CD (rust.yml)

- 在多个平台上构建和测试代码
- 创建发布版本的二进制文件
- 在标签推送时创建GitHub Release
- 在标签推送时发布到crates.io

### 2. 更新状态徽章 (badges.yml)

- 在CI/CD工作流完成后更新README中的徽章
- 显示构建状态、最新版本、crates.io版本等信息

### 3. 文档部署 (docs.yml)

- 生成项目文档
- 部署文档到GitHub Pages

## 注意事项

- 确保在发布前所有测试都通过
- 版本号应遵循[语义化版本](https://semver.org/)规范
- 发布到crates.io需要设置`CRATES_IO_TOKEN`密钥
- 更新README徽章需要设置适当的GitHub权限

## 故障排除

如果发布过程中遇到问题，请检查：

1. GitHub Actions日志以获取详细错误信息
2. 确保所有必要的密钥都已设置
3. 确保您有足够的权限推送到仓库和创建Release 