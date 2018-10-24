extern crate inkwell;

use self::inkwell::OptimizationLevel;
use self::inkwell::context::Context;
use self::inkwell::targets::{InitializationConfig, Target};
use self::inkwell::execution_engine::JitFunction;

#[test]
fn test_tari_example() {
    Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");

    let context = Context::create();
    let module = context.create_module("sum");
    let builder = context.create_builder();
    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();

    let i64_type = context.i64_type();
    let fn_type = i64_type.fn_type(&[i64_type.into(), i64_type.into(), i64_type.into()], false);

    let function = module.add_function("sum", fn_type, None);
    let basic_block = context.append_basic_block(&function, "entry");

    builder.position_at_end(&basic_block);

    let x = function.get_nth_param(0).unwrap().into_int_value();
    let y = function.get_nth_param(1).unwrap().into_int_value();
    let z = function.get_nth_param(2).unwrap().into_int_value();

    let sum = builder.build_int_add(x, y, "sum");
    let sum = builder.build_int_add(sum, z, "sum");

    builder.build_return(Some(&sum));

    unsafe {
        type Sum = unsafe extern "C" fn(u64, u64, u64) -> u64;
        let sum: JitFunction<Sum> = execution_engine.get_function("sum").unwrap();

        let x = 1u64;
        let y = 2u64;
        let z = 3u64;

        assert_eq!(sum.call(x, y, z), x + y + z);
    }
}
