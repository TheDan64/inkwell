use inkwell::context::Context;
use inkwell::memory_buffer::MemoryBuffer;
use inkwell::types::BasicType;
use inkwell::values::AnyValue;
use inkwell::AddressSpace;

#[test]
fn test_struct_type() {
    let context = Context::create();
    let int = context.i8_type();
    let int_vector = int.vec_type(100);
    let float = context.f32_type();
    let float_array = float.array_type(3);
    let av_struct = context.struct_type(&[int_vector.into(), float_array.into()], false);

    assert!(!av_struct.is_packed());
    assert!(!av_struct.is_opaque());
    assert!(av_struct.is_sized());
    assert!(av_struct.get_name().is_none());
    assert_eq!(av_struct.get_context(), context);
    assert_eq!(av_struct.count_fields(), 2);
    for types in [av_struct.get_field_types(), av_struct.get_field_types_iter().collect()] {
        assert_eq!(types, &[int_vector.into(), float_array.into()]);
    }

    let field_1 = av_struct.get_field_type_at_index(0).unwrap();
    let field_2 = av_struct.get_field_type_at_index(1).unwrap();

    assert!(field_1.is_vector_type());
    assert!(field_2.is_array_type());
    assert!(av_struct.get_field_type_at_index(2).is_none());
    assert!(av_struct.get_field_type_at_index(200).is_none());
    for types in [av_struct.get_field_types(), av_struct.get_field_types_iter().collect()] {
        assert_eq!(types, &[field_1, field_2]);
    }

    let av_struct = context.struct_type(&[int_vector.into(), float_array.into()], true);

    assert!(av_struct.is_packed());
    assert!(!av_struct.is_opaque());
    assert!(av_struct.is_sized());
    // REVIEW: Is there a way to name a non opaque struct?
    assert!(av_struct.get_name().is_none());
    assert_eq!(av_struct.get_context(), context);
    assert_eq!(av_struct.count_fields(), 2);

    let field_1 = av_struct.get_field_type_at_index(0).unwrap();
    let field_2 = av_struct.get_field_type_at_index(1).unwrap();

    assert!(field_1.is_vector_type());
    assert!(field_2.is_array_type());
    assert!(av_struct.get_field_type_at_index(2).is_none());
    assert!(av_struct.get_field_type_at_index(200).is_none());
    for types in [av_struct.get_field_types(), av_struct.get_field_types_iter().collect()] {
        assert_eq!(types, &[field_1, field_2]);
    }

    let opaque_struct = context.opaque_struct_type("opaque_struct");

    assert!(!opaque_struct.is_packed());
    assert!(opaque_struct.is_opaque());
    assert!(!opaque_struct.is_sized());
    assert_eq!(opaque_struct.get_name().map(|s| s.to_str()), Some(Ok("opaque_struct")));
    assert_eq!(opaque_struct.get_context(), context);
    assert_eq!(opaque_struct.count_fields(), 0);
    assert!(opaque_struct.get_field_types().is_empty());
    assert_eq!(opaque_struct.get_field_types_iter().count(), 0);
    assert!(opaque_struct.get_field_type_at_index(0).is_none());
    assert!(opaque_struct.get_field_type_at_index(1).is_none());
    assert!(opaque_struct.get_field_type_at_index(2).is_none());
    assert!(opaque_struct.get_field_type_at_index(200).is_none());

    assert!(opaque_struct.set_body(&[int_vector.into(), float_array.into()], true));

    let no_longer_opaque_struct = opaque_struct;

    assert!(no_longer_opaque_struct.is_packed());
    assert!(!no_longer_opaque_struct.is_opaque());
    assert!(no_longer_opaque_struct.is_sized());
    assert_eq!(
        no_longer_opaque_struct.get_name().map(|s| s.to_str()),
        Some(Ok("opaque_struct"))
    );
    assert_eq!(no_longer_opaque_struct.get_context(), context);
    assert_eq!(no_longer_opaque_struct.count_fields(), 2);
    for types in [
        no_longer_opaque_struct.get_field_types(),
        no_longer_opaque_struct.get_field_types_iter().collect(),
    ] {
        assert_eq!(types, &[int_vector.into(), float_array.into()]);
    }

    let field_1 = no_longer_opaque_struct.get_field_type_at_index(0).unwrap();
    let field_2 = no_longer_opaque_struct.get_field_type_at_index(1).unwrap();

    assert!(field_1.is_vector_type());
    assert!(field_2.is_array_type());
    assert!(no_longer_opaque_struct.get_field_type_at_index(2).is_none());
    assert!(no_longer_opaque_struct.get_field_type_at_index(200).is_none());
    for types in [
        no_longer_opaque_struct.get_field_types(),
        no_longer_opaque_struct.get_field_types_iter().collect(),
    ] {
        assert_eq!(types, &[field_1, field_2]);
    }

    no_longer_opaque_struct.set_body(&[float_array.into(), int_vector.into(), float_array.into()], false);
    let fields_changed_struct = no_longer_opaque_struct;
    // assert!(!fields_changed_struct.is_packed()); FIXME: This seems to be a bug in LLVM
    assert!(!fields_changed_struct.is_opaque());
    assert!(fields_changed_struct.is_sized());
    assert_eq!(fields_changed_struct.count_fields(), 3);
    for types in [
        fields_changed_struct.get_field_types(),
        fields_changed_struct.get_field_types_iter().collect(),
    ] {
        assert_eq!(types, &[float_array.into(), int_vector.into(), float_array.into(),]);
    }
    assert!(fields_changed_struct.get_field_type_at_index(3).is_none());
}

