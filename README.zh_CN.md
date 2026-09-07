# qubit-validation-rules

[![Rust CI](https://github.com/qubit-ltd/rs-case/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-case/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-case/coverage-badge.json)](https://qubit-ltd.github.io/rs-case/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-case.svg?color=blue)](https://crates.io/crates/qubit-case)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

面向 Qubit Rust 服务的类型化校验规则库，并可选提供
`qubit-validator` inventory 注册适配器。

## 概述

Qubit Case 用于在 Java、C++、Python、XML、JSON 和 Rust 工具链常见的 ASCII 标识符命名风格之间转换，提供紧凑的 Rust 枚举 API。

## 设计目标

- **小型 API 表面**：公开一个核心 `CaseStyle` 枚举、五个便捷常量，以及职责明确的解析和校验错误类型。
- **一致行为**：对受支持的 ASCII 标识符转换保持稳定，兼容常见命名风格。
- **严格匹配**：检测字符串是否符合某种命名风格。
- **尽力转换**：即使输入并不严格符合源风格，也会尽量给出转换结果。
- **无运行时依赖**：保持库轻量。

## 支持的风格

- `LOWER_HYPHEN`: `lower-hyphen`
- `LOWER_UNDERSCORE`: `lower_underscore`
- `LOWER_CAMEL`: `lowerCamel`
- `UPPER_CAMEL`: `UpperCamel`
- `UPPER_UNDERSCORE`: `UPPER_UNDERSCORE`

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
qubit-case = "0.2"
```

## 快速开始

```rust
use qubit_case::{
    LOWER_CAMEL,
    LOWER_HYPHEN,
    UPPER_UNDERSCORE,
};

let value = LOWER_HYPHEN.to(LOWER_CAMEL, "hello-world");
assert_eq!(value, "helloWorld");

LOWER_CAMEL
    .validate("http2Client")
    .expect("该字符串应符合小驼峰命名风格");

let constant = LOWER_CAMEL
    .checked_to(UPPER_UNDERSCORE, "http2Client")
    .expect("源字符串应符合指定命名风格");
assert_eq!(constant, "HTTP_2_CLIENT");

assert!(UPPER_UNDERSCORE.matches("HTTP_2_CLIENT"));
assert_eq!(LOWER_CAMEL.to_string(), "lower-camel");
```

## API 参考

### 类型

- [`CaseStyle`](https://docs.rs/qubit-case/latest/qubit_case/enum.CaseStyle.html) -
  支持的命名风格和转换方法。
- [`CaseStyleError`](https://docs.rs/qubit-case/latest/qubit_case/struct.CaseStyleError.html) -
  解析未知风格名称时返回的错误。
- [`CaseStyleValidationError`](https://docs.rs/qubit-case/latest/qubit_case/struct.CaseStyleValidationError.html) -
  字符串不符合预期命名风格时返回的错误。

### 常量

- `LOWER_HYPHEN`
- `LOWER_UNDERSCORE`
- `LOWER_CAMEL`
- `UPPER_CAMEL`
- `UPPER_UNDERSCORE`

### 方法

- `CaseStyle::values()` 按参考实现顺序返回全部风格。
- `CaseStyle::of(name)` 解析风格名称，忽略大小写，并把连字符和下划线视为等价。
- `CaseStyle::name()` 返回规范的小写连字符风格名称。
- `CaseStyle::word_separator()` 返回该风格使用的单词分隔符。
- `CaseStyle::to(target, value)` 执行宽松的尽力转换；它不校验源字符串，也不保证非法输入符合目标风格。
- `CaseStyle::validate(value)` 校验标识符，不符合该风格时返回错误。
- `CaseStyle::checked_to(target, value)` 校验源字符串后再转换。
- `CaseStyle::matches(value)` 检查标识符是否严格符合该风格。

### Trait 实现

- `Display` 使用规范的小写连字符名称格式化命名风格。
- `FromStr` 解析 `CaseStyle::of` 所接受的风格名称。

## 测试

```bash
# 使用默认 feature 集运行测试
cargo test

# 使用项目声明的全部 feature 运行测试
cargo test --all-features

# 运行项目 CI 检查
./ci-check.sh

# 检查代码覆盖率
./coverage.sh
```

## 许可证

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

本项目基于 Apache License 2.0 授权。完整许可证文本请参阅
[LICENSE](LICENSE)。

## 贡献

欢迎贡献。请遵循 Rust API 指南，及时更新公共 API 文档与测试，并在提交
Pull Request 前运行 `./align-ci.sh`格式化代码，运行`./ci-check.sh`对齐CI要求。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-case](https://github.com/qubit-ltd/rs-case)
