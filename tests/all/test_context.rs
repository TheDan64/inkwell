use inkwell::context::Context;
use inkwell::AddressSpace;

#[test]
fn test_no_context_double_free() {
    let context = Context::create();
    let int = context.i8_type();

    {
        int.get_context();
    }
}

#[test]
fn test_no_context_double_free3() {
    unsafe {
        Context::get_global(|_ctx| ());
        Context::get_global(|_ctx| ());
    }
}

#[test]
fn test_get_context_from_contextless_value() {
    let context = Context::create();

    unsafe {
        Context::get_global(|global_context| {
            let int = global_context.i8_type();

            assert_ne!(int.get_context(), context);
            assert_eq!(int.get_context(), *global_context);
            assert_ne!(*global_context, context);
        })
    };
}

#[test]
fn test_basic_block_context() {
    let context = Context::create();
    let module = context.create_module("my_mod");
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let fn_value = module.add_function("my_fn", fn_type, None);
    let basic_block = context.append_basic_block(fn_value, "entry");

    assert_eq!(basic_block.get_context(), context);
}

#[test]
fn test_values_get_context() {
    let context = Context::create();
    let void_type = context.void_type();
    let i8_type = context.i8_type();
    let f32_type = context.f32_type();
    let f32_vec_type = f32_type.vec_type(3);
    #[cfg(not(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    )))]
    let f32_ptr_type = f32_type.ptr_type(AddressSpace::default());
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    let f32_ptr_type = context.ptr_type(AddressSpace::default());
    let f32_array_type = f32_type.array_type(2);
    let fn_type = f32_type.fn_type(&[], false);
    let struct_type = context.struct_type(&[i8_type.into(), f32_type.into()], false);

    assert_eq!(f32_type.get_context(), context);
    assert_eq!(void_type.get_context(), context);
    assert_eq!(f32_vec_type.get_context(), context);
    assert_eq!(f32_ptr_type.get_context(), context);
    assert_eq!(f32_array_type.get_context(), context);
    assert_eq!(fn_type.get_context(), context);
    assert_eq!(i8_type.get_context(), context);
    assert_eq!(struct_type.get_context(), context);
}

#[llvm_versions(12..)]
#[test]
fn test_get_struct_type() {
    let context = Context::create();

    let name = "opaque";
    let opaque = context.opaque_struct_type(name);

    let got = context.get_struct_type(name);
    assert_eq!(got, Some(opaque));

    assert_eq!(context.get_struct_type("non-existent"), None);
}