#[test]
fn test_function_type() {
    let context = Context::create();
    let int = context.i8_type();
    let float = context.f32_type();
    let fn_type = int.fn_type(&[int.into(), int.into(), float.into()], false);

    assert!(!fn_type.is_var_arg());
    assert_eq!(fn_type.get_context(), context);

    let param_types = fn_type.get_param_types();

    assert_eq!(param_types.len(), 3);
    assert_eq!(param_types[0].into_int_type(), int);
    assert_eq!(param_types[1].into_int_type(), int);
    assert_eq!(param_types[2].into_float_type(), float);

    let fn_type = int.fn_type(&[int.into(), float.into()], true);

    assert!(fn_type.is_var_arg());
    assert_eq!(fn_type.get_context(), context);
}

/// Check that `FunctionType::get_param_types()` can handle metadata arguments.
/// Regression test for inkwell#546
#[test]
fn test_function_type_metadata_params() {
    let llvm_ir = r#"
        declare void @my_fn(i32, metadata)
    "#;

    let context = Context::create();
    let i32_type = context.i32_type();
    let md_type = context.metadata_type();

    let memory_buffer = MemoryBuffer::create_from_memory_range_copy(llvm_ir.as_bytes(), "my_mod");
    let module = context.create_module_from_ir(memory_buffer).unwrap();

    let fn_type = module.get_function("my_fn").unwrap().get_type();
    let param_types = fn_type.get_param_types();

    assert_eq!(param_types.len(), 2);
    assert_eq!(param_types[0].into_int_type(), i32_type);
    assert_eq!(param_types[1].into_metadata_type(), md_type);
}

#[test]
fn test_sized_types() {
    unsafe { Context::get_global(sized_types) }
}

