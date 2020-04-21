extern crate inkwell;

use self::inkwell::context::Context;
use self::inkwell::module::Module;
use self::inkwell::targets::{
    ByteOrdering, CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetData,
    TargetMachine, TargetTriple,
};
use self::inkwell::OptimizationLevel;

#[test]
fn test_section_iterator() {
    Target::initialize_native(&InitializationConfig::default())
        .expect("Failed to initialize native target");
    let target_triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&target_triple).unwrap();
    let target_machine = target
        .create_target_machine(
            &target_triple,
            &TargetMachine::get_host_cpu_name().to_string(),
            &TargetMachine::get_host_cpu_features().to_string(),
            OptimizationLevel::None,
            RelocMode::Static,
            CodeModel::Small,
        )
        .unwrap();

    let context = Context::create();
    let mut module = context.create_module("my_module");
    module.set_inline_assembly(
        ".section A\n\
        .byte 1\n\
        .section B\n\
        .byte 2, 2\n\
        .section C\n\
        .byte 3, 3, 3",
    );
    module.set_triple(&target_triple);
    module.set_data_layout(&target_machine.get_target_data().get_data_layout());

    let memory_buffer = target_machine
        .write_to_memory_buffer(&mut module, FileType::Object)
        .unwrap();
    let object_file = memory_buffer.create_object_file().unwrap();

    let mut has_section_a = false;
    let mut has_section_b = false;
    let mut has_section_c = false;
    for (i, section) in object_file.get_sections().enumerate() {
        // TODO: the first section has no name, skip it.
        if i == 0 {
            continue;
        }
        match section.get_name().to_str().unwrap() {
            "A" => {
                assert!(!has_section_a);
                has_section_a = true;
                assert_eq!(section.size(), 1);
            }
            "B" => {
                assert!(!has_section_b);
                has_section_b = true;
                assert_eq!(section.size(), 2);
            }
            "C" => {
                assert!(!has_section_c);
                has_section_c = true;
                assert_eq!(section.size(), 3);
            }
            _ => {}
        }
    }
    assert!(has_section_a);
    assert!(has_section_b);
    assert!(has_section_c);
}
