# qubit-validation-rules

基于 `qubit-validator` 的类型安全验证规则集合。

## 使用方式

规则仍然可以作为普通的类型化 Rust validator 使用。需要动态选择规则时，
可以在不启用任何 feature 的情况下构造隔离的局部注册表：

```rust
use qubit_validation_rules::registrations;
use qubit_validator::ValidatorRegistry;

let registry = ValidatorRegistry::from_registrations(registrations())?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

启用 `inventory` 后，可以使用进程级 `ValidatorRegistry::global` 和
`register_validator!` 自动发现。`regex` 与 `china-identity` 分别启用对应规则族。

注册表绑定层会为每条结构化违规统一写入注册规则 ID；规则实现只负责提供违规代码和
安全参数。

## Features

- `inventory`：进程级静态注册。
- `regex`：正则表达式规则。
- `china-identity`：中国大陆居民身份证规则。

## 许可证

Apache-2.0。
