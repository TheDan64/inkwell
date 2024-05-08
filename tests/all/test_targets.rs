use inkwell::context::Context;
use inkwell::targets::{
    ByteOrdering, CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetData, TargetMachine, TargetTriple,
};
use inkwell::{AddressSpace, OptimizationLevel};

use regex::Regex;

use std::env::temp_dir;
use std::fs::{remove_file, File};
use std::io::Read;
use std::str::from_utf8;

fn write_target_machine_to_memory_buffer(target_machine: TargetMachine) {
    let context = Context::create();
    let module = context.create_module("my_module");
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);

    module.add_function("my_fn", fn_type, None);

    let buffer = target_machine
        .write_to_memory_buffer(&module, FileType::Assembly)
        .unwrap();

    assert!(!buffer.get_size() > 0);

    let string = from_utf8(buffer.as_slice()).unwrap();

    assert!(string.contains(".text"));
    assert!(string.contains(".file"));
    assert!(string.contains("my_module"));
    assert!(string.contains(".section"));
}

// REVIEW: Inconsistently failing on different tries :(
// #[test]
// fn test_target() {
//     // REVIEW: Some of the machine specific stuff may vary. Should allow multiple possibilities
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
    Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");

    let bad_target = Target::from_name("asd");

    assert!(bad_target.is_none());

    let bad_target2 = Target::from_triple(&TargetTriple::create("sadas"));

    #[cfg(any(feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0", feature = "llvm7-0"))]
    assert_eq!(
        bad_target2.unwrap_err().to_string(),
        "No available targets are compatible with this triple."
    );
    #[cfg(not(any(feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0", feature = "llvm7-0")))]
    assert_eq!(
        bad_target2.unwrap_err().to_string(),
        "No available targets are compatible with triple \"sadas\""
    );

    Target::initialize_x86(&InitializationConfig::default());

    let good_target = Target::from_name("x86-64");

    assert!(good_target.is_some());

    let good_target2 = Target::from_triple(&TargetTriple::create("x86_64-pc-linux-gnu"));

    assert!(good_target2.is_ok(), "{}", good_target2.unwrap_err());

    let good_target = good_target.unwrap();
    let good_target2 = good_target2.unwrap();

    assert_eq!(good_target, good_target2);
    assert_eq!(good_target.get_name().to_str(), Ok("x86-64"));
    assert_eq!(good_target2.get_name().to_str(), Ok("x86-64"));
    assert_eq!(
        good_target.get_description().to_str(),
        Ok("64-bit X86: EM64T and AMD64")
    );
    assert_eq!(
        good_target2.get_description().to_str(),
        Ok("64-bit X86: EM64T and AMD64")
    );
    assert!(good_target.has_jit());
    assert!(good_target2.has_jit());
    assert!(good_target.has_target_machine());
    assert!(good_target2.has_target_machine());
    assert!(good_target.has_asm_backend());
    assert!(good_target2.has_asm_backend());

    let next_target = good_target.get_next().unwrap();

    assert_eq!(next_target.get_name().to_str(), Ok("x86"));

    let target_machine = good_target
        .create_target_machine(
            &TargetTriple::create("x86_64-pc-linux-gnu"),
            "x86-64",
            "+avx2",
            OptimizationLevel::Default,
            RelocMode::Default,
            CodeModel::Default,
        )
        .unwrap();

    // TODO: Test target_machine failure

    target_machine.set_asm_verbosity(true);

    let triple = target_machine.get_triple();

    assert_eq!(target_machine.get_target(), good_target);
    assert_eq!(triple.as_str().to_str(), Ok("x86_64-pc-linux-gnu"));
    assert_eq!(target_machine.get_cpu().to_str(), Ok("x86-64"));
    assert_eq!(target_machine.get_feature_string().to_str(), Ok("+avx2"));

    #[cfg(not(any(feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0")))]
    {
        // TODO: Try and find a triple that actually gets normalized..
        assert_eq!(
            TargetMachine::normalize_triple(&triple).as_str().to_str(),
            Ok("x86_64-pc-linux-gnu"),
        );

        let _host_name = TargetMachine::get_host_cpu_name();
        let _host_cpu_features = TargetMachine::get_host_cpu_features();
    }
}

