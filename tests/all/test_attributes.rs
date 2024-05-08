use inkwell::attributes::{Attribute, AttributeLoc};
use inkwell::context::Context;
use inkwell::values::BasicValueEnum;
use inkwell::AddressSpace;

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

#[llvm_versions(12..)]
#[test]
fn test_type_attribute() {
    use inkwell::types::{AnyType, BasicType};
    use inkwell::AddressSpace;

    let context = Context::create();
    let kind_id = Attribute::get_named_enum_kind_id("sret");

    #[allow(deprecated)]
    let ptr_type = context.i32_type().ptr_type(AddressSpace::default());
    let any_types = [
        context.i32_type().as_any_type_enum(),
        context.f32_type().as_any_type_enum(),
        context.void_type().as_any_type_enum(),
        context.i32_type().vec_type(1).as_any_type_enum(),
        context.i32_type().array_type(1).as_any_type_enum(),
        context.i32_type().fn_type(&[], false).as_any_type_enum(),
        ptr_type.as_any_type_enum(),
        context
            .struct_type(&[context.i32_type().as_basic_type_enum()], false)
            .as_any_type_enum(),
    ];

    let different_type = context.i128_type().as_any_type_enum();
    assert!(!any_types.contains(&different_type));

    for any_type in &any_types {
        let type_attribute = context.create_type_attribute(kind_id, *any_type);
        assert!(type_attribute.is_type());
        assert!(!type_attribute.is_enum());
        assert!(!type_attribute.is_string());
        assert_eq!(type_attribute.get_type_value(), *any_type);
        assert_ne!(type_attribute.get_type_value(), different_type);
    }
}

