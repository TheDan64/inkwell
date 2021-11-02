# Inkwell(s)

[![Crates.io](https://img.shields.io/crates/v/inkwell.svg?style=plastic)](https://crates.io/crates/inkwell)
[![Build Status](https://travis-ci.com/TheDan64/inkwell.svg?branch=master)](https://travis-ci.com/TheDan64/inkwell)
[![codecov](https://codecov.io/gh/TheDan64/inkwell/branch/master/graph/badge.svg)](https://codecov.io/gh/TheDan64/inkwell)
[![lines of code](https://tokei.rs/b1/github/TheDan64/inkwell)](https://github.com/Aaronepower/tokei)
[![Join the chat at https://gitter.im/inkwell-rs/Lobby](https://badges.gitter.im/inkwell-rs/Lobby.svg)](https://gitter.im/inkwell-rs/Lobby?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)
![Minimum rustc 1.42](https://img.shields.io/badge/rustc-1.42+-brightgreen.svg)

**I**t's a **N**ew **K**ind of **W**rapper for **E**xposing **LL**VM (*S*afely)

Inkwell aims to help you pen your own programming languages by safely wrapping llvm-sys. It provides a more strongly typed interface than the underlying LLVM C API so that certain types of errors can be caught at compile time instead of at LLVM's runtime. This means we are trying to replicate LLVM IR's strong typing as closely as possible. The ultimate goal is to make LLVM safer from the rust end and a bit easier to learn (via documentation) and use.

## Requirements

* Rust 1.42+
* Rust Stable, Beta, or Nightly
* LLVM 3.6, 3.7, 3.8, 3.9, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, or 13.0

## Usage

You'll need to point your Cargo.toml to an existing preview crate on crates.io or the master
branch with a corresponding LLVM feature flag:

```toml
[dependencies]
inkwell = { git = "https://github.com/TheDan64/inkwell", branch = "master", features = ["llvm12-0"] }
```

Supported versions:

| LLVM Version | Cargo Feature Flag |
| :----------: | :-----------: |
| 3.6.x        | llvm3-6       |
| 3.7.x        | llvm3-7       |
| 3.8.x        | llvm3-8       |
| 3.9.x        | llvm3-9       |
| 4.0.x        | llvm4-0       |
| 5.0.x        | llvm5-0       |
| 6.0.x        | llvm6-0       |
| 7.0.x        | llvm7-0       |
| 8.0.x        | llvm8-0       |
| 9.0.x        | llvm9-0       |
| 10.0.x       | llvm10-0      |
| 11.0.x       | llvm11-0      |
| 12.0.x       | llvm12-0      |
| 13.0.x       | llvm13-0      |

Please be aware that we may make breaking changes on master from time to time since we are
pre-v1.0.0, in compliance with semver. Please prefer a crates.io release whenever possible!

## Documentation

Documentation is automatically [deployed here](https://thedan64.github.io/inkwell/) based on master. These docs are not yet 100% complete and only show the latest supported LLVM version due to a rustdoc issue. See [#2](https://github.com/TheDan64/inkwell/issues/2) for more info.

## Examples

### Tari's [llvm-sys example](https://gitlab.com/taricorp/llvm-sys.rs/blob/6411edb2fed1a805b7ec5029afc9c3ae1cf6c842/examples/jit-function.rs) written in safe code<sup>1</sup> with Inkwell:

```rust
use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::targets::{InitializationConfig, Target};
use std::error::Error;

/// Convenience type alias for the `sum` function.
///
/// Calling this is innately `unsafe` because there's no guarantee it doesn't
/// do `unsafe` operations internally.
type SumFunc = unsafe extern "C" fn(u64, u64, u64) -> u64;

struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    fn jit_compile_sum(&self) -> Option<JitFunction<SumFunc>> {
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[i64_type.into(), i64_type.into(), i64_type.into()], false);
        let function = self.module.add_function("sum", fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(basic_block);

        let x = function.get_nth_param(0)?.into_int_value();
        let y = function.get_nth_param(1)?.into_int_value();
        let z = function.get_nth_param(2)?.into_int_value();

        let sum = self.builder.build_int_add(x, y, "sum");
        let sum = self.builder.build_int_add(sum, z, "sum");

        self.builder.build_return(Some(&sum));

        unsafe { self.execution_engine.get_function("sum").ok() }
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    let context = Context::create();
    let module = context.create_module("sum");
    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None)?;
    let codegen = CodeGen {
        context: &context,
        module,
        builder: context.create_builder(),
        execution_engine,
    };

    let sum = codegen.jit_compile_sum().ok_or("Unable to JIT compile `sum`")?;

    let x = 1u64;
    let y = 2u64;
    let z = 3u64;

    unsafe {
        println!("{} + {} + {} = {}", x, y, z, sum.call(x, y, z));
        assert_eq!(sum.call(x, y, z), x + y + z);
    }

    Ok(())
}
```

<sup>1</sup> There are two uses of `unsafe` in this example because the actual
act of compiling and executing code on the fly is innately `unsafe`. For one,
there is no way of verifying we are calling `get_function()` with the right function
signature. It is also `unsafe` to *call* the function we get because there's no
guarantee the code itself doesn't do `unsafe` things internally (the same reason
you need `unsafe` when calling into C).

### LLVM's [Kaleidoscope Tutorial](https://llvm.org/docs/tutorial/index.html)

Can be found in the examples directory.

## Alternative Crate(s)

* [llvm-ir](https://github.com/cdisselkoen/llvm-ir)

## Contributing

Check out our [Contributing Guide](.github/CONTRIBUTING.md)