#[test]
fn test_default_triple() {
    let default_triple = TargetMachine::get_default_triple();
    let default_triple = default_triple.as_str().to_string_lossy();

    let archs = ["x86_64", "arm64"];
    let has_known_arch = archs.iter().any(|arch| default_triple.starts_with(*arch));
    assert!(has_known_arch, "Target triple '{default_triple}' has unknown arch");

    let vendors = if cfg!(target_os = "linux") {
        vec!["pc", "unknown", "redhat"]
    } else if cfg!(target_os = "macos") {
        vec!["apple"]
    } else {
        vec![]
    };

    let has_known_vendor = vendors.iter().any(|vendor| default_triple.contains(*vendor));
    assert!(has_known_vendor, "Target triple '{default_triple}' has unknown vendor");

    let os = [
        #[cfg(target_os = "linux")]
        "linux",
        #[cfg(target_os = "macos")]
        "darwin",
    ];
    let has_known_os = os.iter().any(|os| default_triple.contains(*os));
    assert!(has_known_os, "Target triple '{default_triple}' has unknown OS");

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

    // https://llvm.org/docs/LangRef.html#data-layout
    let datalayout_specification_re = Regex::new("[Ee]|S\\d+|P\\d+|A\\d+|p(\\d+)?:\\d+:\\d+(:\\d+)?|i\\d+:\\d+(:\\d+)?|v\\d+:\\d+(:\\d+)?|f\\d+:\\d+(:\\d+)?|a:\\d+(:\\d)?|F[in]\\d+|m:[emoxw]|n\\d+(:\\d)*|ni:\\d+(:\\d)*").unwrap();
    for specification in data_layout.as_str().to_str().unwrap().split('-') {
        assert!(datalayout_specification_re.is_match(specification));
    }
    assert!(data_layout.as_str().to_str().unwrap().matches('-').count() > 2);

    // REVIEW: Why is llvm 3.9+ a %? 4.0 on travis doesn't have it, but does for me locally...
    // assert_eq!(module.get_data_layout().as_str(), &*CString::new("%").unwrap());

    module.set_data_layout(&data_layout);

    assert_eq!(*module.get_data_layout(), data_layout);

    let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    let f32_type = context.f32_type();
    let f64_type = context.f64_type();
    let struct_type = context.struct_type(
        &[i32_type.into(), i64_type.into(), f64_type.into(), f32_type.into()],
        false,
    );
    let struct_type2 = context.struct_type(
        &[f32_type.into(), i32_type.into(), i64_type.into(), f64_type.into()],
        false,
    );

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
    assert_eq!(target_data.element_at_offset(&struct_type, u64::MAX), 3); // OoB; Odd as it seems to cap at max element number

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
    assert_eq!(target_data.element_at_offset(&struct_type2, u64::MAX), 3); // OoB; TODOC: Odd but seems to cap at max element number

    TargetData::create("e-m:e-i64:64-f80:128-n8:16:32:64-S128");
}

#[test]
fn test_ptr_sized_int() {
    Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");

    let context = Context::create();
    let module = context.create_module("sum");
    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
    let target_data = execution_engine.get_target_data();
    let address_space = AddressSpace::from(1u16);
    let int_type = context.ptr_sized_int_type(target_data, None);

    assert_eq!(int_type.get_bit_width(), target_data.get_pointer_byte_size(None) * 8);

    let int_type2 = context.ptr_sized_int_type(target_data, Some(address_space));

    assert_eq!(
        int_type2.get_bit_width(),
        target_data.get_pointer_byte_size(Some(address_space)) * 8
    );

    let int_type3 = context.ptr_sized_int_type(target_data, None);

    assert_eq!(int_type3.get_context(), context);
    assert_eq!(int_type3.get_bit_width(), target_data.get_pointer_byte_size(None) * 8);

    let int_type4 = context.ptr_sized_int_type(target_data, Some(address_space));

    assert_eq!(int_type4.get_context(), context);
    assert_eq!(
        int_type4.get_bit_width(),
        target_data.get_pointer_byte_size(Some(address_space)) * 8
    );
}

#[test]
fn test_write_target_machine_to_file() {
    Target::initialize_x86(&InitializationConfig::default());

    let target = Target::from_name("x86-64").unwrap();
    let target_machine = target
        .create_target_machine(
            &TargetTriple::create("x86_64-pc-linux-gnu"),
            "x86-64",
            "+avx2",
            OptimizationLevel::Less,
            RelocMode::Static,
            CodeModel::Small,
        )
        .unwrap();
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
    let target_machine = target
        .create_target_machine(
            &TargetTriple::create("x86_64-pc-linux-gnu"),
            "x86-64",
            "+avx2",
            OptimizationLevel::Aggressive,
            RelocMode::PIC,
            CodeModel::Medium,
        )
        .unwrap();

    write_target_machine_to_memory_buffer(target_machine);
}

#[llvm_versions(18..)]
#[test]
fn test_create_target_machine_from_default_options() {
    Target::initialize_x86(&InitializationConfig::default());

    let triple = TargetTriple::create("x86_64-pc-linux-gnu");
    let target = Target::from_triple(&triple).unwrap();
    let options = Default::default();

    let target_machine = target.create_target_machine_from_options(&triple, options).unwrap();

    assert_eq!(target_machine.get_cpu().to_str(), Ok(""));
    assert_eq!(target_machine.get_feature_string().to_str(), Ok(""));

    write_target_machine_to_memory_buffer(target_machine);
}

#[llvm_versions(18..)]
#[test]
fn test_create_target_machine_from_options() {
    Target::initialize_x86(&InitializationConfig::default());

    let triple = TargetTriple::create("x86_64-pc-linux-gnu");
    let target = Target::from_triple(&triple).unwrap();
    let options = inkwell::targets::TargetMachineOptions::new()
        .set_cpu("x86-64")
        .set_features("+avx2")
        .set_abi("sysv")
        .set_level(OptimizationLevel::Aggressive)
        .set_reloc_mode(RelocMode::PIC)
        .set_code_model(CodeModel::JITDefault);

    let target_machine = target.create_target_machine_from_options(&triple, options).unwrap();

    assert_eq!(target_machine.get_cpu().to_str(), Ok("x86-64"));
    assert_eq!(target_machine.get_feature_string().to_str(), Ok("+avx2"));

    write_target_machine_to_memory_buffer(target_machine);
}
