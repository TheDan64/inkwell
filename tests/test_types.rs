extern crate inkwell;

use self::inkwell::context::Context;
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
