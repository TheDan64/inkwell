extern crate inkwell;

use self::inkwell::context::Context;

#[test]
fn test_build_call() {
    let context = Context::create();
    let module = context.create_module("sum");
    let builder = context.create_builder();

    let f32_type = context.f32_type();
    let fn_type = f32_type.fn_type(&[], false);

    let function = module.add_function("get_pi", &fn_type, None);
    let basic_block = context.append_basic_block(&function, "entry");

    builder.position_at_end(&basic_block);

    let pi = f32_type.const_float(3.14);

    builder.build_return(Some(&pi));

    let function2 = module.add_function("wrapper", &fn_type, None);
    let basic_block2 = context.append_basic_block(&function2, "entry");

    builder.position_at_end(&basic_block2);

    let pi2 = builder.build_call(&function, &[], "get_pi", false).left().unwrap();

    builder.build_return(Some(&pi2));
}
