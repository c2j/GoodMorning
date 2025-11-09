# Playwright 支持恢复指南

## 概述

由于当前环境的权限限制，阶段一实施时暂时注释了 Playwright 相关代码。本文档说明如何在生产环境中恢复完整的 Playwright 支持。

## 恢复步骤

### 步骤 1: 恢复 Cargo.toml 配置

在 `/app1/Cargo.toml` 中：

```toml
# 恢复依赖声明（第88-89行）
playwright = { git = "https://github.com/c2j/playwright-rust.git", branch = "Playwright-1.50+", optional = true }

# 恢复特性标志（第22行和第32行）
[features]
default = ["rustls", "playwright"]  # 添加 "playwright"
# ...
playwright = ["dep:playwright"]  # 取消注释
```

### 步骤 2: 恢复 lib.rs 中的模块导入

在 `/app1/src/lib.rs` 中：

```rust
// 取消注释第45-47行
#[cfg(feature = "playwright")]
mod page_test;

// 取消注释第57-59行
#[cfg(feature = "playwright")]
use crate::page_test::cli::PwArgs;
```

### 步骤 3: 恢复 Opts 结构体

在 `/app1/src/lib.rs` 中，Opts 结构体（第68-75行）：

```rust
pub struct Opts {
    #[arg(help = "Target URL or file with multiple URLs.")]
    url: Option<String>,  // 修改为 Option<String>

    #[cfg(feature = "playwright")]
    #[command(flatten)]
    pw_args: Option<PwArgs>,
    // ...
}
```

### 步骤 4: 恢复 run() 函数中的检查

在 `/app1/src/lib.rs` 中，run() 函数（第338-358行）：

```rust
pub async fn run(mut opts: Opts) -> anyhow::Result<()> {
    // 取消注释整个 Playwright 检查块
    #[cfg(feature = "playwright")]
    {
        if let Some(pw_args) = &opts.pw_args {
            if pw_args.record {
                if let Some(url) = &opts.url {
                    return crate::page_test::run_record_mode(pw_args.clone(), url.clone()).await;
                } else {
                    anyhow::bail!("URL is required for recording mode");
                }
            } else if pw_args.replay || pw_args.scenario_mode {
                if let Some(scenario) = &pw_args.scenario {
                    return crate::page_test::run_replay_mode(pw_args.clone(), scenario.clone()).await;
                } else {
                    anyhow::bail!("Scenario file is required for replay mode");
                }
            }
        }
    }
    // ...
}
```

### 步骤 5: 恢复 URL 处理逻辑

在 `/app1/src/lib.rs` 中（第408-436行）：

```rust
// Get URL for non-Playwright mode
#[cfg(feature = "playwright")]
let url = opts.url.expect("URL is required for non-Playwright mode");
#[cfg(not(feature = "playwright"))]
let url = opts.url;

// 后续代码使用 url 而不是 opts.url
```

## 完整修改清单

### 文件: Cargo.toml
```diff
  [features]
- default = ["rustls"]
+ default = ["rustls", "playwright"]
  # ...
- # playwright = ["dep:playwright"]  # disabled for testing
+ playwright = ["dep:playwright"]

  [dependencies]
  # ...
- # Playwright support (temporarily disabled for testing)
- # playwright = { git = "https://github.com/c2j/playwright-rust.git", branch = "Playwright-1.50+", optional = true }
+ # Playwright support
+ playwright = { git = "https://github.com/c2j/playwright-rust.git", branch = "Playwright-1.50+", optional = true }
```

### 文件: src/lib.rs
```diff
-mod page_test;
+#[cfg(feature = "playwright")]
+mod page_test;

-use crate::page_test::cli::PwArgs;
+// Temporarily comment out for testing
+// #[cfg(feature = "playwright")]
+// use crate::page_test::cli::PwArgs;
```

**其他修改**:
- 将 Opts.url 从 `String` 改为 `Option<String>`
- 取消注释 run() 函数中的 Playwright 检查块
- 恢复 URL 变量提取逻辑

## 安装依赖

### 1. 安装 Rust 依赖
```bash
# 清理缓存
cargo clean

# 更新依赖
cargo update

# 构建（启用 playwright 特性）
cargo build --release --features playwright
```

### 2. 安装 Playwright 浏览器
```bash
# 安装 Chromium（推荐）
playwright install chromium

# 或安装所有浏览器
playwright install

# 验证安装
playwright --version
```

### 3. 安装系统依赖（Linux）

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y libnss3 libatk-bridge2.0-0 libdrm2 libxkbcommon0 libxcomposite1 libxdamage1 libxfixes3 libxrandr2 libgbm1 libasound2

# CentOS/RHEL
sudo yum install -y nss atk GConf2 libXcomposite libXcursor libXdamage libXext libXi libXrandr libXScrnSaver libgbm pango alsa-lib

# Fedora
sudo dnf install -y nss atk GConf2 libXcomposite libXcursor libXdamage libXext libXi libXrandr libXScrnSaver libgbm pango alsa-lib
```

## 验证安装

### 1. 检查特性
```bash
cargo build --features playwright --no-default-features 2>&1 | grep -i playwright
```

### 2. 测试编译
```bash
cargo build --release --features playwright 2>&1 | tail -20
```

### 3. 测试运行
```bash
# 查看帮助信息
./target/release/oha --help

# 测试 CLI 参数
./target/release/oha --pw-browser chromium --help
```

## 常见问题

### Q1: 编译时提示 "cannot find playwright"
**A**: 确保已正确恢复 Cargo.toml 中的依赖声明，并运行 `cargo update`

### Q2: 运行时提示 "browser not found"
**A**: 运行 `playwright install chromium` 安装浏览器

### Q3: 权限错误 "Permission denied"
**A**: 确保 cargo 目录有正确的权限
```bash
chown -R $USER:$USER ~/.cargo
```

### Q4: 编译很慢
**A**: 首次编译需要下载和构建 Playwright，可能需要 5-10 分钟，后续会缓存

## 下一步

恢复 Playwright 支持后，可以继续阶段二的开发：

1. 实现真正的 Playwright API 调用
2. 添加网络监控功能
3. 实现录制功能
4. 生成 scenario.json

## 参考资源

- [Playwright Rust 文档](https://docs.rs/playwright/latest/playwright/)
- [Playwright 官方文档](https://playwright.dev/)
- [c2j/playwright-rust 仓库](https://github.com/c2j/playwright-rust/tree/Playwright-1.50%2B)

---

**最后更新**: 2025-11-09
**适用版本**: OHA v1.11.0+