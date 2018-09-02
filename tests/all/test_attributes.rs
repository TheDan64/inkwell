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
    assert_eq!(Attribute::get_named_enum_kind_id("sret"), 50);
    assert_eq!(Attribute::get_named_enum_kind_id("swifterror"), 51);
    assert_eq!(Attribute::get_named_enum_kind_id("swiftself"), 52);
    assert_eq!(Attribute::get_named_enum_kind_id("zeroext"), 55);

    // REVIEW: Does last mean there is another kind at 56 we're missing?
    // or is it the next "free" slot
    assert_eq!(Attribute::get_last_enum_kind_id(), 56);
}

#[test]
fn test_enum_attributes() {
    assert_eq!(Attribute::get_last_enum_kind_id(), 56);

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

    // This hasn't changed from 56, so it doesn't seem like
    // enum creation influences the kind ids
    assert_eq!(Attribute::get_last_enum_kind_id(), 56);
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
