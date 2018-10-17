extern crate inkwell;

use self::inkwell::attributes::Attribute;
use self::inkwell::context::Context;

use std::ffi::CString;

#[test]
fn test_enum_attribute_kinds() {
    // Does not exist:
    assert_eq!(Attribute::get_named_enum_kind_id("foo"), 0);
    assert_eq!(Attribute::get_named_enum_kind_id("bar"), 0);

    // Many of the values change and are not consistent across LLVM versions
    // so it only seems to make sense to test some consistent subset.
    // Otherwise it's not worth keeping track of. Users will have to
    // play around with it and determine which ones they are interested in
    // for their particular LLVM version

    // Function Attributes:
    assert_eq!(Attribute::get_named_enum_kind_id("allocsize"), 2);
    assert_eq!(Attribute::get_named_enum_kind_id("alwaysinline"), 3);
    assert_eq!(Attribute::get_named_enum_kind_id("argmemonly"), 4);
    assert_eq!(Attribute::get_named_enum_kind_id("builtin"), 5);
    assert_eq!(Attribute::get_named_enum_kind_id("cold"), 7);
    assert_eq!(Attribute::get_named_enum_kind_id("convergent"), 8);
    assert_eq!(Attribute::get_named_enum_kind_id("inaccessiblememonly"), 13);
    assert_eq!(Attribute::get_named_enum_kind_id("inaccessiblemem_or_argmemonly"), 14);
    assert_eq!(Attribute::get_named_enum_kind_id("inlinehint"), 15);
    assert_eq!(Attribute::get_named_enum_kind_id("jumptable"), 16);
    assert_eq!(Attribute::get_named_enum_kind_id("minsize"), 17);
    assert_eq!(Attribute::get_named_enum_kind_id("naked"), 18);
    assert_eq!(Attribute::get_named_enum_kind_id("nobuiltin"), 21);

    // REVIEW: The LLVM docs suggest these fn attrs exist, but don't turn up:
    // assert_eq!(Attribute::get_named_enum_kind_id("no-jump-tables"), 19);
    // assert_eq!(Attribute::get_named_enum_kind_id("null-pointer-is-valid"), 31);
    // assert_eq!(Attribute::get_named_enum_kind_id("optforfuzzing"), 32);
    // assert_eq!(Attribute::get_named_enum_kind_id("patchable-function"), 35);
    // assert_eq!(Attribute::get_named_enum_kind_id("probe-stack"), 36);
    // assert_eq!(Attribute::get_named_enum_kind_id("stack-probe-size"), 36);
    // assert_eq!(Attribute::get_named_enum_kind_id("no-stack-arg-probe"), 32);
    // assert_eq!(Attribute::get_named_enum_kind_id("thunk"), 45);
    // assert_eq!(Attribute::get_named_enum_kind_id("nocf_check"), 45);
    // assert_eq!(Attribute::get_named_enum_kind_id("shadowcallstack"), 45);

    // Parameter Attributes:
    assert_eq!(Attribute::get_named_enum_kind_id("align"), 1);
    assert_eq!(Attribute::get_named_enum_kind_id("byval"), 6);
    assert_eq!(Attribute::get_named_enum_kind_id("dereferenceable"), 9);
    assert_eq!(Attribute::get_named_enum_kind_id("dereferenceable_or_null"), 10);
    assert_eq!(Attribute::get_named_enum_kind_id("inalloca"), 11);
    assert_eq!(Attribute::get_named_enum_kind_id("inreg"), 12);
    assert_eq!(Attribute::get_named_enum_kind_id("nest"), 19);
    assert_eq!(Attribute::get_named_enum_kind_id("noalias"), 20);
    assert_eq!(Attribute::get_named_enum_kind_id("nocapture"), 22);
}

#[test]
fn test_enum_attributes() {
    let context = Context::create();
    let enum_attribute = context.create_enum_attribute(0, 10);

    assert!(enum_attribute.is_enum());
    assert!(!enum_attribute.is_string());

    assert_eq!(enum_attribute.get_enum_kind_id(), 0);
    assert_eq!(enum_attribute.get_enum_value(), 10);

    let enum_attribute2 = context.create_enum_attribute(3, 14);

    assert!(enum_attribute2.is_enum());
    assert!(!enum_attribute2.is_string());

    assert_eq!(enum_attribute2.get_enum_kind_id(), 3);
    assert_eq!(enum_attribute2.get_enum_value(), 14);

    let enum_attribute3 = context.create_enum_attribute(56, 20);

    assert!(enum_attribute3.is_enum());
    assert!(!enum_attribute3.is_string());

    assert_eq!(enum_attribute3.get_enum_kind_id(), 56);
    assert_eq!(enum_attribute3.get_enum_value(), 20);
}