fn sized_types(global_ctx: &Context) {
    let void_type = global_ctx.void_type();
    let bool_type = global_ctx.bool_type();
    let i8_type = global_ctx.i8_type();
    let i16_type = global_ctx.i16_type();
    let i32_type = global_ctx.i32_type();
    let i64_type = global_ctx.i64_type();
    let i128_type = global_ctx.i128_type();
    let f16_type = global_ctx.f16_type();
    let f32_type = global_ctx.f32_type();
    let f64_type = global_ctx.f64_type();
    let f80_type = global_ctx.x86_f80_type();
    let f128_type = global_ctx.f128_type();
    let ppc_f128_type = global_ctx.ppc_f128_type();
    #[cfg(not(feature = "typed-pointers"))]
    let ptr_type = global_ctx.ptr_type(AddressSpace::default());
    let struct_type = global_ctx.struct_type(&[i8_type.into(), f128_type.into()], false);
    let struct_type2 = global_ctx.struct_type(&[], false);
    let struct_type3 = global_ctx.struct_type(&[i8_type.into(), f128_type.into()], true);
    let struct_type4 = global_ctx.struct_type(&[], true);
    let fn_type = void_type.fn_type(&[], false);
    let fn_type2 = i8_type.fn_type(&[], false);
    let fn_type3 = void_type.fn_type(&[i32_type.into(), struct_type.into()], false);
    let fn_type4 = i8_type.fn_type(&[struct_type.into(), i32_type.into()], false);

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
    assert!(f80_type.is_sized());
    assert!(f128_type.is_sized());
    assert!(ppc_f128_type.is_sized());
    #[cfg(not(feature = "typed-pointers"))]
    assert!(ptr_type.is_sized());
    assert!(struct_type.is_sized());
    assert!(struct_type2.is_sized());
    assert!(struct_type3.is_sized());
    assert!(struct_type4.is_sized());
    assert!(!fn_type.is_sized());
    assert!(!fn_type2.is_sized());
    assert!(!fn_type3.is_sized());
    assert!(!fn_type4.is_sized());

    #[cfg(feature = "typed-pointers")]
    {
        assert!(bool_type.ptr_type(AddressSpace::default()).is_sized());
        assert!(i8_type.ptr_type(AddressSpace::default()).is_sized());
        assert!(i16_type.ptr_type(AddressSpace::default()).is_sized());
        assert!(i32_type.ptr_type(AddressSpace::default()).is_sized());
        assert!(i64_type.ptr_type(AddressSpace::default()).is_sized());
        assert!(i128_type.ptr_type(AddressSpace::default()).is_sized());
        assert!(f16_type.ptr_type(AddressSpace::default()).is_sized());
        assert!(f32_type.ptr_type(AddressSpace::default()).is_sized());
        assert!(f64_type.ptr_type(AddressSpace::default()).is_sized());
        assert!(f80_type.ptr_type(AddressSpace::default()).is_sized());
        assert!(f128_type.ptr_type(AddressSpace::default()).is_sized());
        assert!(ppc_f128_type.ptr_type(AddressSpace::default()).is_sized());
        assert!(struct_type.ptr_type(AddressSpace::default()).is_sized());
        assert!(struct_type2.ptr_type(AddressSpace::default()).is_sized());
        assert!(struct_type3.ptr_type(AddressSpace::default()).is_sized());
        assert!(struct_type4.ptr_type(AddressSpace::default()).is_sized());
    }

    assert!(bool_type.array_type(42).is_sized());
    assert!(i8_type.array_type(42).is_sized());
    assert!(i16_type.array_type(42).is_sized());
    assert!(i32_type.array_type(42).is_sized());
    assert!(i64_type.array_type(42).is_sized());
    assert!(i128_type.array_type(42).is_sized());
    assert!(f16_type.array_type(42).is_sized());
    assert!(f32_type.array_type(42).is_sized());
    assert!(f64_type.array_type(42).is_sized());
    assert!(f80_type.array_type(42).is_sized());
    assert!(f128_type.array_type(42).is_sized());
    assert!(ppc_f128_type.array_type(42).is_sized());
    #[cfg(not(feature = "typed-pointers"))]
    assert!(ptr_type.array_type(42).is_sized());
    assert!(struct_type.array_type(0).is_sized());
    assert!(struct_type2.array_type(0).is_sized());
    assert!(struct_type3.array_type(0).is_sized());
    assert!(struct_type4.array_type(0).is_sized());

    assert!(bool_type.vec_type(42).is_sized());
    assert!(i8_type.vec_type(42).is_sized());
    assert!(i16_type.vec_type(42).is_sized());
    assert!(i32_type.vec_type(42).is_sized());
    assert!(i64_type.vec_type(42).is_sized());
    assert!(i128_type.vec_type(42).is_sized());
    assert!(f16_type.vec_type(42).is_sized());
    assert!(f32_type.vec_type(42).is_sized());
    assert!(f64_type.vec_type(42).is_sized());
    assert!(f80_type.vec_type(42).is_sized());
    assert!(f128_type.vec_type(42).is_sized());
    assert!(ppc_f128_type.vec_type(42).is_sized());
    #[cfg(not(feature = "typed-pointers"))]
    assert!(ptr_type.vec_type(42).is_sized());

    #[cfg(any(
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0",
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-1"
    ))]
    {
        assert!(bool_type.scalable_vec_type(42).is_sized());
        assert!(i8_type.scalable_vec_type(42).is_sized());
        assert!(i16_type.scalable_vec_type(42).is_sized());
        assert!(i32_type.scalable_vec_type(42).is_sized());
        assert!(i64_type.scalable_vec_type(42).is_sized());
        assert!(i128_type.scalable_vec_type(42).is_sized());
        assert!(f16_type.scalable_vec_type(42).is_sized());
        assert!(f32_type.scalable_vec_type(42).is_sized());
        assert!(f64_type.scalable_vec_type(42).is_sized());
        assert!(f80_type.scalable_vec_type(42).is_sized());
        assert!(f128_type.scalable_vec_type(42).is_sized());
        assert!(ppc_f128_type.scalable_vec_type(42).is_sized());
        #[cfg(not(feature = "typed-pointers"))]
        assert!(ptr_type.scalable_vec_type(42).is_sized());
    }

    let opaque_struct_type = global_ctx.opaque_struct_type("opaque");

    assert!(!opaque_struct_type.is_sized());
    assert!(!opaque_struct_type.array_type(0).is_sized());

    #[cfg(feature = "typed-pointers")]
    {
        let opaque_struct_ptr_type = opaque_struct_type.ptr_type(AddressSpace::default());
        assert!(opaque_struct_ptr_type.is_sized());
    }
}

