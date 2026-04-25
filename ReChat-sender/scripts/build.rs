#!/usr/bin/env rustc

use std::process::Command;
use std::env;

fn main() {
    println!("Building ReChat-sender...");

    // 构建项目
    let build_result = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .status()
        .expect("Failed to build project");

    if !build_result.success() {
        panic!("Build failed with exit code: {:?}", build_result.code());
    }

    println!("Build successful!");

    // 复制可执行文件到指定目录
    let target_dir = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
    let exe_path = format!("{}/release/rechat-sender", target_dir);
    let output_dir = "build";

    // 创建输出目录
    std::fs::create_dir_all(output_dir).expect("Failed to create output directory");

    // 复制可执行文件
    std::fs::copy(exe_path, format!("{}/rechat-sender", output_dir))
        .expect("Failed to copy executable");

    println!("Executable copied to {}/rechat-sender", output_dir);
}
