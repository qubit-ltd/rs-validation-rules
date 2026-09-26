# 用户手册

[English](user_guide.md) | [中文 README](../README.zh_CN.md)

本手册面向使用 `qubit-validation-rules` 0.1 和 `qubit-validator` 0.1 的
Rust 应用开发者，介绍如何在固定业务逻辑中直接调用类型化规则，以及如何在
规则 ID 或参数来自配置时使用注册表规则。

## 手册目标与读者

当应用需要复用常见校验逻辑，又不想分别维护普通 Rust 调用和可配置注册表的
两套实现时，可以使用本 crate。规则负责检查输入的结构和边界；业务策略、外部
查询和身份核验仍由应用处理。

## 概念模型

规则有两种使用方式：

| 形式 | 选择方式 | 结果 |
| --- | --- | --- |
| 类型化规则 | 直接使用 `text::Uri`、`collection::ItemCount` 等 Rust 类型 | 失败时返回对应的 Rust 错误类型 |
| 注册规则 | 使用 `ids::TEXT_URI` 等稳定 ID，通过 `ValidatorRegistry::bind` 绑定 | 返回 `ValidationOutcome`；无效输入包含结构化违规信息 |

`registrations()` 提供内置动态注册项。应用可据此建立局部注册表，并自行控制
其生命周期和内容。可选的 `inventory` feature 支持通过
`ValidatorRegistry::global()` 进行进程级发现。
每条内置规则只使用一个注册常量，同时供局部列表和 inventory 发现使用，因此两条路径的 ID、描述符和来源元数据保持一致。

## 实战场景：检查回调 URI

服务接收邮箱地址和回调 URI。固定字段可以直接调用规则校验；如果回调规则由配置
选择，则按稳定 ID 从注册表绑定。URI 语法校验通过，只表示它是绝对 URI，并不表示
其 scheme 或目标地址适合服务使用。

## 安装与最小配置

在应用的 manifest 中加入这两个 crate。项目要求 Rust 1.94 或更高版本。

```toml
[dependencies]
qubit-validation-rules = "0.1"
qubit-validator = "0.1"
```

## 核心工作流

如果代码已确定要执行哪些检查，可以直接调用类型化规则。`Validator` trait 提供
`validate` 方法：

```rust
use qubit_validation_rules::text::EmailAscii;
use qubit_validation_rules::text::Uri;
use qubit_validator::Validator;

fn validate_fixed_fields(email: &str, callback_uri: &str) -> Result<(), Box<dyn std::error::Error>> {
    EmailAscii.validate(email, &())?;
    Uri.validate(callback_uri, &())?;
    Ok(())
}
```

如果规则由 ID 选择，先用内置注册项创建注册表，再绑定并执行规则：

```rust
use qubit_validation_rules::ids;
use qubit_validation_rules::registrations;
use qubit_validator::BoundValidationContext;
use qubit_validator::InputType;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorRegistry;

fn validate_configured_uri(value: &str) -> Result<ValidationOutcome, Box<dyn std::error::Error>> {
    let registry = ValidatorRegistry::from_registrations(registrations())?;
    let rule = registry.bind(ids::TEXT_URI, InputType::Text, &[])?;
    Ok(rule.validate(
        ValidationValue::Text(value),
        &BoundValidationContext::new(&[]),
    )?)
}
```

`ValidationOutcome::Valid` 表示 URI 语法通过。`ValidationOutcome::Invalid` 会携带
结构化违规信息。若 ID 未知或参数无效，绑定阶段会先返回错误，规则不会开始校验。

带参数的规则在绑定时接收具名参数。下面的例子要求文本至少包含两个 Unicode
标量值：

```rust
use qubit_validation_rules::ids;
use qubit_validation_rules::registrations;
use qubit_validator::BoundValidationContext;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorRegistry;

let registry = ValidatorRegistry::from_registrations(registrations())?;
let arguments = [NamedValidationArgument::new(
    "min",
    ValidationArgument::Unsigned(2),
)];
let rule = registry.bind(ids::TEXT_CHAR_LENGTH, InputType::Text, &arguments)?;
let outcome = rule.validate(
    ValidationValue::Text("éa"),
    &BoundValidationContext::new(&[]),
)?;
assert!(matches!(outcome, qubit_validator::ValidationOutcome::Valid));
# Ok::<(), Box<dyn std::error::Error>>(())
```

## 进阶用法

### 选择类型化规则或注册规则