#[test]
fn test_const_zero() {
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
    let f80_type = context.x86_f80_type();
    let f128_type = context.f128_type();
    let ppc_f128_type = context.ppc_f128_type();
    let struct_type = context.struct_type(&[i8_type.into(), f128_type.into()], false);
    #[cfg(feature = "typed-pointers")]
    let ptr_type = f64_type.ptr_type(AddressSpace::default());
    #[cfg(not(feature = "typed-pointers"))]
    let ptr_type = context.ptr_type(AddressSpace::default());
    let vec_type = f64_type.vec_type(42);
    #[cfg(any(
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0",
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-1"
    ))]
    let scalable_vec_type = f64_type.scalable_vec_type(42);
    let array_type = f64_type.array_type(42);

    bool_type.size_of();
    f32_type.size_of();
    struct_type.size_of();
    ptr_type.size_of();
    vec_type.size_of();
    #[cfg(any(
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0",
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-1"
    ))]
    scalable_vec_type.size_of();
    array_type.size_of();

    bool_type.get_alignment();
    f32_type.get_alignment();
    struct_type.get_alignment();
    ptr_type.get_alignment();
    vec_type.get_alignment();
    #[cfg(any(
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0",
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-1"
    ))]
    scalable_vec_type.get_alignment();
    array_type.get_alignment();

    let bool_zero = bool_type.const_zero();
    let i8_zero = i8_type.const_zero();
    let i16_zero = i16_type.const_zero();
    let i32_zero = i32_type.const_zero();
    let i64_zero = i64_type.const_zero();
    let i128_zero = i128_type.const_zero();
    let f16_zero = f16_type.const_zero();
    let f32_zero = f32_type.const_zero();
    let f64_zero = f64_type.const_zero();
    let f80_zero = f80_type.const_zero();
    let f128_zero = f128_type.const_zero();
    let ppc_f128_zero = ppc_f128_type.const_zero();
    let struct_zero = struct_type.const_zero();
    let ptr_zero = ptr_type.const_zero();
    let vec_zero = vec_type.const_zero();
    #[cfg(any(
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0",
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-1"
    ))]
    let scalable_vec_zero = scalable_vec_type.const_zero();
    let array_zero = array_type.const_zero();

    assert!(bool_zero.is_null());
    assert!(i8_zero.is_null());
    assert!(i16_zero.is_null());
    assert!(i32_zero.is_null());
    assert!(i64_zero.is_null());
    assert!(i128_zero.is_null());
    assert!(f16_zero.is_null());
    assert!(f32_zero.is_null());
    assert!(f64_zero.is_null());
    assert!(f80_zero.is_null());
    assert!(f128_zero.is_null());
    assert!(ppc_f128_zero.is_null());
    assert!(struct_zero.is_null());
    assert!(ptr_zero.is_null());
    assert!(vec_zero.is_null());
    #[cfg(any(
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0",
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-1"
    ))]
    assert!(scalable_vec_zero.is_null());
    assert!(array_zero.is_null());

    assert_eq!(bool_zero.print_to_string().to_str(), Ok("i1 false"));
    assert_eq!(i8_zero.print_to_string().to_str(), Ok("i8 0"));
    assert_eq!(i16_zero.print_to_string().to_str(), Ok("i16 0"));
    assert_eq!(i32_zero.print_to_string().to_str(), Ok("i32 0"));
    assert_eq!(i64_zero.print_to_string().to_str(), Ok("i64 0"));
    assert_eq!(i128_zero.print_to_string().to_str(), Ok("i128 0"));
    assert_eq!(f16_zero.print_to_string().to_str(), Ok("half 0xH0000"));
    assert_eq!(f32_zero.print_to_string().to_str(), Ok("float 0.000000e+00"));
    assert_eq!(f64_zero.print_to_string().to_str(), Ok("double 0.000000e+00"));
    assert_eq!(
        f80_zero.print_to_string().to_str(),
        Ok("x86_fp80 0xK00000000000000000000")
    );
    assert_eq!(
        f128_zero.print_to_string().to_str(),
        Ok("fp128 0xL00000000000000000000000000000000")
    );
    assert_eq!(
        ppc_f128_zero.print_to_string().to_str(),
        Ok("ppc_fp128 0xM00000000000000000000000000000000")
    );
    assert_eq!(
        struct_zero.print_to_string().to_str(),
        Ok("{ i8, fp128 } zeroinitializer")
    );

    // handle opaque pointers
    let ptr_type = if cfg!(not(feature = "typed-pointers")) {
        "ptr null"
    } else {
        "double* null"
    };

    assert_eq!(ptr_zero.print_to_string().to_str(), Ok(ptr_type));

    assert_eq!(vec_zero.print_to_string().to_str(), Ok("<42 x double> zeroinitializer"));
    #[cfg(any(
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0",
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-1"
    ))]
    assert_eq!(
        scalable_vec_zero.print_to_string().to_str(),
        Ok("<vscale x 42 x double> zeroinitializer")
    );
    assert_eq!(
        array_zero.print_to_string().to_str(),
        Ok("[42 x double] zeroinitializer")
    );
}

