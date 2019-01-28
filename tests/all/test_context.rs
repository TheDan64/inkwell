extern crate inkwell;

use self::inkwell::AddressSpace;
use self::inkwell::context::Context;
use self::inkwell::types::{IntType, StructType};

#[test]
fn test_no_context_double_free() {
    let context = Context::create();
    let int = context.i8_type();

    {
        int.get_context();
    }
}

// FIXME: This isn't actually safe and stopped working as
// of a recent rust version (late 2018)
// #[test]
// fn test_no_context_double_free2() {
//     let context = Context::create();
//     let int = context.i8_type();
//     let context2 = int.get_context();

//     fn move_(_: Context) {}

//     move_(context);

//     context2.i8_type().const_int(0, false);
// }

#[test]
fn test_no_context_double_free3() {
    Context::get_global();
    Context::get_global();
}

#[test]
fn test_get_context_from_contextless_value() {
    let int = IntType::i8_type();
    let context = Context::create();
    let global_context = Context::get_global();

    assert_eq!(int.get_context(), global_context);
    assert_ne!(*int.get_context(), context);
    assert_ne!(*global_context, context);
}

#[test]
fn test_basic_block_context() {
    let context = Context::create();
    let module = context.create_module("my_mod");
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let fn_value = module.add_function("my_fn", fn_type, None);
    let basic_block = fn_value.append_basic_block("entry");

    assert_eq!(basic_block.get_context(), Context::get_global());

    let basic_block2 = context.append_basic_block(&fn_value, "entry2");

    assert_eq!(*basic_block2.get_context(), context);
}

#[test]
fn test_values_get_context() {
    let context = Context::create();
    let void_type = context.void_type();
    let i8_type = context.i8_type();
    let f32_type = context.f32_type();
    let f32_vec_type = f32_type.vec_type(3);
    let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    let f32_array_type = f32_type.array_type(2);
    let fn_type = f32_type.fn_type(&[], false);
    let struct_type = StructType::struct_type(&[i8_type.into(), f32_type.into()], false);

    assert_eq!(*f32_type.get_context(), context);
    assert_eq!(*void_type.get_context(), context);
    assert_eq!(*f32_vec_type.get_context(), context);
    assert_eq!(*f32_ptr_type.get_context(), context);
    assert_eq!(*f32_array_type.get_context(), context);
    assert_eq!(*fn_type.get_context(), context);
    assert_eq!(*i8_type.get_context(), context);
    assert_eq!(struct_type.get_context(), Context::get_global());
}
