# Inkwell(s)
[![Build Status](https://travis-ci.org/TheDan64/inkwell.svg?branch=master)](https://travis-ci.org/TheDan64/inkwell)

**I**t's a **N**ew **K**ind of **W**rapper for **E**xposing **LL**VM (*S*afely)

Inkwell aims to help you pen your own programming languages by safely wrapping llvm-sys. It provides a more strongly typed interface than LLVM itself so that certain types of errors can be caught at compile time instead of at LLVM's runtime. The ultimate goal is to make LLVM safer from the rust end and a bit easier to use.

# Documentation
[docs.rs](https://docs.rs/crate/inkwell/) documentation is planned but not yet complete. See [#2](https://github.com/TheDan64/inkwell/issues/2) for the tracking issue.

## Example

Here's [tari's llvm-sys example](https://bitbucket.org/tari/llvm-sys.rs/src/ea4ac92a171da2c1851806b91e531ed3a0b41091/examples/jit-function.rs) written in safe code<sup>1</sup> with Inkwell:

```rust
    use context::Context;
    use std::mem::transmute;

    let context = Context::create();
    let module = context.create_module("sum");
    let builder = context.create_builder();
    let execution_engine = module.create_execution_engine(true).unwrap();

    let i64_type = context.i64_type();
    let fn_type = i64_type.fn_type(&[&i64_type, &i64_type, &i64_type], false);

    let function = module.add_function("sum", &fn_type);
    let basic_block = context.append_basic_block(&function, "entry");

    builder.position_at_end(&basic_block);

    let x = function.get_nth_param(0).unwrap();
    let y = function.get_nth_param(1).unwrap();
    let z = function.get_nth_param(2).unwrap();

    let sum = builder.build_int_add(&x, &y, "sum");
    let sum = builder.build_int_add(&sum, &z, "sum");

    builder.build_return(Some(sum));

    let addr = execution_engine.get_function_address("sum").unwrap();

    let sum: extern "C" fn(u64, u64, u64) -> u64 = unsafe { transmute(addr) };

    let x = 1u64;
    let y = 2u64;
    let z = 3u64;

    assert_eq!(sum(x, y, z), x + y + z);
```

<sup>1</sup> Casting the LLVM JIT function address into a rust function does require a single unsafe transmute, since Inkwell doesn't know what the function signature is. Maybe we can do something about this in the future? In theory, fn_type does contain all the needed info, so whether or not we can do this automagically depends on what rust is capable of. Converting structs, pointers, and other types could be tricky but might be seen as a form of deserialization. See [#5](https://github.com/TheDan64/inkwell/issues/5) for the tracking issue.

# Building

Inkwell requires LLVM to be installed when building. Currently only LLVM 3.7 is supported, though multi-version support is planned. See [#1](https://github.com/TheDan64/inkwell/issues/1) for the tracking issue. Inkwell currently compiles on stable, though this may be subject to change in the future.