#[test]
fn test_vec_type() {
    let context = Context::create();
    let int = context.i8_type();
    let vec_type = int.vec_type(42);

    assert_eq!(vec_type.get_size(), 42);
}

#[llvm_versions(12..)]
#[test]
fn test_scalable_vec_type() {
    let context = Context::create();
    let int = context.i8_type();
    let vec_type = int.scalable_vec_type(42);

    assert_eq!(vec_type.get_size(), 42);
}

#[test]
fn test_type_copies() {
    let context = Context::create();
    let i8_type = context.i8_type();
    let i8_type_copy = i8_type;

    assert_eq!(i8_type, i8_type_copy);
}

#[test]
fn test_ptr_type() {
    let context = Context::create();
    #[cfg(feature = "typed-pointers")]
    let ptr_type = context.i8_type().ptr_type(AddressSpace::default());
    #[cfg(not(feature = "typed-pointers"))]
    let ptr_type = context.ptr_type(AddressSpace::default());

    assert_eq!(ptr_type.get_address_space(), AddressSpace::default());

    #[cfg(feature = "typed-pointers")]
    assert_eq!(ptr_type.get_element_type().into_int_type(), context.i8_type());

    // Fn ptr:
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    #[allow(deprecated)]
    let fn_ptr_type = fn_type.ptr_type(AddressSpace::default());

    #[cfg(feature = "typed-pointers")]
    assert_eq!(fn_ptr_type.get_element_type().into_function_type(), fn_type);

    assert_eq!(fn_ptr_type.get_context(), context);
}

