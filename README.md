# Inkwell(s)

[![Crates.io](https://img.shields.io/crates/v/inkwell.svg?style=plastic)](https://crates.io/crates/inkwell)
[![Build Status](https://travis-ci.org/TheDan64/inkwell.svg?branch=master)](https://travis-ci.org/TheDan64/inkwell)
[![codecov](https://codecov.io/gh/TheDan64/inkwell/branch/master/graph/badge.svg)](https://codecov.io/gh/TheDan64/inkwell)
[![lines of code](https://tokei.rs/b1/github/TheDan64/inkwell)](https://github.com/Aaronepower/tokei)
[![Join the chat at https://gitter.im/inkwell-rs/Lobby](https://badges.gitter.im/inkwell-rs/Lobby.svg)](https://gitter.im/inkwell-rs/Lobby?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

**I**t's a **N**ew **K**ind of **W**rapper for **E**xposing **LL**VM (*S*afely)

Inkwell aims to help you pen your own programming languages by safely wrapping llvm-sys. It provides a more strongly typed interface than the underlying LLVM API so that certain types of errors can be caught at compile time instead of at LLVM's runtime. This means we are trying to replicate LLVM IR's strong typing as closely as possible. The ultimate goal is to make LLVM safer from the rust end and a bit easier to learn (via documentation) and use.

## Requirements

* Any Rust version released in the last year or so
* Rust Stable, Beta, or Nightly
* LLVM 3.6, 3.7, or 3.8 (3.9+ support is planned: [#1](https://github.com/TheDan64/inkwell/issues/1))

## Usage

You'll need to point your Cargo.toml to a branch and use a feature flag corresponding to a supported LLVM version:

```toml
[dependencies]
inkwell = { git = "https://github.com/TheDan64/inkwell", branch = "llvm3-7", features = ["llvm3-7"] }
```

Supported versions:

| GitHub Branch | Feature Flag |
| :-----------: | :----------: |
| llvm3-6       | llvm3-6      |
| llvm3-7       | llvm3-7      |
| llvm3-8       | llvm3-8      |

In the root of your source code you will have to add an extern crate to begin using Inkwell:

```rust
extern crate inkwell;
```

## Documentation

Documenation is automatically deployed [here](https://thedan64.github.io/inkwell/) based on master. These docs are not yet 100% complete and only show the latest supported LLVM version due to a rustdoc issue. See [#2](https://github.com/TheDan64/inkwell/issues/2) for more info.

## Examples

### Tari's [llvm-sys example](https://bitbucket.org/tari/llvm-sys.rs/src/ea4ac92a171da2c1851806b91e531ed3a0b41091/examples/jit-function.rs) written in safe code<sup>1</sup> with Inkwell:

```rust
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use inkwell::targets::{InitializationConfig, Target};
use std::mem::transmute;

Target::initialize_native(&InitializationConfig::default())?;

let context = Context::create();
let module = context.create_module("sum");
let builder = context.create_builder();
let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None)?;

let i64_type = context.i64_type();
let fn_type = i64_type.fn_type(&[&i64_type, &i64_type, &i64_type], false);

let function = module.add_function("sum", &fn_type, None);
let basic_block = context.append_basic_block(&function, "entry");

builder.position_at_end(&basic_block);

let x = function.get_nth_param(0)?.into_int_value();
let y = function.get_nth_param(1)?.into_int_value();
let z = function.get_nth_param(2)?.into_int_value();

let sum = builder.build_int_add(&x, &y, "sum");
let sum = builder.build_int_add(&sum, &z, "sum");

builder.build_return(Some(&sum));

let addr = execution_engine.get_function_address("sum")?;

let sum: extern "C" fn(u64, u64, u64) -> u64 = unsafe { transmute(addr) };

let x = 1u64;
let y = 2u64;
let z = 3u64;

assert_eq!(sum(x, y, z), x + y + z);
```

<sup>1</sup> Casting the LLVM JIT function address into a rust function does require a single unsafe transmute, since Inkwell doesn't know what the function signature is. Maybe we can do something about this in the future? In theory, fn_type does contain all the needed info, so whether or not we can do this automagically depends on what rust is capable of. Converting structs, pointers, and other types could be tricky but might be seen as a form of deserialization. See [#5](https://github.com/TheDan64/inkwell/issues/5) for the tracking issue.

### LLVM's [Kaleidoscope Tutorial](https://llvm.org/docs/tutorial/index.html)

Can be found in the examples directory.

## Contributing

Check out our [Contributing Guide](.github/CONTRIBUTING.md)
