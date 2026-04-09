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

    // TryInto TypedIntValue<'_, 32>
    let typed_param1: TypedIntValue<'_, 32> = param1.try_into().unwrap();
    let typed_param2: TypedIntValue<'_, 32> = param2.try_into().unwrap();

    // Compile-time guaranteed math
    let sum = typed_builder.build_int_add(typed_param1, typed_param2, "add_res").unwrap();
    assert_eq!(sum.as_untyped().get_type().get_bit_width(), 32);

    let diff = typed_builder.build_int_sub(sum, typed_param2, "sub_res").unwrap();
    assert_eq!(diff.as_untyped().get_type().get_bit_width(), 32);

    let result = typed_builder.build_int_mul(diff, typed_param1, "mul_res").unwrap();
    assert_eq!(result.as_untyped().get_type().get_bit_width(), 32);
    
    // Fallback to untyped to build return value
    builder.build_return(Some(&result.as_untyped())).unwrap();

    assert!(fn_value.verify(true));
}

#[test]
fn test_typed_int_negative_width_match() {
    let context = Context::create();

    let i64_type = context.i64_type();
    let val_64 = i64_type.const_int(42, false);

    // Should fail to convert to TypedIntValue<'_, 32>
    let typed_val_32_res: Result<TypedIntValue<'_, 32>, _> = val_64.try_into();
    assert!(typed_val_32_res.is_err());

    let new_res = TypedIntValue::<'_, 32>::new(val_64);
    assert!(new_res.is_none());
}