如果应用代码决定使用哪条规则，并能处理对应的 Rust 错误类型，优先直接调用类型化
规则。如果配置或共享模型通过 ID 引用规则，并需要统一的结构化结果，则使用注册表。
两种入口执行的是同一套规则行为。

### 按需启用规则族

默认 feature 集包含类型化规则和局部 `registrations()`。其他 feature 提供：

| Feature | 增加的能力 |
| --- | --- |
| `inventory` | 支持内置规则的进程级注册表发现 |
| `regex` | 增加 `regex_rule::RegexMatch` 及其动态注册项 |
| `china-identity` | 增加类型化规则 `identity::ChinaIdentity18Structure` |

`Range<T>` 只作为类型化规则提供。中国大陆身份证规则也不会加入动态注册项。
应用只需开启实际使用的 feature。`rs-model-metadata` 使用
`registrations()` 建立局部注册表，该路径不需要启用 `inventory`。

### 其他规则的校验范围

- `CharLength` 按 Unicode 标量值计数；`ByteLength` 按 UTF-8 字节计数。两者都不按
  用户感知的字素簇计数。
- `AllowedChars` 支持 `Unicode`、`PrintableUnicode`、`Ascii`、`PrintableAscii` 和
  `Code`。`PrintableUnicode` 会拒绝控制字符和格式字符，包括零宽连字符。
- `EmailAscii` 检查 ASCII 邮箱的格式轮廓和长度，不确认邮箱是否存在或能否收信。
- `Uri` 检查 RFC 3986 绝对 URI 语法，也接受 `mailto:`、`urn:` 等非 Web scheme。
- `ItemCount` 使用 `usize` 表示上下界；`Range<T>` 可为可部分排序的值配置包含、
  排除或无界端点；端点有序不代表离散类型的区间内一定有值。
- `CharLength`、`ByteLength` 和 `ItemCount` 至少要配置一个边界；绑定时 `min`、`max`
  都缺省会返回 `InvalidBounds`。

## 错误与诊断

类型化规则返回领域错误，例如 `TextRuleError::Uri`、
`TextLengthError::TooShort` 或 `RangeError::Unordered`。带可配置边界的构造函数在
边界缺失或无效时返回 `qubit_validator::BindError`。长度与数量规则缺少两个边界时
返回 `BindErrorKind::InvalidBounds`；`min` 大于 `max` 时返回 `ParameterOutOfRange`。

注册表会在执行前检查规则 ID、输入类型、参数、依赖和 feature 是否可用。例如，
`min` 大于 `max` 时无法绑定。规则执行后，未通过的值由
`ValidationOutcome::Invalid` 表示；这与绑定错误不同。程序应根据违规码和参数处理
结果，不要解析面向人的错误展示文本。

## 排障

| 现象 | 检查方向 |
| --- | --- |
| 注册表绑定失败 | 核对规则 ID、`InputType`、参数名称和类型，以及所需 feature 是否启用。 |
| `CharLength` 拒绝文本 | 检查 Unicode 标量值数量；一个可见字符可能由多个标量值组成。 |
| `ByteLength` 拒绝文本 | 检查 UTF-8 字节数；非 ASCII 文本中的一个标量值通常占多个字节。 |
| URI 通过校验但不适合作为回调地址 | 语法校验后，还需由应用限制 scheme、主机和目标地址。 |
| `PrintableUnicode` 拒绝看起来可见的文本 | 检查是否含有零宽连字符等格式字符。 |

## 限制与最佳实践

- 这些规则只检查语法、结构或边界，不执行 DNS、网络、邮箱、签发状态或身份核验。
- URI 通过校验不代表授权或网络安全。将 URI 用作回调地址前，应由应用限制 scheme
  和目标地址。
- 字符长度按 Unicode 标量值计算，不等于用户感知的字素簇数量。外部协议按字节
  定义长度时，应使用字节长度规则。
- `Range<T>` 使用部分排序；无法排序的值（包括 `NaN`）会被拒绝。构造时检查端点关系，
  无法保证任意离散类型的有序开区间中存在值，例如开整数区间 `(1_i32, 2_i32)`。
- `ItemCount` 的边界类型为平台相关的 `usize`，不同架构可表示的最大数量可能不同。

## 延伸阅读

- [中文 README](../README.zh_CN.md) 和 [English user guide](user_guide.md)
- [API 文档](https://docs.rs/qubit-validation-rules)
- 注册表与校验模型详见 [qubit-validator README](../../rs-validator/README.zh_CN.md)
