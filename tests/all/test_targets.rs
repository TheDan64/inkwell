extern crate inkwell;

use self::inkwell::{AddressSpace, OptimizationLevel};
use self::inkwell::context::Context;
use self::inkwell::targets::{ByteOrdering, CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetData, TargetMachine};

use std::env::temp_dir;
use std::ffi::CString;
use std::fs::{File, remove_file};
use std::io::Read;
use std::str::from_utf8;

// REVIEW: Inconsistently failing on different tries :(
// #[test]
// fn test_target() {
//     // REVIEW: Some of the machine specific stuff may vary. Should allow multiple possibilites
//     assert!(Target::get_first().is_none());

//     let mut config = InitializationConfig {
//         asm_parser: false,
//         asm_printer: false,
//         base: false,
//         disassembler: false,
//         info: true,
//         machine_code: false,
//     };

//     Target::initialize_x86(&config);

//     let target = Target::get_first().expect("Did not find any target");

//     assert_eq!(target.get_name(), &*CString::new("x86-64").unwrap());
//     assert_eq!(target.get_description(), &*CString::new("64-bit X86: EM64T and AMD64").unwrap());
//     assert!(target.has_jit());
//     assert!(!target.has_asm_backend());
//     assert!(!target.has_target_machine());

//     assert!(target.create_target_machine("x86-64", "xx", "yy", OptimizationLevel::Default, RelocMode::Default, CodeModel::Default).is_none());

//     config.base = true;

//     Target::initialize_x86(&config);

//     let target = Target::get_first().expect("Did not find any target");

//     assert!(!target.has_asm_backend());
//     assert!(target.has_target_machine());

//     let target_machine = target.create_target_machine("zz", "xx", "yy", OptimizationLevel::Default, RelocMode::Default, CodeModel::Default).expect("Could not create TargetMachine");

//     config.machine_code = true;

//     Target::initialize_x86(&config);

//     let target = Target::get_first().expect("Did not find any target");

//     assert!(target.has_asm_backend());
//     assert!(target.has_target_machine());

//     // TODO: See what happens to create_target_machine when when target.has_target_machine() is false
//     // Maybe it should return an Option<TargetMachine>
//     // TODO: TargetMachine testing

//     target.get_next().expect("Did not find any target2");
// }

#[test]
fn test_target_and_target_machine() {
    let bad_target = Target::from_name("asd");

    assert!(bad_target.is_none());

    let _bad_target2 = Target::from_triple("x86_64-pc-linux-gnu");

    // REVIEW: Inconsistent success :(
    // assert_eq!(*bad_target2.unwrap_err(), *CString::new("Unable to find target for this triple (no targets are registered)").unwrap());

    Target::initialize_x86(&InitializationConfig::default());

    let good_target = Target::from_name("x86-64");

    assert!(good_target.is_some());

    let good_target2 = Target::from_triple("x86_64-pc-linux-gnu");

    assert!(good_target2.is_ok(), "{}", good_target2.unwrap_err());

    let good_target = good_target.unwrap();
    let good_target2 = good_target2.unwrap();

    assert_eq!(good_target, good_target2);
    assert_eq!(*good_target.get_name(), *CString::new("x86-64").unwrap());
    assert_eq!(*good_target2.get_name(), *CString::new("x86-64").unwrap());
    assert_eq!(*good_target.get_description(), *CString::new("64-bit X86: EM64T and AMD64").unwrap());
    assert_eq!(*good_target2.get_description(), *CString::new("64-bit X86: EM64T and AMD64").unwrap());
    assert!(good_target.has_jit());
    assert!(good_target2.has_jit());
    assert!(good_target.has_target_machine());
    assert!(good_target2.has_target_machine());
    assert!(good_target.has_asm_backend());
    assert!(good_target2.has_asm_backend());

    let next_target = good_target.get_next().unwrap();

    assert_eq!(*next_target.get_name(), *CString::new("x86").unwrap());

    let target_machine = good_target.create_target_machine("x86_64-pc-linux-gnu", "x86-64", "+avx2", OptimizationLevel::Default, RelocMode::Default, CodeModel::Default).unwrap();

    // TODO: Test target_machine failure

    target_machine.set_asm_verbosity(true);

    let triple = target_machine.get_triple();

    assert_eq!(target_machine.get_target(), good_target);
    assert_eq!(*triple, *CString::new("x86_64-pc-linux-gnu").unwrap());
    assert_eq!(*target_machine.get_cpu(), *CString::new("x86-64").unwrap());
    assert_eq!(*target_machine.get_feature_string(), *CString::new("+avx2").unwrap());

    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8", feature = "llvm3-9",
                  feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0")))]
    {
        use either::Either::{Left, Right};

        // TODO: Try and find a triple that actually gets normalized..
        assert_eq!(*TargetMachine::normalize_target_triple(Left("x86_64-pc-linux-gnu")), *CString::new("x86_64-pc-linux-gnu").unwrap());
        assert_eq!(*TargetMachine::normalize_target_triple(Right(&*triple)), *CString::new("x86_64-pc-linux-gnu").unwrap());

        let _host_name = TargetMachine::get_host_cpu_name();
        let _host_cpu_features = TargetMachine::get_host_cpu_features();
    }
}

