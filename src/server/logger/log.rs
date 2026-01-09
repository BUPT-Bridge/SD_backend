use std::env;
use std::fs;
use std::path::Path;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    EnvFilter,
    fmt::{self},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

/// 初始化 tracing 日志系统
/// 根据 Rust 构建模式自动判断：
/// - Debug 构建 (cargo build): DEBUG 及以上级别
/// - Release 构建 (cargo build --release): INFO 及以上级别
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    // 使用编译时标志判断模式
    let (level_filter, mode_name) = if cfg!(debug_assertions) {
        ("DEBUG", "Debug")
    } else {
        ("INFO", "Release")
    };

    println!("✓ 运行模式: {} 构建", mode_name);
    println!("✓ 日志级别: {}", level_filter);

    // 创建日志目录
    let log_path = env::var("LOG_PATH").unwrap_or_else(|_| "logs".to_string());
    if !Path::new(&log_path).exists() {
        fs::create_dir_all(&log_path)?;
    }

    // 创建文件 appender（每天滚动）
    let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_path, "app.log");

    // 创建环境过滤器
    // 优先使用 RUST_LOG 环境变量，否则使用构建模式决定的级别
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level_filter));

    // 控制台输出层（带颜色）
    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_target(false)
        .with_level(true)
        .compact();

    // 文件输出层（无颜色）
    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(false)
        .with_level(true)
        .compact();

    // 组合所有层
    tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
        .init();

    println!("✓ 日志输出: 控制台 + 文件 ({})", log_path);

    Ok(())
}

// 为了兼容其他模块的使用，提供便捷的宏

/// 记录 TRACE 级别日志
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        tracing::trace!($($arg)*)
    };
}

/// 记录 DEBUG 级别日志
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*)
    };
}

/// 记录 INFO 级别日志（默认）
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*)
    };
}

/// 记录 WARN 级别日志
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*)
    };
}

/// 记录 ERROR 级别日志
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*)
    };
}
