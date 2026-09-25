# qubit-validation-rules

[![Rust CI](https://github.com/qubit-ltd/rs-validation-rules/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-validation-rules/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-validation-rules/coverage-badge.json)](https://qubit-ltd.github.io/rs-validation-rules/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-validation-rules.svg?color=blue)](https://crates.io/crates/qubit-validation-rules)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-validation-rules` 为 Qubit Rust 服务提供类型化验证规则及带名称的注册项。
如果同一条规则既要直接用于 Rust 代码，又要按配置从 `qubit-validator` 注册表中
选取，这个库可以共用规则实现，并保留结构化违规信息。

## 安装

在应用的 `Cargo.toml` 中加入两个 crate；最低 Rust 版本为 1.94：

```toml
[dependencies]
qubit-validation-rules = "0.1"
qubit-validator = "0.1"
```

需要额外的规则族或全局自动发现时，启用下文对应的 feature。

## 快速开始

例如，服务需要校验用户邮箱和回调 URI。固定字段可以直接调用类型化规则；
当回调 URI 的规则 ID 来自配置时，再用同一套规则建立局部注册表：

```rust
use qubit_validation_rules::registrations;
use qubit_validation_rules::ids;
use qubit_validation_rules::text::EmailAscii;
use qubit_validation_rules::text::Uri;
use qubit_validator::BoundValidationContext;
use qubit_validator::InputType;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::Validator;
use qubit_validator::ValidatorRegistry;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let email = "user@example.com";
    let callback_uri = "https://example.com/callback";
    EmailAscii.validate(email, &())?;
    Uri.validate(callback_uri, &())?;

    let registry = ValidatorRegistry::from_registrations(registrations())?;
    let rule = registry.bind(ids::TEXT_URI, InputType::Text, &[], &[])?;
    let outcome = rule.validate(
        ValidationValue::Text(callback_uri),
        &BoundValidationContext::new(&[]),
    )?;
    assert_eq!(outcome, ValidationOutcome::Valid);
    Ok(())
}
```

直接调用会返回类型化错误；通过注册表绑定的规则在输入无效时返回结构化违规，
其中包含规则 ID 和违规码。应用仍需自行规定允许哪些 URI scheme 和目标地址。

## 规则及适用边界

| 规则 | 校验范围 |
| --- | --- |
| `text::CharLength` | 按 Unicode 标量值（`char`）计数，不按用户可见字符簇计数。 |
| `text::ByteLength` | 按 UTF-8 字节计数；同一段文本的结果可能与 `CharLength` 不同。 |
| `text::EmailAscii` | 检查 ASCII 邮箱轮廓和长度，不确认邮箱是否存在或能否收信。 |
| `text::AllowedChars` | 按所选字符策略校验。`PrintableUnicode` 允许 Unicode 字母、标记、数字、标点、符号和空格分隔符；拒绝控制、格式、私用区、未分配、行分隔符及段分隔符字符。 |
| `text::Uri` | 检查 RFC 3986 绝对 URI 的通用语法，接受 `mailto:`、`urn:` 等 scheme；不确认 scheme 是否适合应用、主机是否存在或地址是否可达。 |
| `collection::ItemCount` | 使用 `usize` 表示包含端点的数量上下界；可接受的最大值随目标架构而变。 |
| `collection::Range<T>` | 对可比较的值检查包含或排除端点的区间；`NaN` 等无法排序的值会被拒绝。 |
| `identity::ChinaIdentity18Structure` | 启用 `china-identity` 后，检查中国大陆 18 位身份证号码的长度、主体数字、出生日期及校验位；不确认地区码有效或已分配、号码已签发，也不核实持有人身份。 |

内置规则还覆盖非空白文本、允许的字符、文本依赖、标准形式 UUID 文本、
中国大陆手机号结构，以及可选的正则表达式。`Range<T>` 与
`ChinaIdentity18Structure` 是类型化规则；后者有意不加入内置动态注册项。

## 注册方式与功能开关

`registrations()` 会列出内置动态规则；即使不启用任何 feature，也能用它建立
局部 `ValidatorRegistry`，明确控制每个注册表的生命周期和内容。如果应用需要
进程级自动发现，可以启用 `inventory`，使用 `ValidatorRegistry::global()` 和
`register_validator!`。

`ids` 模块公开 `ids::TEXT_URI` 等内置规则 ID 常量。应用代码引用内置规则时，
应使用这些常量。

| Feature | 作用 |
| --- | --- |
| 默认 | 不启用可选功能；类型化规则和 `registrations()` 仍可使用。 |
| `inventory` | 将内置动态规则纳入全局自动发现。 |
| `regex` | 增加 `regex_rule::RegexMatch` 及其动态注册项。 |
| `china-identity` | 增加类型化规则 `identity::ChinaIdentity18Structure`，不将它动态注册。 |

## 延伸阅读

- 运行 `cargo doc --all-features --no-deps --open`，在本地生成并打开 API 文档。
- [English documentation](README.md)

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

仓库地址：[https://github.com/qubit-ltd/rs-validation-rules](https://github.com/qubit-ltd/rs-validation-rules)
