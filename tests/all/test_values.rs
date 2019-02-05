extern crate inkwell;

use self::inkwell::{DLLStorageClass, FloatPredicate, GlobalVisibility, ThreadLocalMode, AddressSpace};
use self::inkwell::context::Context;
use self::inkwell::module::Linkage::*;
use self::inkwell::types::{StructType, VectorType};
use self::inkwell::values::{InstructionOpcode::*, MetadataValue, FIRST_CUSTOM_METADATA_KIND_ID, VectorValue};
#[llvm_versions(7.0 => latest)]
use self::inkwell::comdat::ComdatSelectionKind;

use std::ffi::CString;

// TODO: Test GlobalValues used as PointerValues

#[test]
fn test_linkage() {
    let context = Context::create();
    let module = context.create_module("testing");

    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);

    let function = module.add_function("free_f32", fn_type, None);

    assert_eq!(function.get_linkage(), External);
}

#[test]
fn test_call_site() {
    let context = Context::create();
    let module = context.create_module("testing");
    let builder = context.create_builder();

    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);

    let function = module.add_function("do_nothing", fn_type, None);

    let call_site = builder.build_call(function, &[], "to_infinity_and_beyond");

    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8")))]
    assert_eq!(call_site.count_arguments(), 0);
    assert!(!call_site.is_tail_call());

    call_site.set_tail_call(true);

    assert!(call_site.is_tail_call());

    call_site.set_tail_call(false);

    assert!(!call_site.is_tail_call());

    assert_eq!(call_site.get_call_convention(), 0);

    call_site.set_call_convention(2);

    assert_eq!(call_site.get_call_convention(), 2);

    call_site.set_param_alignment_attribute(0, 12);
}

