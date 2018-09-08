extern crate inkwell;

use self::inkwell::attributes::Attribute;
use self::inkwell::context::Context;

use std::ffi::CString;

#[test]
fn test_enum_attribute_kinds() {
    // Does not exist:
    assert_eq!(Attribute::get_named_enum_kind_id("foo"), 0);
    assert_eq!(Attribute::get_named_enum_kind_id("bar"), 0);

    // TODO: Document these kinds of attributes:
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
    assert_eq!(Attribute::get_named_enum_kind_id("noduplicate"), 23);
    assert_eq!(Attribute::get_named_enum_kind_id("noimplicitfloat"), 24);
    assert_eq!(Attribute::get_named_enum_kind_id("noinline"), 25);
    assert_eq!(Attribute::get_named_enum_kind_id("norecurse"), 26);
    assert_eq!(Attribute::get_named_enum_kind_id("noredzone"), 27);
    assert_eq!(Attribute::get_named_enum_kind_id("noreturn"), 28);
    assert_eq!(Attribute::get_named_enum_kind_id("nounwind"), 29);
    assert_eq!(Attribute::get_named_enum_kind_id("nonlazybind"), 30);
    assert_eq!(Attribute::get_named_enum_kind_id("optsize"), 32);
    assert_eq!(Attribute::get_named_enum_kind_id("optnone"), 33);
    assert_eq!(Attribute::get_named_enum_kind_id("readnone"), 34);
    assert_eq!(Attribute::get_named_enum_kind_id("readonly"), 35);
    assert_eq!(Attribute::get_named_enum_kind_id("returns_twice"), 37);
    assert_eq!(Attribute::get_named_enum_kind_id("safestack"), 39);
    assert_eq!(Attribute::get_named_enum_kind_id("sanitize_address"), 40);

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
    assert_eq!(Attribute::get_named_enum_kind_id("nonnull"), 31);
    assert_eq!(Attribute::get_named_enum_kind_id("returned"), 36);
    assert_eq!(Attribute::get_named_enum_kind_id("signext"), 38);

    #[cfg(feature = "llvm4-0")]
    {
        // Function Attributes:
        // assert_eq!(Attribute::get_named_enum_kind_id("sanitize_hwaddress"), 41);
        assert_eq!(Attribute::get_named_enum_kind_id("sanitize_memory"), 41);
        // assert_eq!(Attribute::get_named_enum_kind_id("sanitize_thread"), 42);
        // assert_eq!(Attribute::get_named_enum_kind_id("speculatable"), 43);
        assert_eq!(Attribute::get_named_enum_kind_id("alignstack"), 43); // 43 vs 44
        assert_eq!(Attribute::get_named_enum_kind_id("ssp"), 44); // 44 vs 45
        assert_eq!(Attribute::get_named_enum_kind_id("sspreq"), 45); // 45 vs 46
        assert_eq!(Attribute::get_named_enum_kind_id("sspstrong"), 46); // 46 vs 47
        // assert_eq!(Attribute::get_named_enum_kind_id("strictfp"), 48);
        assert_eq!(Attribute::get_named_enum_kind_id("uwtable"), 50); // 50 vs 51 vs 53
        assert_eq!(Attribute::get_named_enum_kind_id("writeonly"), 51); // 51 vs 52 vs 54

        // Parameter Attributes:
        assert_eq!(Attribute::get_named_enum_kind_id("sret"), 47); // 47 vs 48 vs 50
        assert_eq!(Attribute::get_named_enum_kind_id("swifterror"), 48); // 48 vs 49 vs 51
        assert_eq!(Attribute::get_named_enum_kind_id("swiftself"), 49); // 49 vs 50 vs 52
        assert_eq!(Attribute::get_named_enum_kind_id("zeroext"), 52); // 52 vs 53 vs 55

        // REVIEW: Does last mean there is another kind at 53 we're missing?
        // or is it the next "free" slot
        assert_eq!(Attribute::get_last_enum_kind_id(), 53);
    }

    #[cfg(feature = "llvm5-0")]
    {
        // Function Attributes:
        // assert_eq!(Attribute::get_named_enum_kind_id("sanitize_hwaddress"), 41);
        assert_eq!(Attribute::get_named_enum_kind_id("sanitize_memory"), 41);
        assert_eq!(Attribute::get_named_enum_kind_id("sanitize_thread"), 42);
        assert_eq!(Attribute::get_named_enum_kind_id("speculatable"), 43);
        assert_eq!(Attribute::get_named_enum_kind_id("alignstack"), 44);
        assert_eq!(Attribute::get_named_enum_kind_id("ssp"), 45);
        assert_eq!(Attribute::get_named_enum_kind_id("sspreq"), 46);
        assert_eq!(Attribute::get_named_enum_kind_id("sspstrong"), 47);
        // assert_eq!(Attribute::get_named_enum_kind_id("strictfp"), 48);
        assert_eq!(Attribute::get_named_enum_kind_id("uwtable"), 51); // 51 vs 53
        assert_eq!(Attribute::get_named_enum_kind_id("writeonly"), 52); // 52 vs 54

        // Parameter Attributes:
        assert_eq!(Attribute::get_named_enum_kind_id("sret"), 48); // 48 vs 50
        assert_eq!(Attribute::get_named_enum_kind_id("swifterror"), 49); // 49 vs 51
        assert_eq!(Attribute::get_named_enum_kind_id("swiftself"), 50); // 50 vs 52
        assert_eq!(Attribute::get_named_enum_kind_id("zeroext"), 53); // 53 vs 55

        // REVIEW: Does last mean there is another kind at 54 we're missing?
        // or is it the next "free" slot
        assert_eq!(Attribute::get_last_enum_kind_id(), 54);
    }

    #[cfg(not(any(feature = "llvm4-0", feature = "llvm5-0")))]
    {
        // Function Attributes:
        assert_eq!(Attribute::get_named_enum_kind_id("sanitize_hwaddress"), 41);
        assert_eq!(Attribute::get_named_enum_kind_id("sanitize_memory"), 42);
        assert_eq!(Attribute::get_named_enum_kind_id("sanitize_thread"), 43);
        assert_eq!(Attribute::get_named_enum_kind_id("speculatable"), 44);
        assert_eq!(Attribute::get_named_enum_kind_id("alignstack"), 45);
        assert_eq!(Attribute::get_named_enum_kind_id("ssp"), 46);
        assert_eq!(Attribute::get_named_enum_kind_id("sspreq"), 47);
        assert_eq!(Attribute::get_named_enum_kind_id("sspstrong"), 48);
        assert_eq!(Attribute::get_named_enum_kind_id("strictfp"), 49);
        assert_eq!(Attribute::get_named_enum_kind_id("uwtable"), 53);
        assert_eq!(Attribute::get_named_enum_kind_id("writeonly"), 54);

        // Parameter Attributes:
        assert_eq!(Attribute::get_named_enum_kind_id("sret"), 50);
        assert_eq!(Attribute::get_named_enum_kind_id("swifterror"), 51);
        assert_eq!(Attribute::get_named_enum_kind_id("swiftself"), 52);
        assert_eq!(Attribute::get_named_enum_kind_id("zeroext"), 55);

        // REVIEW: Does last mean there is another kind at 56 we're missing?
        // or is it the next "free" slot
        assert_eq!(Attribute::get_last_enum_kind_id(), 56);
    }
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
