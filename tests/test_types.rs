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
    let av_struct = context.struct_type(&[&int_vector, &float_array], false, "");

    assert!(!av_struct.is_packed());
    assert!(!av_struct.is_opaque());
    assert!(av_struct.is_sized());
    assert!(av_struct.get_name().is_none());

    let av_struct = context.struct_type(&[&int_vector, &float_array], true, "av_struct");

    assert!(av_struct.is_packed());
    assert!(!av_struct.is_opaque());
    assert!(av_struct.is_sized());
    assert_eq!(av_struct.get_name(), Some(&*CString::new("av_struct").unwrap()));
}
