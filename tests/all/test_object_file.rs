extern crate inkwell;

use self::inkwell::context::Context;
use self::inkwell::module::Module;
use self::inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use self::inkwell::types::IntType;
use self::inkwell::values::BasicValue;
use self::inkwell::OptimizationLevel;

#[llvm_versions(7.0..=latest)]
fn get_host_cpu_name() -> String {
    TargetMachine::get_host_cpu_name().to_string()
}
#[llvm_versions(7.0..=latest)]
fn get_host_cpu_features() -> String {
    TargetMachine::get_host_cpu_features().to_string()
}
#[llvm_versions(4.0..=latest)]
fn ptr_sized_int_type<'ctx>(
    target_machine: &TargetMachine,
    context: &'ctx Context,
) -> IntType<'ctx> {
    let target_data = target_machine.get_target_data();
    context.ptr_sized_int_type(&target_data, None)
}
#[llvm_versions(4.0..=latest)]
fn apply_target_to_module<'ctx>(target_machine: &TargetMachine, module: &Module) {
    module.set_triple(&target_machine.get_triple());
    module.set_data_layout(&target_machine.get_target_data().get_data_layout());
}

#[llvm_versions(3.6..7.0)]
fn get_host_cpu_name() -> String {
    "".to_string()
}
#[llvm_versions(3.6..7.0)]
fn get_host_cpu_features() -> String {
    "".to_string()
}
#[llvm_versions(3.6..4.0)]
fn ptr_sized_int_type<'ctx>(
    _target_machine: &TargetMachine,
    context: &'ctx Context,
) -> IntType<'ctx> {
    context.i64_type()
}
#[llvm_versions(3.6..4.0)]
fn apply_target_to_module(target_machine: &TargetMachine, module: &Module) {
    module.set_triple(&target_machine.get_triple());
}

fn get_native_target_machine() -> TargetMachine {
    Target::initialize_native(&InitializationConfig::default())
        .expect("Failed to initialize native target");
    let target_triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&target_triple).unwrap();
    target
        .create_target_machine(
            &target_triple,
            &get_host_cpu_name(),
            &get_host_cpu_features(),
            OptimizationLevel::None,
            RelocMode::Default,
            CodeModel::Default,
        )
        .unwrap()
}

#[test]
fn test_section_iterator() {
    let target_machine = get_native_target_machine();

    let context = Context::create();
    let mut module = context.create_module("test_section_iterator");

    let gv_a = module.add_global(context.i8_type(), None, "a");
    gv_a.set_initializer(&context.i8_type().const_zero().as_basic_value_enum());
    gv_a.set_section("A");

    let gv_b = module.add_global(context.i16_type(), None, "b");
    gv_b.set_initializer(&context.i16_type().const_zero().as_basic_value_enum());
    gv_b.set_section("B");

    let gv_c = module.add_global(context.i32_type(), None, "c");
    gv_c.set_initializer(&context.i32_type().const_zero().as_basic_value_enum());
    gv_c.set_section("C");

    apply_target_to_module(&target_machine, &module);

    let memory_buffer = target_machine
        .write_to_memory_buffer(&mut module, FileType::Object)
        .unwrap();
    let object_file = memory_buffer.create_object_file().unwrap();

    let mut has_section_a = false;
    let mut has_section_b = false;
    let mut has_section_c = false;
    for section in object_file.get_sections() {
        if let Some(name) = section.get_name() {
            match name.to_str().unwrap() {
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
                    assert_eq!(section.size(), 4);
                }
                _ => {}
            }
        }
    }
    assert!(has_section_a);
    assert!(has_section_b);
    assert!(has_section_c);
}

#[test]
fn test_symbol_iterator() {
    let target_machine = get_native_target_machine();

    let context = Context::create();
    let mut module = context.create_module("test_symbol_iterator");
    module
        .add_global(context.i8_type(), None, "a")
        .set_initializer(&context.i8_type().const_zero().as_basic_value_enum());
    module
        .add_global(context.i16_type(), None, "b")
        .set_initializer(&context.i16_type().const_zero().as_basic_value_enum());
    module
        .add_global(context.i32_type(), None, "c")
        .set_initializer(&context.i32_type().const_zero().as_basic_value_enum());
    apply_target_to_module(&target_machine, &module);

    let memory_buffer = target_machine
        .write_to_memory_buffer(&mut module, FileType::Object)
        .unwrap();
    let object_file = memory_buffer.create_object_file().unwrap();

    let mut has_symbol_a = false;
    let mut has_symbol_b = false;
    let mut has_symbol_c = false;
    for symbol in object_file.get_symbols() {
        if let Some(name) = symbol.get_name() {
            match name.to_str().unwrap() {
                "a" => {
                    assert!(!has_symbol_a);
                    has_symbol_a = true;
                    assert_eq!(symbol.size(), 1);
                }
                "b" => {
                    assert!(!has_symbol_b);
                    has_symbol_b = true;
                    assert_eq!(symbol.size(), 2);
                }
                "c" => {
                    assert!(!has_symbol_c);
                    has_symbol_c = true;
                    assert_eq!(symbol.size(), 4);
                }
                _ => {}
            }
        }
    }
    assert!(has_symbol_a);
    assert!(has_symbol_b);
    assert!(has_symbol_c);
}

#[test]
fn test_reloc_iterator() {
    let target_machine = get_native_target_machine();

    let context = Context::create();
    let intptr_t = ptr_sized_int_type(&target_machine, &context);

    let mut module = context.create_module("test_reloc_iterator");
    let x_ptr = module
        .add_global(context.i8_type(), None, "x")
        .as_pointer_value();
    let x_plus_4 = x_ptr
        .const_to_int(intptr_t)
        .const_add(intptr_t.const_int(4, false));
    module
        .add_global(intptr_t, None, "a")
        .set_initializer(&x_plus_4);

    apply_target_to_module(&target_machine, &module);

    let memory_buffer = target_machine
        .write_to_memory_buffer(&mut module, FileType::Object)
        .unwrap();
    let object_file = memory_buffer.create_object_file().unwrap();

    let mut found_relocation = false;
    for section in object_file.get_sections() {
        for _ in section.get_relocations() {
            found_relocation = true;
            // We don't stop the traversal here, so as to exercise the iterators.
        }
    }
    assert!(found_relocation);
}

#[test]
fn test_section_contains_nul() {
    let target_machine = get_native_target_machine();

    let context = Context::create();
    let mut module = context.create_module("test_section_iterator");

    let gv = module.add_global(context.i32_type(), None, "gv");
    gv.set_initializer(
        &context
            .i32_type()
            .const_int(0xff0000ff, false)
            .as_basic_value_enum(),
    );
    gv.set_section("test");

    apply_target_to_module(&target_machine, &module);

    let memory_buffer = target_machine
        .write_to_memory_buffer(&mut module, FileType::Object)
        .unwrap();
    let object_file = memory_buffer.create_object_file().unwrap();

    let mut has_section_test = false;
    for section in object_file.get_sections() {
        if section.get_name().and_then(|name| name.to_str().ok()) == Some("test") {
            assert_eq!(section.get_contents(), 0xff0000ffu32.to_ne_bytes());
            has_section_test = true;
            break;
        }
    }
    assert!(has_section_test);
}
