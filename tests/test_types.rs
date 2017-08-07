extern crate inkwell;

use self::inkwell::context::Context;
use self::inkwell::types::{FloatType, IntType, StructType, VoidType};
use std::ffi::CString;

#[test]
fn test_struct_type() {
    let context = Context::create();
    let int = context.i8_type();
    let int_vector = int.vec_type(100);
    let float = context.f32_type();
    let float_array = float.array_type(3);
    let av_struct = context.struct_type(&[&int_vector, &float_array], false);

    assert!(!av_struct.is_packed());
    assert!(!av_struct.is_opaque());
    assert!(av_struct.is_sized());
    assert!(av_struct.get_name().is_none());

    let av_struct = context.struct_type(&[&int_vector, &float_array], true);

    assert!(av_struct.is_packed());
    assert!(!av_struct.is_opaque());
    assert!(av_struct.is_sized());
    // REVIEW: Is there a way to name a non opaque struct?
    assert!(av_struct.get_name().is_none());

    let opaque_struct = context.opaque_struct_type("opaque_struct");

    assert!(!opaque_struct.is_packed());
    assert!(opaque_struct.is_opaque());
    assert!(!opaque_struct.is_sized());
    assert_eq!(opaque_struct.get_name(), Some(&*CString::new("opaque_struct").unwrap()));

    assert!(opaque_struct.set_body(&[&int_vector, &float_array], true));

    assert!(opaque_struct.is_packed());
    assert!(!opaque_struct.is_opaque());
    assert!(opaque_struct.is_sized());
    assert_eq!(opaque_struct.get_name(), Some(&*CString::new("opaque_struct").unwrap()));
}

#[test]
fn test_function_type() {
    let context = Context::create();
    let int = context.i8_type();
    let float = context.f32_type();
    let fn_type = int.fn_type(&[&int, &int, &float], false);

    assert!(!fn_type.is_var_arg());

    let param_types = fn_type.get_param_types();

    assert_eq!(param_types.len(), 3);
    assert_eq!(*param_types[0].as_int_type(), int);
    assert_eq!(*param_types[1].as_int_type(), int);
    assert_eq!(*param_types[2].as_float_type(), float);

    let fn_type = int.fn_type(&[&int, &float], true);

    assert!(fn_type.is_var_arg());
}

#[test]
fn test_sized_types() {
    let void_type = VoidType::void_type();
    let bool_type = IntType::bool_type();
    let i8_type = IntType::i8_type();
    let i16_type = IntType::i16_type();
    let i32_type = IntType::i32_type();
    let i64_type = IntType::i64_type();
    let i128_type = IntType::i128_type();
    let f16_type = FloatType::f16_type();
    let f32_type = FloatType::f32_type();
    let f64_type = FloatType::f64_type();
    let f128_type = FloatType::f128_type();
    let ppc_f128_type = FloatType::ppc_f128_type();
    let struct_type = StructType::struct_type(&[&i8_type, &f128_type], false);

    // REVIEW: Should these maybe just be constant functions instead of bothering to calling LLVM?

    assert!(!void_type.is_sized());
    assert!(bool_type.is_sized());
    assert!(i8_type.is_sized());
    assert!(i16_type.is_sized());
    assert!(i32_type.is_sized());
    assert!(i64_type.is_sized());
    assert!(i128_type.is_sized());
    assert!(f16_type.is_sized());
    assert!(f32_type.is_sized());
    assert!(f64_type.is_sized());
    assert!(f128_type.is_sized());
    assert!(ppc_f128_type.is_sized());
    assert!(struct_type.is_sized());

    assert!(void_type.ptr_type(0).is_sized());
    assert!(bool_type.ptr_type(0).is_sized());
    assert!(i8_type.ptr_type(0).is_sized());
    assert!(i16_type.ptr_type(0).is_sized());
    assert!(i32_type.ptr_type(0).is_sized());
    assert!(i64_type.ptr_type(0).is_sized());
    assert!(i128_type.ptr_type(0).is_sized());
    assert!(f16_type.ptr_type(0).is_sized());
    assert!(f32_type.ptr_type(0).is_sized());
    assert!(f64_type.ptr_type(0).is_sized());
    assert!(f128_type.ptr_type(0).is_sized());
    assert!(ppc_f128_type.ptr_type(0).is_sized());
    assert!(struct_type.ptr_type(0).is_sized());

    // REVIEW: You can't have array of void right?
    assert!(void_type.ptr_type(0).array_type(42).is_sized());
    assert!(bool_type.array_type(42).is_sized());
    assert!(i8_type.array_type(42).is_sized());
    assert!(i16_type.array_type(42).is_sized());
    assert!(i32_type.array_type(42).is_sized());
    assert!(i64_type.array_type(42).is_sized());
    assert!(i128_type.array_type(42).is_sized());
    assert!(f16_type.array_type(42).is_sized());
    assert!(f32_type.array_type(42).is_sized());
    assert!(f64_type.array_type(42).is_sized());
    assert!(f128_type.array_type(42).is_sized());
    assert!(ppc_f128_type.array_type(42).is_sized());
    assert!(struct_type.array_type(0).is_sized());

    // REVIEW: You can't have array of void right?
    assert!(void_type.ptr_type(0).vec_type(42).is_sized());
    assert!(bool_type.vec_type(42).is_sized());
    assert!(i8_type.vec_type(42).is_sized());
    assert!(i16_type.vec_type(42).is_sized());
    assert!(i32_type.vec_type(42).is_sized());
    assert!(i64_type.vec_type(42).is_sized());
    assert!(i128_type.vec_type(42).is_sized());
    assert!(f16_type.vec_type(42).is_sized());
    assert!(f32_type.vec_type(42).is_sized());
    assert!(f64_type.vec_type(42).is_sized());
    assert!(f128_type.vec_type(42).is_sized());
    assert!(ppc_f128_type.vec_type(42).is_sized());
    assert!(struct_type.vec_type(42).is_sized());
}