#[test]
fn test_set_get_name() {
    let context = Context::create();
    let bool_type = context.bool_type();
    let i8_type = context.i8_type();
    let i16_type = context.i16_type();
    let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    let i128_type = context.i128_type();
    let f16_type = context.f16_type();
    let f32_type = context.f32_type();
    let f64_type = context.f64_type();
    let f128_type = context.f128_type();
    let array_type = f64_type.array_type(42);
    let ppc_f128_type = context.ppc_f128_type();

    let bool_val = bool_type.const_int(0, false);
    let i8_val = i8_type.const_int(0, false);
    let i16_val = i16_type.const_int(0, false);
    let i32_val = i32_type.const_int(0, false);
    let i64_val = i64_type.const_int(0, false);
    let i128_val = i128_type.const_int(0, false);
    let f16_val = f16_type.const_float(0.0);
    let f32_val = f32_type.const_float(0.0);
    let f64_val = f64_type.const_float(0.0);
    let f128_val = f128_type.const_float(0.0);
    let ptr_val = bool_type.ptr_type(AddressSpace::Generic).const_null();
    let array_val = f64_type.const_array(&[f64_val]);
    let struct_val = context.const_struct(&[i8_val.into(), f128_val.into()], false);
    let vec_val = VectorType::const_vector(&[i8_val]);
    let ppc_f128_val = ppc_f128_type.const_float(0.0);

    assert_eq!(bool_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(i8_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(i16_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(i32_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(i64_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(i128_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(f16_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(f32_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(f64_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(f128_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(ptr_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(array_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(struct_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(vec_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(ppc_f128_val.get_name(), &*CString::new("").unwrap());

    // LLVM Gem: You can't set names on constant values, so this doesn't do anything:
    bool_val.set_name("my_val");
    i8_val.set_name("my_val2");
    i16_val.set_name("my_val3");
    i32_val.set_name("my_val4");
    i64_val.set_name("my_val5");
    i128_val.set_name("my_val6");
    f16_val.set_name("my_val7");
    f32_val.set_name("my_val8");
    f64_val.set_name("my_val9");
    f128_val.set_name("my_val10");
    ptr_val.set_name("my_val11");
    array_val.set_name("my_val12");
    struct_val.set_name("my_val13");
    vec_val.set_name("my_val14");
    ppc_f128_val.set_name("my_val14");

    assert_eq!(bool_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(i8_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(i16_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(i32_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(i64_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(i128_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(f16_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(f32_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(f64_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(f128_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(ptr_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(array_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(struct_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(vec_val.get_name(), &*CString::new("").unwrap());
    assert_eq!(ppc_f128_val.get_name(), &*CString::new("").unwrap());

    let void_type = context.void_type();
    let ptr_type = bool_type.ptr_type(AddressSpace::Generic);
    let struct_type = context.struct_type(&[bool_type.into()], false);
    let vec_type = bool_type.vec_type(1);

    let module = context.create_module("types");
    let builder = context.create_builder();

    // You can set names on variables, though:
    let fn_type_params = [
        bool_type.into(),
        f32_type.into(),
        struct_type.into(),
        array_type.into(),
        ptr_type.into(),
        vec_type.into(),
    ];
    let fn_type = void_type.fn_type(&fn_type_params, false);

    let function = module.add_function("do_stuff", fn_type, None);
    let basic_block = context.append_basic_block(&function, "entry");

    builder.position_at_end(&basic_block);

    let int_param = function.get_nth_param(0).unwrap().into_int_value();
    let float_param = function.get_nth_param(1).unwrap().into_float_value();
    let struct_param = function.get_nth_param(2).unwrap().into_struct_value();
    let array_param = function.get_nth_param(3).unwrap().into_array_value();
    let ptr_param = function.get_nth_param(4).unwrap().into_pointer_value();
    let vec_param = function.get_nth_param(5).unwrap().into_vector_value();
    let phi_val = builder.build_phi(bool_type, "phi_node");

    assert_eq!(int_param.get_name(), &*CString::new("").unwrap());
    assert_eq!(float_param.get_name(), &*CString::new("").unwrap());
    assert_eq!(struct_param.get_name(), &*CString::new("").unwrap());
    assert_eq!(array_param.get_name(), &*CString::new("").unwrap());
    assert_eq!(ptr_param.get_name(), &*CString::new("").unwrap());
    assert_eq!(vec_param.get_name(), &*CString::new("").unwrap());
    assert_eq!(phi_val.get_name(), &*CString::new("phi_node").unwrap());

    int_param.set_name("my_val");
    float_param.set_name("my_val2");
    ptr_param.set_name("my_val3");
    array_param.set_name("my_val4");
    struct_param.set_name("my_val5");
    vec_param.set_name("my_val6");
    phi_val.set_name("phi");

    assert_eq!(int_param.get_name(), &*CString::new("my_val").unwrap());
    assert_eq!(float_param.get_name(), &*CString::new("my_val2").unwrap());
    assert_eq!(ptr_param.get_name(), &*CString::new("my_val3").unwrap());
    assert_eq!(array_param.get_name(), &*CString::new("my_val4").unwrap());
    assert_eq!(struct_param.get_name(), &*CString::new("my_val5").unwrap());
    assert_eq!(vec_param.get_name(), &*CString::new("my_val6").unwrap());
    assert_eq!(phi_val.get_name(), &*CString::new("phi").unwrap());

    // TODO: Test globals, supposedly constant globals work?
}

#[test]
fn test_undef() {
    let context = Context::create();
    let bool_type = context.bool_type();
    let i8_type = context.i8_type();
    let i16_type = context.i16_type();
    let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    let i128_type = context.i128_type();
    let f16_type = context.f16_type();
    let f32_type = context.f32_type();
    let f64_type = context.f64_type();
    let f128_type = context.f128_type();
    let array_type = f64_type.array_type(42);
    let ppc_f128_type = context.ppc_f128_type();

    assert_eq!(array_type.get_element_type().into_float_type(), f64_type);

    let bool_val = bool_type.const_int(0, false);
    let i8_val = i8_type.const_int(0, false);
    let i16_val = i16_type.const_int(0, false);
    let i32_val = i32_type.const_int(0, false);
    let i64_val = i64_type.const_int(0, false);
    let i128_val = i128_type.const_int(0, false);
    let f16_val = f16_type.const_float(0.0);
    let f32_val = f32_type.const_float(0.0);
    let f64_val = f64_type.const_float(0.0);
    let f128_val = f128_type.const_float(0.0);
    let ptr_val = bool_type.ptr_type(AddressSpace::Generic).const_null();
    let array_val = f64_type.const_array(&[f64_val]);
    let struct_val = context.const_struct(&[i8_val.into(), f128_val.into()], false);
    let vec_val = VectorType::const_vector(&[i8_val]);
    let ppc_f128_val = ppc_f128_type.const_float(0.0);

    assert!(!bool_val.is_undef());
    assert!(!i8_val.is_undef());
    assert!(!i16_val.is_undef());
    assert!(!i32_val.is_undef());
    assert!(!i64_val.is_undef());
    assert!(!i128_val.is_undef());
    assert!(!f16_val.is_undef());
    assert!(!f32_val.is_undef());
    assert!(!f64_val.is_undef());
    assert!(!f128_val.is_undef());
    assert!(!ptr_val.is_undef());
    assert!(!array_val.is_undef());
    assert!(!struct_val.is_undef());
    assert!(!vec_val.is_undef());
    assert!(!ppc_f128_val.is_undef());

    let bool_undef = bool_type.get_undef();
    let i8_undef = i8_type.get_undef();
    let i16_undef = i16_type.get_undef();
    let i32_undef = i32_type.get_undef();
    let i64_undef = i64_type.get_undef();
    let i128_undef = i128_type.get_undef();
    let f16_undef = f16_type.get_undef();
    let f32_undef = f32_type.get_undef();
    let f64_undef = f64_type.get_undef();
    let f128_undef = f128_type.get_undef();
    let ptr_undef = bool_type.ptr_type(AddressSpace::Generic).get_undef();
    let array_undef = array_type.get_undef();
    let struct_undef = StructType::struct_type(&[bool_type.into()], false).get_undef();
    let vec_undef = bool_type.vec_type(1).get_undef();
    let ppc_f128_undef = ppc_f128_type.get_undef();

    assert!(bool_undef.is_undef());
    assert!(i8_undef.is_undef());
    assert!(i16_undef.is_undef());
    assert!(i32_undef.is_undef());
    assert!(i64_undef.is_undef());
    assert!(i128_undef.is_undef());
    assert!(f16_undef.is_undef());
    assert!(f32_undef.is_undef());
    assert!(f64_undef.is_undef());
    assert!(f128_undef.is_undef());
    assert!(ptr_undef.is_undef());
    assert!(array_undef.is_undef());
    assert!(struct_undef.is_undef());
    assert!(vec_undef.is_undef());
    assert!(ppc_f128_undef.is_undef());
}

#[test]
fn test_consecutive_fns() {
    let context = Context::create();
    let module = context.create_module("fns");

    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);

    let function = module.add_function("fn", fn_type, None);

    assert!(function.get_previous_function().is_none());
    assert!(function.get_next_function().is_none());

    let function2 = module.add_function("fn2", fn_type, None);

    assert_ne!(function, function2);

    assert!(function.get_previous_function().is_none());
    assert_eq!(function.get_next_function().unwrap(), function2);

    assert_eq!(function2.get_previous_function().unwrap(), function);
    assert!(function2.get_next_function().is_none());
}

#[test]
fn test_verify_fn() {
    let context = Context::create();
    let builder = context.create_builder();
    let module = context.create_module("fns");

    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);

    let function = module.add_function("fn", fn_type, None);

    #[cfg(not(any(feature = "llvm3-9", feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0", feature = "llvm7-0")))]
    assert!(!function.verify(false));
    // REVIEW: Why does 3.9 -> 7.0 return true here? LLVM bug? Bugfix?
    #[cfg(any(feature = "llvm3-9", feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0", feature = "llvm7-0"))]
    assert!(function.verify(false));

    let basic_block = context.append_basic_block(&function, "entry");

    builder.position_at_end(&basic_block);
    builder.build_return(None);

    assert!(function.verify(false));

    // TODO: Verify other verify modes
}

#[test]
fn test_metadata() {
    let context = Context::create();
    let module = context.create_module("my_mod");

    // TODOC: From looking at the source, seem to be a bunch of predefined ones
    // and then afterwards it assigns a new value for newly requested keys.
    // REVIEW: So Maybe we could/should change the above to an enum input:
    // enum {Debug, Tbaa, ..., Custom(Cow)} which evaluates to the predefined value
    // or a new lookup

    assert_eq!(context.get_kind_id("foo"), FIRST_CUSTOM_METADATA_KIND_ID);
    assert_eq!(MetadataValue::get_kind_id("foo"), FIRST_CUSTOM_METADATA_KIND_ID);
    assert_eq!(context.get_kind_id("bar"), FIRST_CUSTOM_METADATA_KIND_ID + 1);
    assert_eq!(MetadataValue::get_kind_id("bar"), FIRST_CUSTOM_METADATA_KIND_ID + 1);

    // Predefined
    assert_eq!(context.get_kind_id("dbg"), 0);
    assert_eq!(MetadataValue::get_kind_id("dbg"), 0);
    assert_eq!(context.get_kind_id("tbaa"), 1);
    assert_eq!(MetadataValue::get_kind_id("tbaa"), 1);
    assert_eq!(context.get_kind_id("prof"), 2);
    assert_eq!(MetadataValue::get_kind_id("prof"), 2);
    assert_eq!(context.get_kind_id("fpmath"), 3);
    assert_eq!(MetadataValue::get_kind_id("fpmath"), 3);
    assert_eq!(context.get_kind_id("range"), 4);
    assert_eq!(MetadataValue::get_kind_id("range"), 4);
    assert_eq!(context.get_kind_id("tbaa.struct"), 5);
    assert_eq!(MetadataValue::get_kind_id("tbaa.struct"), 5);
    assert_eq!(context.get_kind_id("invariant.load"), 6);
    assert_eq!(MetadataValue::get_kind_id("invariant.load"), 6);
    assert_eq!(context.get_kind_id("alias.scope"), 7);
    assert_eq!(MetadataValue::get_kind_id("alias.scope"), 7);
    assert_eq!(context.get_kind_id("noalias"), 8);
    assert_eq!(MetadataValue::get_kind_id("noalias"), 8);
    assert_eq!(context.get_kind_id("nontemporal"), 9);
    assert_eq!(MetadataValue::get_kind_id("nontemporal"), 9);
    assert_eq!(context.get_kind_id("llvm.mem.parallel_loop_access"), 10);
    assert_eq!(MetadataValue::get_kind_id("llvm.mem.parallel_loop_access"), 10);
    assert_eq!(context.get_kind_id("nonnull"), 11);
    assert_eq!(MetadataValue::get_kind_id("nonnull"), 11);

    #[cfg(not(feature = "llvm3-6"))]
    {
        assert_eq!(context.get_kind_id("dereferenceable"), 12);
        assert_eq!(MetadataValue::get_kind_id("dereferenceable"), 12);
        assert_eq!(context.get_kind_id("dereferenceable_or_null"), 13);
        assert_eq!(MetadataValue::get_kind_id("dereferenceable_or_null"), 13);
    }

    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7")))]
    {
        assert_eq!(context.get_kind_id("make.implicit"), 14);
        assert_eq!(MetadataValue::get_kind_id("make.implicit"), 14);
        assert_eq!(context.get_kind_id("unpredictable"), 15);
        assert_eq!(MetadataValue::get_kind_id("unpredictable"), 15);
        assert_eq!(context.get_kind_id("invariant.group"), 16);
        assert_eq!(MetadataValue::get_kind_id("invariant.group"), 16);
        assert_eq!(context.get_kind_id("align"), 17);
        assert_eq!(MetadataValue::get_kind_id("align"), 17);
    }

    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8")))]
    {
        assert_eq!(context.get_kind_id("llvm.loop"), 18);
        assert_eq!(MetadataValue::get_kind_id("llvm.loop"), 18);
        assert_eq!(context.get_kind_id("type"), 19);
        assert_eq!(MetadataValue::get_kind_id("type"), 19);
    }

    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8", feature = "llvm3-9")))]
    {
        assert_eq!(context.get_kind_id("section_prefix"), 20);
        assert_eq!(MetadataValue::get_kind_id("section_prefix"), 20);
        assert_eq!(context.get_kind_id("absolute_symbol"), 21);
        assert_eq!(MetadataValue::get_kind_id("absolute_symbol"), 21);
    }

    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8", feature = "llvm3-9", feature = "llvm4-0")))]
    {
        assert_eq!(context.get_kind_id("associated"), 22);
        assert_eq!(MetadataValue::get_kind_id("associated"), 22);
    }

    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8", feature = "llvm3-9", feature = "llvm4-0", feature = "llvm5-0")))]
    {
        assert_eq!(context.get_kind_id("callees"), 23);
        assert_eq!(MetadataValue::get_kind_id("callees"), 23);
        assert_eq!(context.get_kind_id("irr_loop"), 24);
        assert_eq!(MetadataValue::get_kind_id("irr_loop"), 24);
    }

    // TODO: 7+?

    assert_eq!(module.get_global_metadata_size("my_string_md"), 0);
    assert_eq!(module.get_global_metadata("my_string_md").len(), 0);

    let md_string = MetadataValue::create_string("lots of metadata here");

    assert_eq!(md_string.get_node_size(), 0);
    assert_eq!(md_string.get_node_values().len(), 0);
    assert_eq!(md_string.get_string_value().unwrap(), &*CString::new("lots of metadata here").unwrap());

    let bool_type = context.bool_type();
    let i8_type = context.i8_type();
    // let i16_type = context.i16_type();
    // let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    // let i128_type = context.i128_type();
    // let f16_type = context.f16_type();
    let f32_type = context.f32_type();
    let f64_type = context.f64_type();
    let f128_type = context.f128_type();
    let array_type = f64_type.array_type(42);
    // let ppc_f128_type = context.ppc_f128_type();
    let fn_type = bool_type.fn_type(&[i64_type.into(), array_type.into()], false);

    let bool_val = bool_type.const_int(0, false);
    let i8_val = i8_type.const_int(0, false);
    // let i16_val = i16_type.const_int(0, false);
    // let i32_val = i32_type.const_int(0, false);
    // let i64_val = i64_type.const_int(0, false);
    // let i128_val = i128_type.const_int(0, false);
    // let f16_val = f16_type.const_float(0.0);
    let f32_val = f32_type.const_float(0.0);
    let f64_val = f64_type.const_float(0.0);
    let f128_val = f128_type.const_float(0.0);
    // let ppc_f128_val = ppc_f128_type.const_float(0.0);
    let ptr_val = bool_type.ptr_type(AddressSpace::Generic).const_null();
    let array_val = f64_type.const_array(&[f64_val]);
    let struct_val = context.const_struct(&[i8_val.into(), f128_val.into()], false);
    let vec_val = VectorType::const_vector(&[i8_val]);
    let fn_val = module.add_function("my_fn", fn_type, None);

    let md_node = MetadataValue::create_node(&[&bool_val, &f32_val]);

    let node_values = md_node.get_node_values();

    assert_eq!(md_node.get_string_value(), None);
    assert_eq!(node_values.len(), 2);
    assert_eq!(node_values[0].as_int_value(), &bool_val);
    assert_eq!(node_values[1].as_float_value(), &f32_val);

    module.add_global_metadata("my_md", &md_string);
    module.add_global_metadata("my_md", &md_node);

    assert_eq!(module.get_global_metadata_size("my_md"), 2);

    let global_md = module.get_global_metadata("my_md");

    assert_eq!(global_md.len(), 2);

    let (md_0, md_1) = (global_md[0].get_node_values(), global_md[1].get_node_values());

    assert_eq!(md_0.len(), 1);
    assert_eq!(md_1.len(), 2);
    assert_eq!(md_0[0].as_metadata_value().get_string_value(), md_string.get_string_value());
    assert_eq!(md_1[0].as_int_value(), &bool_val);
    assert_eq!(md_1[1].as_float_value(), &f32_val);

    assert_eq!(module.get_global_metadata_size("other_md"), 0);

    // REVIEW: const_null_ptr/ ptr.const_null seem to cause UB. Need to test and adapt
    // and see if they should be allowed to have metadata? Also, while we're at it we should
    // try with undef

    // REVIEW: initial has_metadata seems inconsistent. Some have it. Some don't for kind_id 0. Some sometimes have it.
    // furthermore, when they do have it, it is a SF when printing out. Unclear what can be done here. Maybe just disallow index 0?
    // assert!(bool_val.has_metadata());
    // assert!(i8_val.has_metadata());
    // assert!(i16_val.has_metadata());
    // assert!(i32_val.has_metadata());
    // assert!(i64_val.has_metadata());
    // assert!(!i128_val.has_metadata());
    // assert!(!f16_val.has_metadata());
    // assert!(!f32_val.has_metadata());
    // assert!(!f64_val.has_metadata());
    // assert!(!f128_val.has_metadata());
    // assert!(!ppc_f128_val.has_metadata());
    // assert!(ptr_val.has_metadata());
    // assert!(array_val.has_metadata());
    // assert!(struct_val.has_metadata());
    // assert!(!vec_val.has_metadata());
    // assert!(!fn_val.has_metadata());

    bool_val.set_metadata(md_string, 3);

    assert!(bool_val.has_metadata());
    assert!(bool_val.get_metadata(1).is_none());
    assert!(bool_val.get_metadata(2).is_none());

    let md_node_values = bool_val.get_metadata(3).unwrap().get_node_values();

    assert_eq!(md_node_values.len(), 1);
    assert_eq!(md_node_values[0].as_metadata_value().get_string_value(), md_string.get_string_value());

    f128_val.set_metadata(md_node, 3);

    assert!(f128_val.has_metadata());
    assert!(f128_val.get_metadata(1).is_none());
    assert!(f128_val.get_metadata(2).is_none());

    let md_node_values = f128_val.get_metadata(3).unwrap().get_node_values();

    assert_eq!(md_node_values.len(), 2);
    assert_eq!(md_node_values[0].as_int_value(), &bool_val);
    assert_eq!(md_node_values[1].as_float_value(), &f32_val);

    array_val.set_metadata(md_string, 2);

    assert!(array_val.has_metadata());
    assert!(array_val.get_metadata(1).is_none());

    let md_node_values = array_val.get_metadata(2).unwrap().get_node_values();

    assert_eq!(md_node_values.len(), 1);
    assert_eq!(md_node_values[0].as_metadata_value().get_string_value(), md_string.get_string_value());

    struct_val.set_metadata(md_node, 4);

    assert!(struct_val.has_metadata());
    assert!(struct_val.get_metadata(1).is_none());
    assert!(struct_val.get_metadata(2).is_none());
    assert!(struct_val.get_metadata(3).is_none());

    let md_node_values = struct_val.get_metadata(4).unwrap().get_node_values();

    assert_eq!(md_node_values.len(), 2);
    assert_eq!(md_node_values[0].as_int_value(), &bool_val);
    assert_eq!(md_node_values[1].as_float_value(), &f32_val);

    vec_val.set_metadata(md_string, 1);

    assert!(vec_val.has_metadata());

    let md_node_values = vec_val.get_metadata(1).unwrap().get_node_values();

    assert_eq!(md_node_values.len(), 1);
    assert_eq!(md_node_values[0].as_metadata_value().get_string_value(), md_string.get_string_value());

    fn_val.set_metadata(md_node, 4);

    assert!(fn_val.has_metadata());
    assert!(fn_val.get_metadata(1).is_none());
    assert!(fn_val.get_metadata(2).is_none());
    assert!(fn_val.get_metadata(3).is_none());

    let md_node_values = fn_val.get_metadata(4).unwrap().get_node_values();

    assert_eq!(md_node_values.len(), 2);
    assert_eq!(md_node_values[0].as_int_value(), &bool_val);
    assert_eq!(md_node_values[1].as_float_value(), &f32_val);

    ptr_val.set_metadata(md_string, 1);

    assert!(ptr_val.has_metadata());

    let md_node_values = ptr_val.get_metadata(1).unwrap().get_node_values();

    assert_eq!(md_node_values.len(), 1);
    assert_eq!(md_node_values[0].as_metadata_value().get_string_value(), md_string.get_string_value());

    // New Context Metadata
    let context_metadata_node = context.metadata_node(&[bool_val.into(), f32_val.into()]);
    let context_metadata_string = context.metadata_string("my_context_metadata");

    assert!(context_metadata_node.is_node());
    assert!(context_metadata_string.is_string());
}

#[test]
fn test_floats() {
    let context = Context::create();

    let f32_type = context.f32_type();
    let f64_type = context.f64_type();
    let f128_type = context.f128_type();
    let i64_type = context.i32_type();

    let f64_pi = f64_type.const_float(::std::f64::consts::PI);

    let f32_pi = f64_pi.const_truncate(f32_type);
    let f128_pi = f64_pi.const_extend(f128_type);
    let i64_pi = f64_pi.const_to_signed_int(i64_type);
    let u64_pi = f64_pi.const_to_unsigned_int(i64_type);
    let f128_pi_cast = f64_pi.const_cast(f128_type);

    assert_eq!(i64_pi.get_type(), i64_type);
    assert_eq!(u64_pi.get_type(), i64_type);
    assert_eq!(f32_pi.get_type(), f32_type);
    assert_eq!(f128_pi.get_type(), f128_type);
    assert_eq!(f128_pi_cast.get_type(), f128_type);

    // REIVEW: Why are these not FPTrunc, FPExt, FPToSI, FPToUI, BitCast instructions?
    // Only thing I can think of is that they're constants and therefore precalculated
    assert!(f32_pi.as_instruction().is_none());
    assert!(f128_pi.as_instruction().is_none());
    assert!(i64_pi.as_instruction().is_none());
    assert!(u64_pi.as_instruction().is_none());
    assert!(f128_pi_cast.as_instruction().is_none());

    let f64_one = f64_type.const_float(1.);
    let f64_two = f64_type.const_float(2.);
    let neg_two = f64_two.const_neg();

    assert_eq!(*neg_two.print_to_string(), *CString::new("double -2.000000e+00").unwrap());

    let neg_three = neg_two.const_sub(f64_one);

    assert_eq!(*neg_three.print_to_string(), *CString::new("double -3.000000e+00").unwrap());

    let pos_six = neg_three.const_mul(neg_two);

    assert_eq!(*pos_six.print_to_string(), *CString::new("double 6.000000e+00").unwrap());

    let pos_eight = pos_six.const_add(f64_two);

    assert_eq!(*pos_eight.print_to_string(), *CString::new("double 8.000000e+00").unwrap());

    let pos_four = pos_eight.const_div(f64_two);

    assert_eq!(*pos_four.print_to_string(), *CString::new("double 4.000000e+00").unwrap());

    let rem = pos_six.const_remainder(pos_four);

    assert_eq!(*rem.print_to_string(), *CString::new("double 2.000000e+00").unwrap());

    assert!(f64_one.const_compare(FloatPredicate::PredicateFalse, f64_two).is_null());
    assert!(!f64_one.const_compare(FloatPredicate::PredicateTrue, f64_two).is_null());
    assert!(f64_one.const_compare(FloatPredicate::OEQ, f64_two).is_null());
    assert!(f64_one.const_compare(FloatPredicate::OGT, f64_two).is_null());
    assert!(f64_one.const_compare(FloatPredicate::OGE, f64_two).is_null());
    assert!(!f64_one.const_compare(FloatPredicate::OLT, f64_two).is_null());
    assert!(!f64_one.const_compare(FloatPredicate::OLE, f64_two).is_null());
    assert!(!f64_one.const_compare(FloatPredicate::ONE, f64_two).is_null());
    assert!(f64_one.const_compare(FloatPredicate::UEQ, f64_two).is_null());
    assert!(f64_one.const_compare(FloatPredicate::UGT, f64_two).is_null());
    assert!(f64_one.const_compare(FloatPredicate::UGE, f64_two).is_null());
    assert!(!f64_one.const_compare(FloatPredicate::ULT, f64_two).is_null());
    assert!(!f64_one.const_compare(FloatPredicate::ULE, f64_two).is_null());
    assert!(!f64_one.const_compare(FloatPredicate::UNE, f64_two).is_null());
    assert!(!f64_one.const_compare(FloatPredicate::ORD, f64_two).is_null());
    assert!(f64_one.const_compare(FloatPredicate::UNO, f64_two).is_null());
}

#[test]
fn test_function_value_no_params() {
    let context = Context::create();
    let module = context.create_module("my_mod");
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let fn_value = module.add_function("no_params", fn_type, None);

    // REVIEW: According to this, fn_value.get_type() is a void ptr??
    // assert_eq!(fn_value.get_type(), fn_type);
    assert_eq!(fn_value.count_params(), 0);
    assert_eq!(fn_value.get_param_iter().collect::<Vec<_>>().len(), 0);
    assert_eq!(fn_value.get_params().len(), 0);
    assert!(fn_value.get_first_param().is_none());
    assert!(fn_value.get_last_param().is_none());
    assert!(fn_value.get_nth_param(0).is_none());

    // Here we're able to avoid a segfault in every version except 3.8 :(
    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-8")))]
    assert!(fn_value.get_personality_function().is_none());
    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8")))]
    assert!(!fn_value.has_personality_function());
    assert!(!fn_value.is_null());
    assert!(!fn_value.is_undef());
}

#[test]
fn test_value_from_string() {
    let context = Context::create();
    let i8_type = context.i8_type();
    let i8_val = i8_type.const_int_from_string("0121", 10);

    assert_eq!(*i8_val.print_to_string(), *CString::new("i8 121").unwrap());

    let i8_val = i8_type.const_int_from_string("0121", 3);

    assert_eq!(*i8_val.print_to_string(), *CString::new("i8 16").unwrap());

    // LLVM will not throw an error, just parse until it can parse no more (and
    // possibly spit out something completely unexpected):
    let i8_val = i8_type.const_int_from_string("0121", 2);

    assert_eq!(*i8_val.print_to_string(), *CString::new("i8 3").unwrap());

    let i8_val = i8_type.const_int_from_string("ABCD", 2);

    assert_eq!(*i8_val.print_to_string(), *CString::new("i8 -15").unwrap());

    // Floats
    let f64_type = context.f64_type();
    let f64_val = f64_type.const_float_from_string("3.6");

    assert_eq!(f64_val.print_to_string().to_string(), "double 3.600000e+00");

    let f64_val = f64_type.const_float_from_string("3.");

    assert_eq!(f64_val.print_to_string().to_string(), "double 3.000000e+00");

    let f64_val = f64_type.const_float_from_string("3");

    assert_eq!(f64_val.print_to_string().to_string(), "double 3.000000e+00");

    let f64_val = f64_type.const_float_from_string("");

    assert_eq!(f64_val.print_to_string().to_string(), "double 0.000000e+00");

    let f64_val = f64_type.const_float_from_string("3.asd");

    assert_eq!(f64_val.print_to_string().to_string(), "double 0x7FF0000000000000");
}

#[test]
fn test_value_copies() {
    let context = Context::create();
    let i8_type = context.i8_type();

    let i8_value = i8_type.const_int(12, false);
    let i8_value_copy = i8_value;

    assert_eq!(i8_value, i8_value_copy);
}

#[test]
fn test_global_byte_array() {
    let context = Context::create();
    let module = context.create_module("my_mod");
    let my_str = "Hello, World";
    let i8_type = context.i8_type();
    let i8_array_type = i8_type.array_type(my_str.len() as u32);
    let global_string = module.add_global(i8_array_type, Some(AddressSpace::Generic), "message");

    let mut chars = Vec::with_capacity(my_str.len());

    for chr in my_str.bytes() {
        chars.push(i8_type.const_int(chr as u64, false));
    }

    let const_str_array = i8_type.const_array(chars.as_ref());

    global_string.set_initializer(&const_str_array);

    assert!(module.verify().is_ok());
}

#[test]
fn test_globals() {
    #[llvm_versions(7.0 => latest)]
    use self::inkwell::values::UnnamedAddress;

    let context = Context::create();
    let module = context.create_module("my_mod");
    let i8_type = context.i8_type();
    let i8_zero = i8_type.const_int(0, false);

    assert!(module.get_first_global().is_none());
    assert!(module.get_last_global().is_none());
    assert!(module.get_global("my_global").is_none());

    let global = module.add_global(i8_type, None, "my_global");

    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8", feature = "llvm3-9",
                  feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0")))]
    assert_eq!(global.get_unnamed_address(), UnnamedAddress::None);
    assert!(global.get_previous_global().is_none());
    assert!(global.get_next_global().is_none());
    assert!(global.get_initializer().is_none());
    assert!(!global.is_thread_local());
    assert!(global.get_thread_local_mode().is_none());
    assert!(!global.is_constant());
    assert!(global.is_declaration());
    assert!(!global.has_unnamed_addr());
    assert!(!global.is_externally_initialized());
    // REVIEW: Segfaults in 4.0 -> 7.0
    #[cfg(not(any(feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0", feature = "llvm7-0")))]
    assert_eq!(global.get_section(), &*CString::new("").unwrap());
    assert_eq!(global.get_dll_storage_class(), DLLStorageClass::default());
    assert_eq!(global.get_visibility(), GlobalVisibility::default());
    assert_eq!(module.get_first_global().unwrap(), global);
    assert_eq!(module.get_last_global().unwrap(), global);
    assert_eq!(module.get_global("my_global").unwrap(), global);

    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8", feature = "llvm3-9",
                  feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0")))]
    global.set_unnamed_address(UnnamedAddress::Local);
    global.set_dll_storage_class(DLLStorageClass::Import);
    global.set_initializer(&i8_zero);
    global.set_thread_local_mode(Some(ThreadLocalMode::InitialExecTLSModel));
    global.set_unnamed_addr(true);
    global.set_constant(true);
    global.set_visibility(GlobalVisibility::Hidden);
    global.set_section("not sure what goes here");

    // REVIEW: Not sure why this is Global when we set it to Local
    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8", feature = "llvm3-9",
                  feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0")))]
    assert_eq!(global.get_unnamed_address(), UnnamedAddress::Global);
    assert_eq!(global.get_dll_storage_class(), DLLStorageClass::Import);
    assert_eq!(global.get_initializer().unwrap().into_int_value(), i8_zero);
    assert_eq!(global.get_visibility(), GlobalVisibility::Hidden);
    assert_eq!(global.get_thread_local_mode().unwrap(), ThreadLocalMode::InitialExecTLSModel);
    assert!(global.is_thread_local());
    assert!(global.has_unnamed_addr());
    assert!(global.is_constant());
    assert!(!global.is_declaration());
    assert_eq!(global.get_section(), &*CString::new("not sure what goes here").unwrap());

    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8", feature = "llvm3-9",
                  feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0")))]
    global.set_unnamed_address(UnnamedAddress::Global);
    global.set_dll_storage_class(DLLStorageClass::Export);
    global.set_thread_local(false);
    global.set_visibility(GlobalVisibility::Protected);

    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8", feature = "llvm3-9",
                  feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0")))]
    assert_eq!(global.get_unnamed_address(), UnnamedAddress::Global);
    assert!(!global.is_thread_local());
    assert_eq!(global.get_visibility(), GlobalVisibility::Protected);

    global.set_thread_local(true);

    assert_eq!(global.get_dll_storage_class(), DLLStorageClass::Export);
    assert!(global.is_thread_local());
    assert_eq!(global.get_thread_local_mode().unwrap(), ThreadLocalMode::GeneralDynamicTLSModel);

    global.set_thread_local_mode(Some(ThreadLocalMode::LocalExecTLSModel));

    assert_eq!(global.get_thread_local_mode().unwrap(), ThreadLocalMode::LocalExecTLSModel);

    global.set_thread_local_mode(Some(ThreadLocalMode::LocalDynamicTLSModel));

    assert_eq!(global.get_thread_local_mode().unwrap(), ThreadLocalMode::LocalDynamicTLSModel);

    global.set_thread_local_mode(None);

    assert!(global.get_thread_local_mode().is_none());

    let global2 = module.add_global(i8_type, Some(AddressSpace::Const), "my_global2");

    assert_eq!(global2.get_previous_global().unwrap(), global);
    assert_eq!(global.get_next_global().unwrap(), global2);
    assert_eq!(module.get_first_global().unwrap(), global);
    assert_eq!(module.get_last_global().unwrap(), global2);
    assert_eq!(module.get_global("my_global2").unwrap(), global2);
    assert!(!global.is_declaration());
    assert!(!global.is_externally_initialized());
    assert_eq!(global.get_alignment(), 0);

    global.set_alignment(4);

    assert_eq!(global.get_alignment(), 4);

    global2.set_externally_initialized(true);

    // REVIEW: This doesn't seem to work. LLVM bug?
    assert!(global2.is_externally_initialized());

    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8", feature = "llvm3-9",
                  feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0")))]
    {
        assert!(global.get_comdat().is_none());

        let comdat = module.get_or_insert_comdat("my_comdat");

        assert!(global.get_comdat().is_none());

        global.set_comdat(comdat);

        assert_eq!(comdat, global.get_comdat().unwrap());
        assert_eq!(comdat.get_selection_kind(), ComdatSelectionKind::Any);

        comdat.set_selection_kind(ComdatSelectionKind::Largest);

        assert_eq!(comdat.get_selection_kind(), ComdatSelectionKind::Largest);
    }

    unsafe {
        global.delete();
    }
}

#[test]
fn test_phi_values() {
    let context = Context::create();
    let builder = context.create_builder();
    let module = context.create_module("my_mod");
    let void_type = context.void_type();
    let bool_type = context.bool_type();
    let fn_type = void_type.fn_type(&[bool_type.into()], false);
    let fn_value = module.add_function("my_func", fn_type, None);

    assert!(fn_value.as_global_value().is_declaration());

    let entry_block = fn_value.append_basic_block("entry");
    let then_block = fn_value.append_basic_block("then");
    let else_block = fn_value.append_basic_block("else");

    assert!(!fn_value.as_global_value().is_declaration());

    builder.position_at_end(&entry_block);

    let false_val = bool_type.const_int(0, false);
    let true_val = bool_type.const_int(1, false);
    let phi = builder.build_phi(bool_type, "if");

    assert!(!phi.is_null());
    assert!(!phi.is_undef());
    assert!(phi.as_basic_value().is_int_value());
    assert_eq!(phi.as_instruction().get_opcode(), Phi);
    assert_eq!(phi.count_incoming(), 0);
    assert_eq!(*phi.print_to_string(), *CString::new("  %if = phi i1 ").unwrap());

    phi.add_incoming(&[
        (&false_val, &then_block),
        (&true_val, &else_block),
    ]);

    assert_eq!(phi.count_incoming(), 2);
    assert_eq!(*phi.print_to_string(), *CString::new("  %if = phi i1 [ false, %then ], [ true, %else ]").unwrap());

    let (then_val, then_bb) = phi.get_incoming(0).unwrap();
    let (else_val, else_bb) = phi.get_incoming(1).unwrap();

    assert_eq!(then_val.into_int_value(), false_val);
    assert_eq!(else_val.into_int_value(), true_val);
    assert_eq!(then_bb, then_block);
    assert_eq!(else_bb, else_block);
    assert!(phi.get_incoming(2).is_none());
}

#[test]
fn test_allocations() {
    let context = Context::create();
    let builder = context.create_builder();
    let module = context.create_module("my_mod");
    let void_type = context.void_type();
    let i32_type = context.i32_type();
    let i32_three = i32_type.const_int(3, false);
    let fn_type = void_type.fn_type(&[], false);
    let fn_value = module.add_function("my_func", fn_type, None);
    let entry_block = fn_value.append_basic_block("entry");

    builder.position_at_end(&entry_block);

    // REVIEW: Alloca (and possibly malloc) seem to be prone to segfaulting
    // when called with a builder that isn't positioned. I wonder if other
    // builder methods have this problem? We could make builder subtypes:
    // Builder<HasPosition>, Builder<NoPosition> and only define most
    // methods on positioned variant if so. But leave positioning methods
    // on both?

    let stack_ptr = builder.build_alloca(i32_type, "stack_ptr");

    assert_eq!(*stack_ptr.get_type().print_to_string(), *CString::new("i32*").unwrap());

    let stack_array = builder.build_array_alloca(i32_type, i32_three, "stack_array");

    assert_eq!(*stack_array.get_type().print_to_string(), *CString::new("i32*").unwrap());

    let heap_ptr = builder.build_malloc(i32_type, "heap_ptr");

    assert_eq!(*heap_ptr.get_type().print_to_string(), *CString::new("i32*").unwrap());

    let heap_array = builder.build_array_malloc(i32_type, i32_three, "heap_array");

    assert_eq!(*heap_array.get_type().print_to_string(), *CString::new("i32*").unwrap());
}

#[test]
fn test_string_values() {
    let string = VectorValue::const_string("my_string", false);
    let string_null = VectorValue::const_string("my_string", true);

    assert_eq!(string.print_to_string().to_string(), "[9 x i8] c\"my_string\"");
    assert_eq!(string_null.print_to_string().to_string(), "[10 x i8] c\"my_string\\00\"");
    assert!(string.is_const_string());
    assert!(string_null.is_const_string());

    let context = Context::create();
    let i8_type = context.i8_type();
    let string = context.const_string("my_string", false);
    let string_null = context.const_string("my_string", true);

    assert!(!string.is_constant_vector());
    assert!(!string_null.is_constant_vector());
    assert!(!string.is_constant_data_vector());
    assert!(!string_null.is_constant_data_vector());

    assert_eq!(string.print_to_string().to_string(), "[9 x i8] c\"my_string\"");
    assert_eq!(string_null.print_to_string().to_string(), "[10 x i8] c\"my_string\\00\"");
    assert!(string.is_const_string());
    assert!(string_null.is_const_string());
    assert_eq!(string.get_type().get_element_type().into_int_type(), i8_type);
    assert_eq!(string_null.get_type().get_element_type().into_int_type(), i8_type);
    assert_eq!(*string.get_string_constant(), *CString::new("my_string").unwrap());
    assert_eq!(*string_null.get_string_constant(), *CString::new("my_string").unwrap());

    let i8_val = i8_type.const_int(33, false);
    let i8_val2 = i8_type.const_int(43, false);
    let non_string_vec_i8 = VectorType::const_vector(&[i8_val, i8_val2]);

    // TODOC: Will still interpret vec as string even if not generated with const_string:
    assert_eq!(*non_string_vec_i8.get_string_constant(), *CString::new("!+").unwrap());

    let i32_type = context.i32_type();
    let i32_val = i32_type.const_int(33, false);
    let i32_val2 = i32_type.const_int(43, false);
    let non_string_vec_i32 = VectorType::const_vector(&[i32_val, i32_val2, i32_val2]);

    // TODOC: Will still interpret vec with non i8 but in unexpected ways:
    // We may want to restrict this to VectorValue<IntValue<i8>>...
    assert_eq!(*non_string_vec_i32.get_string_constant(), *CString::new("!").unwrap());

    // TODO: Test get_string_constant on non const...
}

#[test]
fn test_consts() {
    let context = Context::create();
    let bool_type = context.bool_type();
    let i8_type = context.i8_type();
    let i16_type = context.i16_type();
    let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    let i128_type = context.i128_type();
    let f16_type = context.f16_type();
    let f32_type = context.f32_type();
    let f64_type = context.f64_type();
    let f128_type = context.f128_type();
    let ppc_f128_type = context.ppc_f128_type();
    let bool_val = bool_type.const_all_ones();
    let i8_val = i8_type.const_all_ones();
    let i16_val = i16_type.const_all_ones();
    let i32_val = i32_type.const_all_ones();
    let i64_val = i64_type.const_all_ones();
    let i128_val = i128_type.const_all_ones();
    let f16_val = f16_type.const_float(1.2);
    let f32_val = f32_type.const_float(3.4);
    let f64_val = f64_type.const_float(5.6);
    let f128_val = f128_type.const_float(7.8);
    let ppc_f128_val = ppc_f128_type.const_float(9.0);
    let vec_val = VectorType::const_vector(&[i8_val]);
    let array_val = i8_type.const_array(&[i8_val]);
    let arbitrary_precision_int = i64_type.const_int_arbitrary_precision(&[1, 2]);

    assert!(bool_val.is_const());
    assert!(i8_val.is_const());
    assert!(i16_val.is_const());
    assert!(i32_val.is_const());
    assert!(i64_val.is_const());
    assert!(i128_val.is_const());
    assert!(f16_val.is_const());
    assert!(f32_val.is_const());
    assert!(f64_val.is_const());
    assert!(f128_val.is_const());
    assert!(ppc_f128_val.is_const());
    assert!(vec_val.is_const());
    assert!(array_val.is_const());
    assert!(arbitrary_precision_int.is_const());

    assert_eq!(*arbitrary_precision_int.print_to_string(), *CString::new("i64 1").unwrap());

    assert!(!vec_val.is_const_string());
    assert!(!vec_val.is_constant_vector());
    assert!(vec_val.is_constant_data_vector());

    assert_eq!(bool_val.get_zero_extended_constant(), Some(1));
    assert_eq!(i8_val.get_zero_extended_constant(), Some(u8::max_value() as u64));
    assert_eq!(i16_val.get_zero_extended_constant(), Some(u16::max_value() as u64));
    assert_eq!(i32_val.get_zero_extended_constant(), Some(u32::max_value() as u64));
    assert_eq!(i64_val.get_zero_extended_constant(), Some(u64::max_value() as u64));
    assert_eq!(i128_val.get_zero_extended_constant(), Some(u128::max_value() as u64));

    // How does a bool get sign extended to -1??
    assert_eq!(bool_val.get_sign_extended_constant(), Some(-1));
    assert_eq!(i8_val.get_sign_extended_constant(), Some(-1));
    assert_eq!(i16_val.get_sign_extended_constant(), Some(-1));
    assert_eq!(i32_val.get_sign_extended_constant(), Some(-1));
    assert_eq!(i64_val.get_sign_extended_constant(), Some(-1));
    assert_eq!(i128_val.get_sign_extended_constant(), Some(-1));

    assert_eq!(f16_val.get_constant(), Some((1.2001953125, false)));
    assert_eq!(f32_val.get_constant(), Some((3.4000000953674316, false)));
    assert_eq!(f64_val.get_constant(), Some((5.6, false)));
    assert_eq!(f128_val.get_constant(), Some((7.8, false)));
    assert_eq!(ppc_f128_val.get_constant(), Some((9.0, false)));

    // Non const test
    let builder = context.create_builder();
    let module = context.create_module("fns");
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[i32_type.into(), f32_type.into()], false);
    let function = module.add_function("fn", fn_type, None);
    let basic_block = context.append_basic_block(&function, "entry");

    builder.position_at_end(&basic_block);

    let i32_param = function.get_first_param().unwrap().into_int_value();
    let f32_param = function.get_nth_param(1).unwrap().into_float_value();

    assert!(i32_param.get_zero_extended_constant().is_none());
    assert!(f32_param.get_constant().is_none());
}

#[test]
fn test_function_value_to_global_to_pointer() {
    let context = Context::create();
    let builder = context.create_builder();
    let module = context.create_module("my_mod");
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let fn_value = module.add_function("my_func", fn_type, None);

    let fn_global_value = fn_value.as_global_value();

    assert!(fn_global_value.is_declaration());

    let bb = fn_value.append_basic_block("entry");

    builder.position_at_end(&bb);
    builder.build_return(None);

    assert!(!fn_global_value.is_declaration());
    assert_eq!(fn_global_value.get_dll_storage_class(), DLLStorageClass::Default);

    fn_global_value.set_dll_storage_class(DLLStorageClass::Export);

    assert_eq!(fn_global_value.get_dll_storage_class(), DLLStorageClass::Export);
    assert!(fn_global_value.get_thread_local_mode().is_none());
    assert_eq!(fn_global_value.get_visibility(), GlobalVisibility::Default);

    let fn_ptr_value = fn_global_value.as_pointer_value();
    let _fn_ptr_type = fn_ptr_value.get_type();

    assert!(!fn_ptr_value.is_null());
    assert_eq!(*fn_ptr_value.get_name(), *CString::new("my_func").unwrap());
    assert!(module.verify().is_ok());
}
