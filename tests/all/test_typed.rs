use inkwell::context::Context;
use std::convert::TryInto;
use inkwell::typed::value::TypedIntValue;

#[test]
fn test_typed_builder_math() {
    let context = Context::create();
    let module = context.create_module("typed_mod");
    let builder = context.create_builder();

    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[i32_type.into(), i32_type.into()], false);
    let fn_value = module.add_function("test_add", fn_type, None);

    let basic_block = context.append_basic_block(fn_value, "entry");
    builder.position_at_end(basic_block);

    let typed_builder = builder.as_typed();

    let param1 = fn_value.get_first_param().unwrap().into_int_value();
    let param2 = fn_value.get_last_param().unwrap().into_int_value();

    // TryInto TypedIntValue<32>
    let typed_param1: TypedIntValue<32> = param1.try_into().unwrap();
    let typed_param2: TypedIntValue<32> = param2.try_into().unwrap();

    // Compile-time guaranteed math
    let result = typed_builder.build_int_add(typed_param1, typed_param2, "add_res").unwrap();
    
    // Fallback to untyped to build return value
    builder.build_return(Some(&result.as_untyped())).unwrap();

    assert!(fn_value.verify(true));
}
