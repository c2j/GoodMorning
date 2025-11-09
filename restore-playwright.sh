#!/bin/bash
# Playwright 支持恢复脚本
# 用法: bash restore-playwright.sh

set -e

echo "========================================="
echo "  OHA Playwright 支持恢复脚本"
echo "========================================="
echo ""

# 检查当前目录
if [ ! -f "Cargo.toml" ]; then
    echo "错误: 请在项目根目录执行此脚本"
    exit 1
fi

echo "步骤 1: 恢复 Cargo.toml 配置..."

# 恢复依赖声明
sed -i 's|# Playwright support (temporarily disabled for testing)|# Playwright support|' Cargo.toml
sed -i 's|# playwright = { git = "https://github.com/c2j/playwright-rust.git", branch = "Playwright-1.50+", optional = true }|# Playwright support|' Cargo.toml
sed -i 's|# playwright = { git = "https://github.com/c2j/playwright-rust.git", branch = "Playwright-1.50+", optional = true }|playwright = { git = "https://github.com/c2j/playwright-rust.git", branch = "Playwright-1.50+", optional = true }|' Cargo.toml

# 恢复特性标志
sed -i 's|default = \["rustls"\]|default = ["rustls", "playwright"]|' Cargo.toml
sed -i 's|# playwright = \["dep:playwright"\]  # disabled for testing|playwright = ["dep:playwright"]|' Cargo.toml
sed -i 's|# playwright = \["dep:playwright"\]  # disabled for testing|playwright = ["dep:playwright"]|' Cargo.toml

echo "✅ Cargo.toml 配置已恢复"
echo ""

echo "步骤 2: 恢复 src/lib.rs 配置..."

# 创建临时文件进行替换
cat > /tmp/restore-lib.rs << 'EOF'
# 恢复模块导入
sed -i 's|// Temporarily comment out page_test module for testing|#[cfg(feature = "playwright")]|' src/lib.rs
sed -i 's|// #\[cfg(feature = "playwright")\]|# [cfg(feature = "playwright")]|' src/lib.rs
sed -i 's|// mod page_test;|mod page_test;|' src/lib.rs
sed -i 's|// #\[cfg(feature = "playwright")\]|# [cfg(feature = "playwright")]|' src/lib.rs
sed -i 's|// mod page_test;|mod page_test;|' src/lib.rs

# 恢复 use 语句
sed -i 's|// Temporarily comment out for testing|#[cfg(feature = "playwright")]|' src/lib.rs
sed -i 's|// #\[cfg(feature = "playwright")\]|# [cfg(feature = "playwright")]|' src/lib.rs
sed -i 's|// use crate::page_test::cli::PwArgs;|use crate::page_test::cli::PwArgs;|' src/lib.rs

# 恢复 Opts 结构体
sed -i 's|    url: String,|    url: Option<String>,|' src/lib.rs
sed -i 's|    // Temporarily comment out for testing|    #[cfg(feature = "playwright")]|' src/lib.rs
sed -i 's|    // #\[cfg(feature = "playwright")\]|# [cfg(feature = "playwright")]|' src/lib.rs
sed -i 's|    // #\[command(flatten)\\]|# [command(flatten)]|' src/lib.rs
sed -i 's|    // pw_args: Option<PwArgs>,|    pw_args: Option<PwArgs>,|' src/lib.rs

# 恢复 run() 函数中的检查
sed -i 's|    // Check if Playwright mode is enabled|    // Check if Playwright mode is enabled|' src/lib.rs
sed -i 's|    // #\[cfg(feature = "playwright")\]|# [cfg(feature = "playwright")]|' src/lib.rs
sed -i 's|    // {|    {|' src/lib.rs
sed -i 's|    //     if let Some(pw_args) = &opts.pw_args {|    if let Some(pw_args) = &opts.pw_args {|' src/lib.rs
sed -i 's|    //         // Ensure we have either a URL or scenario file|        // Ensure we have either a URL or scenario file|' src/lib.rs
sed -i 's|    //         if pw_args.record {|        if pw_args.record {|' src/lib.rs
sed -i 's|    //             if let Some(url) = &opts.url {|            if let Some(url) = &opts.url {|' src/lib.rs
sed -i 's|    //                 return crate::page_test::run_record_mode|                return crate::page_test::run_record_mode|' src/lib.rs
sed -i 's|    //             } else {|            } else {|}' src/lib.rs
sed -i 's|    //                 anyhow::bail!("URL is required for recording mode");|                anyhow::bail!("URL is required for recording mode");|' src/lib.rs
sed -i 's|    //             }|            }|' src/lib.rs
sed -i 's|    //         } else if pw_args.replay || pw_args.scenario_mode {|        } else if pw_args.replay || pw_args.scenario_mode {|}' src/lib.rs
sed -i 's|    //             if let Some(scenario) = &pw_args.scenario {|            if let Some(scenario) = &pw_args.scenario {|'' src/lib.rs
sed -i 's|    //                 return crate::page_test::run_replay_mode|                return crate::page_test::run_replay_mode|' src/lib.rs
sed -i 's|    //             } else {|            } else {|}' src/lib.rs
sed -i 's|    //                 anyhow::bail!("Scenario file is required for replay mode");|                anyhow::bail!("Scenario file is required for replay mode");|' src/lib.rs
sed -i 's|    //             }|            }|' src/lib.rs
sed -i 's|    //         }|        }|' src/lib.rs
sed -i 's|    //     }|    }|' src/lib.rs
sed -i 's|    // }|    }|' src/lib.rs

# 恢复 URL 处理
sed -i 's|    let url_generator = if opts.rand_regex_url {|    // Get URL for non-Playwright mode|' src/lib.rs
sed -i '1i\    #[cfg(feature = "playwright")]\n    let url = opts.url.expect("URL is required for non-Playwright mode");\n    #[cfg(not(feature = "playwright"))]\n    let url = opts.url;\n\n    let url_generator = if opts.rand_regex_url {' src/lib.rs
sed -i 's|        let dot_disabled: String = opts|        let dot_disabled: String = url|' src/lib.rs
sed -i 's|        let path = Path::new(&opts.url);|        let path = Path::new(\&url);|' src/lib.rs
sed -i 's|        UrlGenerator::new_static(Url::parse(&opts.url)?)|        UrlGenerator::new_static(Url::parse(\&url)?)|' src/lib.rs
EOF

bash /tmp/restore-lib.rs
rm /tmp/restore-lib.rs

echo "✅ src/lib.rs 配置已恢复"
echo ""

echo "========================================="
echo "  恢复完成!"
echo "========================================="
echo ""
echo "下一步操作:"
echo "  1. 安装 Playwright 浏览器:"
echo "     playwright install chromium"
echo ""
echo "  2. 构建项目:"
echo "     cargo build --release --features playwright"
echo ""
echo "  3. 测试:"
echo "     ./target/release/oha --help"
echo ""