#[test]
fn test_string_attributes() {
    let context = Context::create();
    let string_attribute = context.create_string_attribute("my_key_123", "my_val");

    assert!(!string_attribute.is_enum());
    assert!(string_attribute.is_string());

    assert_eq!(*string_attribute.get_string_kind_id(), *CString::new("my_key_123").unwrap());
    assert_eq!(*string_attribute.get_string_value(), *CString::new("my_val").unwrap());
}

#[test]
fn test_attributes_on_function_values() {
    let context = Context::create();
    let builder = context.create_builder();
    let module = context.create_module("my_mod");
    let void_type = context.void_type();
    let i32_type = context.i32_type();
    let fn_type = void_type.fn_type(&[i32_type.into()], false);
    let fn_value = module.add_function("my_fn", fn_type, None);
    let entry_bb = fn_value.append_basic_block("entry");
    let string_attribute = context.create_string_attribute("my_key", "my_val");
    let enum_attribute = context.create_enum_attribute(1, 1);

    builder.position_at_end(&entry_bb);
    builder.build_return(None);

    assert_eq!(fn_value.count_attributes(0), 0);
    assert_eq!(fn_value.count_attributes(1), 0);

    fn_value.remove_string_attribute(0, "my_key"); // Noop
    fn_value.remove_enum_attribute(0, 1); // Noop

    // define align 1 "my_key"="my_val" void @my_fn()
    fn_value.add_attribute(0, string_attribute);
    fn_value.add_attribute(1, string_attribute); // Applied to 1st param
    fn_value.add_attribute(0, enum_attribute);

    assert_eq!(fn_value.count_attributes(0), 2);
    assert_eq!(fn_value.get_enum_attribute(0, 1), Some(enum_attribute));
    assert_eq!(fn_value.get_string_attribute(0, "my_key"), Some(string_attribute));

    fn_value.remove_string_attribute(0, "my_key");

    assert_eq!(fn_value.count_attributes(0), 1);

    fn_value.remove_enum_attribute(0, 1);

    assert_eq!(fn_value.count_attributes(0), 0);
    assert!(fn_value.get_enum_attribute(0, 1).is_none());
    assert!(fn_value.get_string_attribute(0, "my_key").is_none());
}

#[test]
fn test_attributes_on_call_site_values() {
    let context = Context::create();
    let builder = context.create_builder();
    let module = context.create_module("my_mod");
    let void_type = context.void_type();
    let i32_type = context.i32_type();
    let fn_type = void_type.fn_type(&[i32_type.into()], false);
    let fn_value = module.add_function("my_fn", fn_type, None);
    let entry_bb = fn_value.append_basic_block("entry");
    let string_attribute = context.create_string_attribute("my_key", "my_val");
    let enum_attribute = context.create_enum_attribute(1, 1);

    builder.position_at_end(&entry_bb);

    let call_site_value = builder.build_call(fn_value, &[], "my_fn");

    builder.build_return(None);

    assert_eq!(call_site_value.count_arguments(), 0);
    assert_eq!(call_site_value.count_attributes(0), 0);
    assert_eq!(call_site_value.count_attributes(1), 0);

    call_site_value.remove_string_attribute(0, "my_key"); // Noop
    call_site_value.remove_enum_attribute(0, 1); // Noop

    // define align 1 "my_key"="my_val" void @my_fn()
    call_site_value.add_attribute(0, string_attribute);
    call_site_value.add_attribute(1, string_attribute); // Applied to 1st param
    call_site_value.add_attribute(0, enum_attribute);

    assert_eq!(call_site_value.count_attributes(0), 2);
    assert_eq!(call_site_value.get_enum_attribute(0, 1), Some(enum_attribute));
    assert_eq!(call_site_value.get_string_attribute(0, "my_key"), Some(string_attribute));

    call_site_value.remove_string_attribute(0, "my_key");

    assert_eq!(call_site_value.count_attributes(0), 1);

    call_site_value.remove_enum_attribute(0, 1);

    assert_eq!(call_site_value.count_attributes(0), 0);
    assert!(call_site_value.get_enum_attribute(0, 1).is_none());
    assert!(call_site_value.get_string_attribute(0, "my_key").is_none());
    assert_eq!(call_site_value.get_called_fn_value(), fn_value);

    call_site_value.set_param_alignment_attribute(0, 12);

    assert_eq!(call_site_value.count_attributes(0), 1);
    assert!(call_site_value.get_enum_attribute(0, 1).is_some());
}
