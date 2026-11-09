# protovalidate-unofficial 2.1.0

This package uses the official Protovalidate 2.1.0 Rust and CEL core. It supports
CPython 3.8 and later through a `cp38-abi3` native extension.

```sh
pip install protovalidate-unofficial==2.1.0
```

Use the same import as version 1:

```python
import protovalidate_unofficial as protovalidate

try:
    protovalidate.validate(message)
except protovalidate.ValidationError as error:
    violations = error.to_proto()
```

`Validator`, `validate`, `collect_violations`, `ValidationError`, `CompilationError`,
and `Violations` remain available. Reuse a `Validator` to cache compiled rules.
`fail_fast=True` stops after the first violation.

This backport accepts `google.protobuf` messages. It does not require or support
`protobuf-py`. Standard rules run in Rust. Custom CEL rules run in the bundled
CEL engine. Validation remains enabled.

Version 2 follows the official 2.1.0 rule semantics. `EvaluationError` reports a
failed CEL evaluation. Error text and violation order can differ from version 1.
Violation objects keep `proto`, `field_value`, and `rule_value`. Their `proto`
is a `google.protobuf` message.

## Build and test

Install Rust 1.88 or later and a C++17 compiler. Use a current Clang or GCC.
The source package contains the fixed native dependencies; no git submodule
checkout is required.

```sh
uv tool run --from maturin maturin build --release -i python3.8
uv pip install target/wheels/protovalidate_unofficial-2.1.0-*.whl pytest
PYTHONPATH=gen python -m pytest
```

Prebuilt wheels do not require a compiler at installation time.

See [UPSTREAM.md](UPSTREAM.md) for source versions and licenses.

## Verification

The same wheel passes the behavior tests on CPython 3.8 through 3.14. Python 3.8
also passes with the minimum `protobuf==3.20.3` dependency. The Protobuf
conformance harness passes 2,854 cases without skips.

Strict error classification has two upstream exceptions: CEL references to an
unknown field and a wrong operand type raise `EvaluationError`, instead of
`CompilationError`. Both messages are rejected.
