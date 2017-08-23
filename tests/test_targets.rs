extern crate inkwell;

use self::inkwell::context::Context;
use self::inkwell::targets::{ByteOrdering, InitializationConfig, Target};

use std::ffi::CString;

// REVIEW: Inconsistently failing on different tries :(
// #[test]
// fn test_target() {
//     // REVIEW: Some of the machine specific stuff may vary. Should allow multiple possibilites
//     assert_eq!(TargetMachine::get_default_triple(), &*CString::new("x86_64-pc-linux-gnu").unwrap());
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

//     assert!(target.create_target_machine("x86-64", "xx", "yy", None, RelocMode::Default, CodeModel::Default).is_none());

//     config.base = true;

//     Target::initialize_x86(&config);

//     let target = Target::get_first().expect("Did not find any target");

//     assert!(!target.has_asm_backend());
//     assert!(target.has_target_machine());

//     let target_machine = target.create_target_machine("zz", "xx", "yy", None, RelocMode::Default, CodeModel::Default).expect("Could not create TargetMachine");

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
fn test_target_data() {
    Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");

    let context = Context::create();
    let module = context.create_module("sum");
    let execution_engine = module.create_jit_execution_engine(0).unwrap();
    let module = execution_engine.get_module_at(0);
    let target_data = execution_engine.get_target_data();

    let data_layout = target_data.get_data_layout(); // TODO: See if you can test data_layout

    assert_eq!(data_layout.as_str(), &*CString::new("e-m:e-i64:64-f80:128-n8:16:32:64-S128").unwrap());
    assert_eq!(module.get_data_layout().as_str(), &*CString::new("").unwrap());

    module.set_data_layout(&data_layout);

    assert_eq!(*module.get_data_layout(), data_layout);

    let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    let f32_type = context.f32_type();
    let f64_type = context.f64_type();
    let struct_type = context.struct_type(&[&i32_type, &i64_type, &f64_type, &f32_type], false);
    let struct_type2 = context.struct_type(&[&f32_type, &i32_type, &i64_type, &f64_type], false);

    assert_eq!(target_data.get_bit_size(&i32_type), 32);
    assert_eq!(target_data.get_bit_size(&i64_type), 64);
    assert_eq!(target_data.get_bit_size(&f32_type), 32);
    assert_eq!(target_data.get_bit_size(&f64_type), 64);
    assert_eq!(target_data.get_bit_size(&struct_type), 256);
    assert_eq!(target_data.get_bit_size(&struct_type2), 192);

    // REVIEW: What if these fail on a different system?
    assert_eq!(target_data.get_byte_ordering(), ByteOrdering::LittleEndian);
    assert_eq!(target_data.get_pointer_byte_size(), 8);

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
}