#[test]
fn test_default_target_triple() {
    let default_target_triple = TargetMachine::get_default_triple();
    let default_target_triple = default_target_triple.to_str().unwrap();

    #[cfg(target_os = "linux")]
    let cond = default_target_triple == "x86_64-pc-linux-gnu" ||
               default_target_triple == "x86_64-unknown-linux-gnu";

    #[cfg(target_os = "macos")]
    let cond = default_target_triple.starts_with("x86_64-apple-darwin");

    assert!(cond, "Unexpected target triple: {}", default_target_triple);

    // TODO: CFG for other supported major OSes
}

#[test]
fn test_target_data() {
    Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");

    let context = Context::create();
    let module = context.create_module("sum");
    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
    let target_data = execution_engine.get_target_data();

    let data_layout = target_data.get_data_layout();

    #[cfg(target_os = "linux")]
    assert_eq!(data_layout.as_str(), &*CString::new("e-m:e-i64:64-f80:128-n8:16:32:64-S128").unwrap());

    #[cfg(target_os = "macos")]
    assert_eq!(
        data_layout.as_str(),
        &*CString::new("e-m:o-i64:64-f80:128-n8:16:32:64-S128").unwrap()
    );

    #[cfg(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8"))]
    assert_eq!(*module.get_data_layout().as_str(), *CString::new("").unwrap());
    // REVIEW: Why is llvm 3.9+ a %? 4.0 on travis doesn't have it, but does for me locally...
    // #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8")))]
    // assert_eq!(module.get_data_layout().as_str(), &*CString::new("%").unwrap());

    module.set_data_layout(&data_layout);

    assert_eq!(*module.get_data_layout(), data_layout);

    let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    let f32_type = context.f32_type();
    let f64_type = context.f64_type();
    let struct_type = context.struct_type(&[i32_type.into(), i64_type.into(), f64_type.into(), f32_type.into()], false);
    let struct_type2 = context.struct_type(&[f32_type.into(), i32_type.into(), i64_type.into(), f64_type.into()], false);

    assert_eq!(target_data.get_bit_size(&i32_type), 32);
    assert_eq!(target_data.get_bit_size(&i64_type), 64);
    assert_eq!(target_data.get_bit_size(&f32_type), 32);
    assert_eq!(target_data.get_bit_size(&f64_type), 64);
    assert_eq!(target_data.get_bit_size(&struct_type), 256);
    assert_eq!(target_data.get_bit_size(&struct_type2), 192);

    // REVIEW: What if these fail on a different system?
    assert_eq!(target_data.get_byte_ordering(), ByteOrdering::LittleEndian);
    assert_eq!(target_data.get_pointer_byte_size(None), 8);

    // REVIEW: Are these just byte size? Maybe rename to get_byte_size?
    assert_eq!(target_data.get_store_size(&i32_type), 4);
    assert_eq!(target_data.get_store_size(&i64_type), 8);
    assert_eq!(target_data.get_store_size(&f32_type), 4);
    assert_eq!(target_data.get_store_size(&f64_type), 8);
    assert_eq!(target_data.get_store_size(&struct_type), 32);
    assert_eq!(target_data.get_store_size(&struct_type2), 24);

    // REVIEW: What's the difference between this an above?
    assert_eq!(target_data.get_abi_size(&i32_type), 4);
    assert_eq!(target_data.get_abi_size(&i64_type), 8);
    assert_eq!(target_data.get_abi_size(&f32_type), 4);
    assert_eq!(target_data.get_abi_size(&f64_type), 8);
    assert_eq!(target_data.get_abi_size(&struct_type), 32);
    assert_eq!(target_data.get_abi_size(&struct_type2), 24);

    assert_eq!(target_data.get_abi_alignment(&i32_type), 4);
    assert_eq!(target_data.get_abi_alignment(&i64_type), 8);
    assert_eq!(target_data.get_abi_alignment(&f32_type), 4);
    assert_eq!(target_data.get_abi_alignment(&f64_type), 8);
    assert_eq!(target_data.get_abi_alignment(&struct_type), 8);
    assert_eq!(target_data.get_abi_alignment(&struct_type2), 8);

    assert_eq!(target_data.get_call_frame_alignment(&i32_type), 4);
    assert_eq!(target_data.get_call_frame_alignment(&i64_type), 8);
    assert_eq!(target_data.get_call_frame_alignment(&f32_type), 4);
    assert_eq!(target_data.get_call_frame_alignment(&f64_type), 8);
    assert_eq!(target_data.get_call_frame_alignment(&struct_type), 8);
    assert_eq!(target_data.get_call_frame_alignment(&struct_type2), 8);

    assert_eq!(target_data.get_preferred_alignment(&i32_type), 4);
    assert_eq!(target_data.get_preferred_alignment(&i64_type), 8);
    assert_eq!(target_data.get_preferred_alignment(&f32_type), 4);
    assert_eq!(target_data.get_preferred_alignment(&f64_type), 8);
    assert_eq!(target_data.get_preferred_alignment(&struct_type), 8);
    assert_eq!(target_data.get_preferred_alignment(&struct_type2), 8);

    // REVIEW: offset in bytes? Rename to byte_offset_of_element?
    assert_eq!(target_data.offset_of_element(&struct_type, 0), Some(0));
    assert_eq!(target_data.offset_of_element(&struct_type, 1), Some(8));
    assert_eq!(target_data.offset_of_element(&struct_type, 2), Some(16));
    assert_eq!(target_data.offset_of_element(&struct_type, 3), Some(24));
    assert!(target_data.offset_of_element(&struct_type, 4).is_none()); // OoB
    assert!(target_data.offset_of_element(&struct_type, 10).is_none()); // OoB

    assert_eq!(target_data.element_at_offset(&struct_type, 0), 0);
    assert_eq!(target_data.element_at_offset(&struct_type, 4), 0);
    assert_eq!(target_data.element_at_offset(&struct_type, 8), 1);
    assert_eq!(target_data.element_at_offset(&struct_type, 16), 2);
    assert_eq!(target_data.element_at_offset(&struct_type, 24), 3);
    assert_eq!(target_data.element_at_offset(&struct_type, 32), 3); // OoB
    assert_eq!(target_data.element_at_offset(&struct_type, ::std::u64::MAX), 3); // OoB; Odd as it seems to cap at max element number

    assert_eq!(target_data.offset_of_element(&struct_type2, 0), Some(0));
    assert_eq!(target_data.offset_of_element(&struct_type2, 1), Some(4));
    assert_eq!(target_data.offset_of_element(&struct_type2, 2), Some(8));
    assert_eq!(target_data.offset_of_element(&struct_type2, 3), Some(16));
    assert!(target_data.offset_of_element(&struct_type2, 4).is_none()); // OoB
    assert!(target_data.offset_of_element(&struct_type2, 5).is_none()); // OoB

    assert_eq!(target_data.element_at_offset(&struct_type2, 0), 0);
    assert_eq!(target_data.element_at_offset(&struct_type2, 2), 0);
    assert_eq!(target_data.element_at_offset(&struct_type2, 4), 1);
    assert_eq!(target_data.element_at_offset(&struct_type2, 8), 2);
    assert_eq!(target_data.element_at_offset(&struct_type2, 16), 3);
    assert_eq!(target_data.element_at_offset(&struct_type2, 32), 3); // OoB
    assert_eq!(target_data.element_at_offset(&struct_type2, ::std::u64::MAX), 3); // OoB; TODOC: Odd but seems to cap at max element number

    TargetData::create("e-m:e-i64:64-f80:128-n8:16:32:64-S128");
}