#[test]
fn test_basic_type_enum() {
    let context = Context::create();
    let addr = AddressSpace::default();
    let int = context.i32_type();
    let types: &[&dyn BasicType] = &[
        // ints and floats
        &int,
        &context.i64_type(),
        &context.f32_type(),
        &context.f64_type(),
        // derived types
        &int.array_type(0),
        #[cfg(feature = "typed-pointers")]
        &int.ptr_type(addr),
        #[cfg(not(feature = "typed-pointers"))]
        &context.ptr_type(addr),
        &context.struct_type(&[int.as_basic_type_enum()], false),
        &int.vec_type(1),
        #[cfg(any(
            feature = "llvm12-0",
            feature = "llvm13-0",
            feature = "llvm14-0",
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-1"
        ))]
        &int.scalable_vec_type(1),
    ];
    for basic_type in types {
        #[cfg(feature = "typed-pointers")]
        assert_eq!(
            basic_type.as_basic_type_enum().ptr_type(addr),
            basic_type.ptr_type(addr)
        );
        assert_eq!(basic_type.as_basic_type_enum().array_type(0), basic_type.array_type(0));
        assert_eq!(basic_type.as_basic_type_enum().size_of(), basic_type.size_of());
    }
}

#[test]
#[should_panic]
fn test_no_vector_zero() {
    let context = Context::create();
    let int = context.i32_type();
    int.vec_type(0);
}

#[llvm_versions(12..)]
#[test]
#[should_panic]
fn test_no_scalable_vector_zero() {
    let context = Context::create();
    let int = context.i32_type();
    int.scalable_vec_type(0);
}

#[test]
fn test_ptr_address_space() {
    let context = Context::create();

    let spaces = [0u32, 1, 2, 3, 4, 5, 6, (1 << 24) - 1];

    for index in spaces {
        let address_space = AddressSpace::try_from(index).unwrap();

        #[cfg(feature = "typed-pointers")]
        let ptr = context.i32_type().ptr_type(address_space);
        #[cfg(not(feature = "typed-pointers"))]
        let ptr = context.ptr_type(address_space);
        assert_eq!(ptr.get_address_space(), address_space);
    }

    assert!(AddressSpace::try_from(1u32 << 24).is_err());
}

#[llvm_versions(15..)]
#[test]
#[allow(deprecated)]
fn test_ptr_is_opaque() {
    let context = Context::create();

    let i32_ptr_type = context.i32_type().ptr_type(AddressSpace::default());
    assert_eq!(!i32_ptr_type.is_opaque(), cfg!(feature = "typed-pointers"));

    #[cfg(not(feature = "typed-pointers"))]
    {
        let ptr_type = context.ptr_type(AddressSpace::default());
        assert!(ptr_type.is_opaque());
    }
}
