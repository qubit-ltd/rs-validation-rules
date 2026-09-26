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

例如，服务可以直接校验邮箱和回调 URI；如果 URI 规则由配置决定，也能再按稳定
规则 ID 从注册表中绑定同一条规则。本库负责提供可复用的校验行为，允许哪些 URI
scheme 和目标地址仍由应用决定。

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
    let rule = registry.bind(ids::TEXT_URI, InputType::Text, &[])?;
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
| `text::CharLength` | 按 Unicode 标量值（`char`）计数，不按用户可见字符簇计数；`min`、`max` 至少提供一个。配置上下界使用 `u32`，实际计数保留为 `usize` 比较，不会窄化回绕。 |
| `text::ByteLength` | 按 UTF-8 字节计数；同一段文本的结果可能与 `CharLength` 不同。`min`、`max` 至少提供一个。配置上下界使用 `u32`，实际计数保留为 `usize` 比较，不会窄化回绕。 |
| `text::EmailAscii` | 检查 ASCII 邮箱轮廓和长度，不确认邮箱是否存在或能否收信。 |
| `text::AllowedChars` | 按所选字符策略校验。`PrintableUnicode` 允许 Unicode 字母、标记、数字、标点、符号和空格分隔符；拒绝控制、格式、私用区、未分配、行分隔符及段分隔符字符。 |
| `text::Uri` | 检查 RFC 3986 绝对 URI 的通用语法，接受 `mailto:`、`urn:` 等 scheme；不确认 scheme 是否适合应用、主机是否存在或地址是否可达。 |
| `collection::ItemCount` | 至少提供一个包含端点的数量边界，边界使用 `usize`；可接受的最大值随目标架构而变。 |
| `collection::Range<T>` | 对可比较的值检查端点顺序及包含或排除关系；`NaN` 等无法排序的值会被拒绝。端点有序不保证离散类型中存在区间成员，例如开整数区间 `(1, 2)`。 |
| `collection::UniqueItems` | 通过 `first_duplicate_with_limit(values, max_comparisons)` 查找第一对重复元素；限额约束实际 `PartialEq` 调用次数。模型执行另有独立比较预算。 |
| `decimal::DecimalValue` | 启用 `decimal` 后，检查规范化 `BigDecimal` 的小数位数、可选 `DECIMAL(p,s)` 总容量和精确区间端点；不会对输入舍入。 |
| `time::TimePrecision` | 启用 `time` 后，对 `DateTime<Utc>`、`NaiveDateTime` 和 `NaiveTime` 检查精确的秒、毫秒、微秒或纳秒粒度，不做舍入。 |
| `identity::ChinaIdentity18Structure` | 启用 `china-identity` 后，检查中国大陆 18 位身份证号码的长度、主体数字、出生日期及校验位；不确认地区码有效或已分配、号码已签发，也不核实持有人身份。 |

内置规则还覆盖非空白文本、允许的字符、文本依赖、标准形式 UUID 文本、
中国大陆手机号结构，以及可选的正则表达式。`Range<T>` 与
`ChinaIdentity18Structure` 是类型化规则；后者有意不加入内置动态注册项。

`qubit-model-metadata` 执行声明时，外层 Map entry 数量通过生成的长度适配器复用
`ItemCount`；外层 sequence `unique_items` 使用生成的元素相等性适配器，并非通用注册规则。
模型计划在构建阶段检查适配器和具体类型。`max_nodes` 限制读取与规则调用，
`max_comparisons` 限制序列元素成对比较。该后端仍不执行 selector 内的标准约束，
也不遍历 Map key/value。准确范围见 metadata 的[执行矩阵](../rs-model-metadata/doc/user_guide.zh_CN.md#限制执行范围与构建拒绝)。

## 注册方式与功能开关

`registrations()` 会列出内置动态规则；即使不启用任何 feature，也能用它建立
局部 `ValidatorRegistry`，明确控制每个注册表的生命周期和内容。如果应用需要
进程级自动发现，可以启用 `inventory`，使用 `ValidatorRegistry::global()` 和
`register_validator!`。

绑定 `CharLength`、`ByteLength` 或 `ItemCount` 时，`min`、`max` 至少要提供一个；
两者都缺省会返回 `BindErrorKind::InvalidBounds`。

`ids` 模块公开 `ids::TEXT_URI` 等内置规则 ID 常量。应用代码引用内置规则时，
应使用这些常量。

| Feature | 作用 |
| --- | --- |
| 默认 | 不启用可选功能；类型化规则和 `registrations()` 仍可使用。 |
| `inventory` | 将内置动态规则纳入全局自动发现。 |
| `regex` | 增加 `regex_rule::RegexMatch` 及其动态注册项。 |
| `china-identity` | 增加类型化规则 `identity::ChinaIdentity18Structure`，不将它动态注册。 |
| `decimal` | 增加类型化 `decimal::DecimalValue` 及面向 `BigDecimal` 的 `ids::DECIMAL_VALUE` 注册项。 |
| `time` | 增加类型化 `time::TimePrecision` 及面向三种 chrono 时间类型的 `ids::TIME_PRECISION` 注册项。 |

`text::MatchesDependency` 用 `MatchesDependencyError::MissingDependency` 表示第零个依赖缺失或不是文本，
用 `Mismatch` 表示两段文本不相等。注册表将真正的不相等报告为 `text.dependency_mismatch`，
缺失依赖则属于执行错误。动态 ID `qubit.rules.text.email_ascii` 不变；模型声明应改用
`email_ascii`，Rust 枚举应改用 `TextFormat::EmailAscii`。

## 延伸阅读

- [用户手册](doc/user_guide.zh_CN.md)：了解类型化规则、注册表绑定、错误处理和
  feature 选择。
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