#[test]
#[allow(deprecated)]
fn test_attributes_on_function_values() {
    let context = Context::create();
    let builder = context.create_builder();
    let module = context.create_module("my_mod");
    let i32_ptr_type = context.i32_type().ptr_type(AddressSpace::default());
    let fn_type = i32_ptr_type.fn_type(&[i32_ptr_type.into()], false);
    let fn_value = module.add_function("my_fn", fn_type, None);
    let entry_bb = context.append_basic_block(fn_value, "entry");
    let string_attribute = context.create_string_attribute("my_key", "my_val");

    let alwaysinline_attribute = Attribute::get_named_enum_kind_id("alwaysinline");
    let function_enum_attribute = context.create_enum_attribute(alwaysinline_attribute, 0);

    let inreg_attribute = Attribute::get_named_enum_kind_id("noalias");
    let return_enum_attribute = context.create_enum_attribute(inreg_attribute, 0);

    builder.position_at_end(entry_bb);
    let null: BasicValueEnum = i32_ptr_type.const_null().into();
    builder.build_return(Some(&null)).unwrap();

    assert_eq!(fn_value.count_attributes(AttributeLoc::Return), 0);
    assert_eq!(fn_value.count_attributes(AttributeLoc::Param(0)), 0);
    assert_eq!(fn_value.attributes(AttributeLoc::Return), vec![]);
    assert_eq!(fn_value.attributes(AttributeLoc::Param(0)), vec![]);

    fn_value.remove_string_attribute(AttributeLoc::Return, "my_key"); // Noop
    fn_value.remove_enum_attribute(AttributeLoc::Return, inreg_attribute); // Noop
    fn_value.remove_enum_attribute(AttributeLoc::Function, alwaysinline_attribute); // Noop

    // ; Function Attrs: alwaysinline
    // define inreg "my_key"="my_val" void @my_fn(i32 "my_key"="my_val" %0) #0 {
    // entry:
    //   ret void
    // }
    //
    // attributes #0 = { alwaysinline }
    fn_value.add_attribute(AttributeLoc::Return, string_attribute);
    fn_value.add_attribute(AttributeLoc::Param(0), string_attribute); // Applied to 1st param
    fn_value.add_attribute(AttributeLoc::Return, return_enum_attribute);
    fn_value.add_attribute(AttributeLoc::Function, function_enum_attribute);

    module.print_to_stderr();

    module.verify().unwrap();

    assert_eq!(fn_value.count_attributes(AttributeLoc::Return), 2);
    assert_eq!(
        fn_value.get_enum_attribute(AttributeLoc::Return, inreg_attribute),
        Some(return_enum_attribute)
    );
    assert_eq!(
        fn_value.get_string_attribute(AttributeLoc::Return, "my_key"),
        Some(string_attribute)
    );
    assert_eq!(
        fn_value.attributes(AttributeLoc::Return),
        vec![return_enum_attribute, string_attribute]
    );

    fn_value.remove_string_attribute(AttributeLoc::Return, "my_key");

    assert_eq!(fn_value.count_attributes(AttributeLoc::Return), 1);
    assert_eq!(fn_value.attributes(AttributeLoc::Return), vec![return_enum_attribute]);
    fn_value.remove_enum_attribute(AttributeLoc::Return, inreg_attribute);

    assert_eq!(fn_value.count_attributes(AttributeLoc::Function), 1);
    assert_eq!(
        fn_value.attributes(AttributeLoc::Function),
        vec![function_enum_attribute]
    );
    fn_value.remove_enum_attribute(AttributeLoc::Function, alwaysinline_attribute);

    assert_eq!(fn_value.count_attributes(AttributeLoc::Function), 0);
    assert_eq!(fn_value.count_attributes(AttributeLoc::Return), 0);
    assert!(fn_value
        .get_enum_attribute(AttributeLoc::Return, inreg_attribute)
        .is_none());
    assert!(fn_value.get_string_attribute(AttributeLoc::Return, "my_key").is_none());
    assert_eq!(fn_value.attributes(AttributeLoc::Function), vec![]);
    assert_eq!(fn_value.attributes(AttributeLoc::Return), vec![]);
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

    let call_site_value = builder
        .build_call(fn_value, &[i32_type.const_int(1, false).into()], "my_fn")
        .unwrap();

    builder.build_return(None).unwrap();

    assert_eq!(call_site_value.count_arguments(), 1);
    assert_eq!(call_site_value.count_attributes(AttributeLoc::Return), 0);
    assert_eq!(call_site_value.count_attributes(AttributeLoc::Param(0)), 0);
    assert_eq!(call_site_value.attributes(AttributeLoc::Return), vec![]);
    assert_eq!(call_site_value.attributes(AttributeLoc::Param(0)), vec![]);

    call_site_value.remove_string_attribute(AttributeLoc::Return, "my_key"); // Noop
    call_site_value.remove_enum_attribute(AttributeLoc::Return, alignstack_attribute); // Noop

    // define align 1 "my_key"="my_val" void @my_fn()
    call_site_value.add_attribute(AttributeLoc::Return, string_attribute);
    call_site_value.add_attribute(AttributeLoc::Param(0), string_attribute); // Applied to 1st param
    call_site_value.add_attribute(AttributeLoc::Return, enum_attribute);

    assert_eq!(call_site_value.count_attributes(AttributeLoc::Return), 2);
    assert_eq!(
        call_site_value.get_enum_attribute(AttributeLoc::Return, alignstack_attribute),
        Some(enum_attribute)
    );
    assert_eq!(
        call_site_value.get_string_attribute(AttributeLoc::Return, "my_key"),
        Some(string_attribute)
    );
    assert_eq!(
        call_site_value.attributes(AttributeLoc::Return),
        vec![enum_attribute, string_attribute]
    );

    call_site_value.remove_string_attribute(AttributeLoc::Return, "my_key");

    assert_eq!(call_site_value.count_attributes(AttributeLoc::Return), 1);
    assert_eq!(call_site_value.attributes(AttributeLoc::Return), vec![enum_attribute]);

    call_site_value.remove_enum_attribute(AttributeLoc::Return, alignstack_attribute);

    assert_eq!(call_site_value.count_attributes(AttributeLoc::Return), 0);
    assert_eq!(call_site_value.attributes(AttributeLoc::Return), vec![]);
    assert!(call_site_value
        .get_enum_attribute(AttributeLoc::Return, alignstack_attribute)
        .is_none());
    assert!(call_site_value
        .get_string_attribute(AttributeLoc::Return, "my_key")
        .is_none());
    assert_eq!(call_site_value.get_called_fn_value(), fn_value);

    call_site_value.set_alignment_attribute(AttributeLoc::Return, 16);

    assert_eq!(call_site_value.count_attributes(AttributeLoc::Return), 1);
    assert_eq!(call_site_value.attributes(AttributeLoc::Return).len(), 1);
    assert!(call_site_value
        .get_enum_attribute(AttributeLoc::Return, align_attribute)
        .is_some());
}
