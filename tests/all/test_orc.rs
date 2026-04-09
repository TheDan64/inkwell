use inkwell::context::Context;
use inkwell::orc::{LLJITBuilder, ThreadSafeContext, ThreadSafeModule};
use inkwell::targets::{InitializationConfig, Target};

#[test]
fn test_orc_lljit_execution() {
    Target::initialize_native(&InitializationConfig::default()).unwrap();

    let context = Context::create();
    let module = context.create_module("orc_jit_test");
    let builder = context.create_builder();

    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[i32_type.into(), i32_type.into()], false);
    let fn_value = module.add_function("orc_add", fn_type, None);

    let entry = context.append_basic_block(fn_value, "entry");
    builder.position_at_end(entry);

    let x = fn_value.get_first_param().unwrap().into_int_value();
    let y = fn_value.get_nth_param(1).unwrap().into_int_value();
    let sum = builder.build_int_add(x, y, "sum").unwrap();
    builder.build_return(Some(&sum)).unwrap();

    // Create a dummy ThreadSafeContext and promote module
    let tsc = ThreadSafeContext::new();
    let tsm = ThreadSafeModule::new(module, &tsc);

    // Create LLJIT instance
    let lljit = LLJITBuilder::new().build().expect("Failed to create LLJIT");
    lljit.add_module(tsm).expect("Failed to add module to LLJIT");

    // Lookup function address and call it
    unsafe {
        let orc_func = lljit.lookup::<unsafe extern "C" fn(i32, i32) -> i32>("orc_add").expect("Failed to find function");
        let func = orc_func.as_raw();
        let result = func(10, 32);
        assert_eq!(result, 42);
    }
}
