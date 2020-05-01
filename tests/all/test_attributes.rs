extern crate inkwell;

use self::inkwell::attributes::{Attribute, AttributeLoc};
use self::inkwell::context::Context;

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
}

#[test]
fn test_string_attributes() {
    let context = Context::create();
    let string_attribute = context.create_string_attribute("my_key_123", "my_val");

    assert!(!string_attribute.is_enum());
    assert!(string_attribute.is_string());

    assert_eq!(string_attribute.get_string_kind_id().to_str(), Ok("my_key_123"));
    assert_eq!(string_attribute.get_string_value().to_str(), Ok("my_val"));
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
    let entry_bb = context.append_basic_block(fn_value, "entry");
    let string_attribute = context.create_string_attribute("my_key", "my_val");
    let alignstack_attribute = Attribute::get_named_enum_kind_id("alignstack");
    let enum_attribute = context.create_enum_attribute(alignstack_attribute, 1);

    builder.position_at_end(entry_bb);
    builder.build_return(None);

    assert_eq!(fn_value.count_attributes(AttributeLoc::Return), 0);
    assert_eq!(fn_value.count_attributes(AttributeLoc::Param(0)), 0);

    fn_value.remove_string_attribute(AttributeLoc::Return, "my_key"); // Noop
    fn_value.remove_enum_attribute(AttributeLoc::Return, alignstack_attribute); // Noop

    // define align 1 "my_key"="my_val" void @my_fn()
    fn_value.add_attribute(AttributeLoc::Return, string_attribute);
    fn_value.add_attribute(AttributeLoc::Param(0), string_attribute); // Applied to 1st param
    fn_value.add_attribute(AttributeLoc::Return, enum_attribute);

    assert_eq!(fn_value.count_attributes(AttributeLoc::Return), 2);
    assert_eq!(fn_value.get_enum_attribute(AttributeLoc::Return, alignstack_attribute), Some(enum_attribute));
    assert_eq!(fn_value.get_string_attribute(AttributeLoc::Return, "my_key"), Some(string_attribute));

    fn_value.remove_string_attribute(AttributeLoc::Return, "my_key");

    assert_eq!(fn_value.count_attributes(AttributeLoc::Return), 1);

    fn_value.remove_enum_attribute(AttributeLoc::Return, alignstack_attribute);

    assert_eq!(fn_value.count_attributes(AttributeLoc::Function), 0);
    assert_eq!(fn_value.count_attributes(AttributeLoc::Return), 0);
    assert!(fn_value.get_enum_attribute(AttributeLoc::Return, alignstack_attribute).is_none());
    assert!(fn_value.get_string_attribute(AttributeLoc::Return, "my_key").is_none());
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
    let entry_bb = context.append_basic_block(fn_value, "entry");
    let string_attribute = context.create_string_attribute("my_key", "my_val");
    let alignstack_attribute = Attribute::get_named_enum_kind_id("alignstack");
    let align_attribute = Attribute::get_named_enum_kind_id("align");
    let enum_attribute = context.create_enum_attribute(alignstack_attribute, 1);

    builder.position_at_end(entry_bb);

    let call_site_value = builder.build_call(fn_value, &[i32_type.const_int(1, false).into()], "my_fn");

    builder.build_return(None);

    assert_eq!(call_site_value.count_arguments(), 1);
    assert_eq!(call_site_value.count_attributes(AttributeLoc::Return), 0);
    assert_eq!(call_site_value.count_attributes(AttributeLoc::Param(0)), 0);

    call_site_value.remove_string_attribute(AttributeLoc::Return, "my_key"); // Noop
    call_site_value.remove_enum_attribute(AttributeLoc::Return, alignstack_attribute); // Noop

    // define align 1 "my_key"="my_val" void @my_fn()
    call_site_value.add_attribute(AttributeLoc::Return, string_attribute);
    call_site_value.add_attribute(AttributeLoc::Param(0), string_attribute); // Applied to 1st param
    call_site_value.add_attribute(AttributeLoc::Return, enum_attribute);

    assert_eq!(call_site_value.count_attributes(AttributeLoc::Return), 2);
    assert_eq!(call_site_value.get_enum_attribute(AttributeLoc::Return, alignstack_attribute), Some(enum_attribute));
    assert_eq!(call_site_value.get_string_attribute(AttributeLoc::Return, "my_key"), Some(string_attribute));

    call_site_value.remove_string_attribute(AttributeLoc::Return, "my_key");

    assert_eq!(call_site_value.count_attributes(AttributeLoc::Return), 1);

    call_site_value.remove_enum_attribute(AttributeLoc::Return, alignstack_attribute);

    assert_eq!(call_site_value.count_attributes(AttributeLoc::Return), 0);
    assert!(call_site_value.get_enum_attribute(AttributeLoc::Return, alignstack_attribute).is_none());
    assert!(call_site_value.get_string_attribute(AttributeLoc::Return, "my_key").is_none());
    assert_eq!(call_site_value.get_called_fn_value(), fn_value);

    call_site_value.set_alignment_attribute(AttributeLoc::Return, 16);

    assert_eq!(call_site_value.count_attributes(AttributeLoc::Return), 1);
    assert!(call_site_value.get_enum_attribute(AttributeLoc::Return, align_attribute).is_some());
}