#[test]
fn test_ptr_sized_int() {
    Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");

    let context = Context::create();
    let module = context.create_module("sum");
    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
    let target_data = execution_engine.get_target_data();
    let address_space = AddressSpace::Global;
    let int_type = target_data.ptr_sized_int_type(None);

    assert_eq!(int_type.get_context(), Context::get_global());
    assert_eq!(int_type.get_bit_width(), target_data.get_pointer_byte_size(None) * 8);

    let int_type2 = target_data.ptr_sized_int_type(Some(address_space));

    assert_eq!(int_type2.get_context(), Context::get_global());
    assert_eq!(int_type2.get_bit_width(), target_data.get_pointer_byte_size(Some(address_space)) * 8);

    let int_type3 = target_data.ptr_sized_int_type_in_context(&context, None);

    assert_eq!(*int_type3.get_context(), context);
    assert_eq!(int_type3.get_bit_width(), target_data.get_pointer_byte_size(None) * 8);

    let int_type4 = target_data.ptr_sized_int_type_in_context(&context, Some(address_space));

    assert_eq!(*int_type4.get_context(), context);
    assert_eq!(int_type4.get_bit_width(), target_data.get_pointer_byte_size(Some(address_space)) * 8);
}

#[test]
fn test_write_target_machine_to_file() {
    Target::initialize_x86(&InitializationConfig::default());

    let target = Target::from_name("x86-64").unwrap();
    let target_machine = target.create_target_machine("x86_64-pc-linux-gnu", "x86-64", "+avx2", OptimizationLevel::Less, RelocMode::Static, CodeModel::Small).unwrap();
    let mut path = temp_dir();

    path.push("temp.asm");

    let context = Context::create();
    let module = context.create_module("my_module");
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);

    module.add_function("my_fn", fn_type, None);

    assert!(target_machine.write_to_file(&module, FileType::Assembly, &path).is_ok());

    let mut contents = Vec::new();
    let mut file = File::open(&path).expect("Could not open temp file");

    file.read_to_end(&mut contents).expect("Unable to verify written file");

    assert!(!contents.is_empty());

    let string = from_utf8(&contents).unwrap();

    assert!(string.contains(".text"));
    assert!(string.contains(".file"));
    assert!(string.contains("my_module"));
    assert!(string.contains(".section"));

    remove_file(&path).unwrap();
}

#[test]
fn test_write_target_machine_to_memory_buffer() {
    Target::initialize_x86(&InitializationConfig::default());

    let target = Target::from_name("x86-64").unwrap();
    let target_machine = target.create_target_machine("x86_64-pc-linux-gnu", "x86-64", "+avx2", OptimizationLevel::Aggressive, RelocMode::PIC, CodeModel::Medium).unwrap();

    let context = Context::create();
    let module = context.create_module("my_module");
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);

    module.add_function("my_fn", fn_type, None);

    let buffer = target_machine.write_to_memory_buffer(&module, FileType::Assembly).unwrap();

    assert!(!buffer.get_size() > 0);

    let string = from_utf8(buffer.as_slice()).unwrap();

    assert!(string.contains(".text"));
    assert!(string.contains(".file"));
    assert!(string.contains("my_module"));
    assert!(string.contains(".section"));
}
