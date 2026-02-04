use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::object_file::LLVMBinaryType;
use inkwell::targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine, TargetTriple};
use inkwell::types::IntType;
use inkwell::values::BasicValue;
use inkwell::OptimizationLevel;

fn ptr_sized_int_type<'ctx>(target_machine: &TargetMachine, context: &'ctx Context) -> IntType<'ctx> {
    let target_data = target_machine.get_target_data();
    context.ptr_sized_int_type(&target_data, None)
}

fn apply_target_to_module(target_machine: &TargetMachine, module: &Module) {
    module.set_triple(&target_machine.get_triple());
    module.set_data_layout(&target_machine.get_target_data().get_data_layout());
}

fn get_x86_64_target_machine() -> TargetMachine {
    Target::initialize_x86(&InitializationConfig::default());
    let target_triple = TargetTriple::create("x86_64-pc-linux-gnu");
    let target = Target::from_triple(&target_triple).unwrap();
    target
        .create_target_machine(
            &target_triple,
            "x86-64",
            "",
            OptimizationLevel::None,
            RelocMode::PIC,
            CodeModel::Default,
        )
        .unwrap()
}

#[test]
fn test_binary_file() {
    let target_machine = get_x86_64_target_machine();
    let context = Context::create();
    let module = context.create_module("test_binary_file");

    apply_target_to_module(&target_machine, &module);

    let memory_buffer = target_machine
        .write_to_memory_buffer(&module, FileType::Object)
        .unwrap();
    let binary_file = memory_buffer.create_binary_file(Some(&context)).unwrap();

    assert!(matches!(
        binary_file.get_binary_type(),
        LLVMBinaryType::LLVMBinaryTypeELF64L
    ));

    let memory_buffer_view = binary_file.get_memory_buffer();

    assert_eq!(memory_buffer.as_slice(), memory_buffer_view.as_slice());
}

#[test]
fn test_sections() {
    let target_machine = get_x86_64_target_machine();
    let context = Context::create();
    let module = context.create_module("test_sections");

    let gv_a = module.add_global(context.i8_type(), None, "a");
    gv_a.set_initializer(&context.i8_type().const_zero().as_basic_value_enum());
    assert!(gv_a.get_section().is_none());
    gv_a.set_section(Some("A"));
    assert_eq!(gv_a.get_section().unwrap().to_str(), Ok("A"));

    let gv_b = module.add_global(context.i16_type(), None, "b");
    gv_b.set_initializer(&context.i16_type().const_zero().as_basic_value_enum());
    assert!(gv_b.get_section().is_none());
    gv_b.set_section(Some("B"));
    assert_eq!(gv_b.get_section().unwrap().to_str(), Ok("B"));

    let gv_c = module.add_global(context.i32_type(), None, "c");
    gv_c.set_initializer(&context.i32_type().const_zero().as_basic_value_enum());
    assert!(gv_c.get_section().is_none());
    gv_c.set_section(Some("C"));
    assert_eq!(gv_c.get_section().unwrap().to_str(), Ok("C"));

    let func = module.add_function("d", context.void_type().fn_type(&[], false), None);

    assert!(func.get_section().is_none());

    func.set_section(Some("D"));

    assert_eq!(func.get_section().unwrap().to_str(), Ok("D"));

    // add a body to the function to make the section non-empty
    let basic_block = context.append_basic_block(func, "entry");
    let builder = context.create_builder();
    builder.position_at_end(basic_block);
    builder.build_return(None).unwrap();

    apply_target_to_module(&target_machine, &module);

    let memory_buffer = target_machine
        .write_to_memory_buffer(&module, FileType::Object)
        .unwrap();
    let binary_file = memory_buffer.create_binary_file(None).unwrap();

    let mut has_section_a = false;
    let mut has_section_b = false;
    let mut has_section_c = false;
    let mut has_section_d = false;
    let mut sections = binary_file.get_sections().unwrap();

    while let Some(section) = sections.next_section() {
        if let Some(name) = section.get_name() {
            match name.to_str().unwrap() {
                "A" => {
                    assert!(!has_section_a);
                    has_section_a = true;
                    assert_eq!(section.get_size(), 1);
                },
                "B" => {
                    assert!(!has_section_b);
                    has_section_b = true;
                    assert_eq!(section.get_size(), 2);
                },
                "C" => {
                    assert!(!has_section_c);
                    has_section_c = true;
                    assert_eq!(section.get_size(), 4);
                },
                "D" => {
                    assert!(!has_section_d);
                    has_section_d = true;
                    assert_eq!(section.get_size(), 1);
                },
                _ => {},
            }
        }
    }
    assert!(has_section_a);
    assert!(has_section_b);
    assert!(has_section_c);
    assert!(has_section_d);
}

#[test]
fn test_symbols() {
    let target_machine = get_x86_64_target_machine();
    let context = Context::create();
    let module = context.create_module("test_symbols");

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
        .write_to_memory_buffer(&module, FileType::Object)
        .unwrap();
    let binary_file = memory_buffer.create_binary_file(None).unwrap();

    let mut has_symbol_a = false;
    let mut has_symbol_b = false;
    let mut has_symbol_c = false;
    let mut symbols = binary_file.get_symbols().unwrap();

    while let Some(symbol) = symbols.next_symbol() {
        if let Some(name) = symbol.get_name() {
            match name.to_str().unwrap() {
                "a" => {
                    assert!(!has_symbol_a);
                    has_symbol_a = true;
                    assert_eq!(symbol.get_size(), 1);
                },
                "b" => {
                    assert!(!has_symbol_b);
                    has_symbol_b = true;
                    assert_eq!(symbol.get_size(), 2);
                },
                "c" => {
                    assert!(!has_symbol_c);
                    has_symbol_c = true;
                    assert_eq!(symbol.get_size(), 4);
                },
                _ => {},
            }
        }
    }

    assert!(has_symbol_a);
    assert!(has_symbol_b);
    assert!(has_symbol_c);
}

#[test]
fn test_relocations() {
    let target_machine = get_x86_64_target_machine();
    let context = Context::create();
    let module = context.create_module("test_relocations");

    let intptr_t = ptr_sized_int_type(&target_machine, &context);
    let x_ptr = module.add_global(context.i8_type(), None, "x").as_pointer_value();
    let x_plus_4 = x_ptr.const_to_int(intptr_t).const_add(intptr_t.const_int(4, false));
    let y_ptr = module.add_global(context.i8_type(), None, "y").as_pointer_value();
    let y_plus_4 = y_ptr.const_to_int(intptr_t).const_add(intptr_t.const_int(4, false));
    module.add_global(intptr_t, None, "a").set_initializer(&x_plus_4);
    module.add_global(intptr_t, None, "b").set_initializer(&y_plus_4);
    module.add_global(intptr_t, None, "c").set_initializer(&x_plus_4);
    module.add_global(intptr_t, None, "d").set_initializer(&y_plus_4);

    apply_target_to_module(&target_machine, &module);

    let memory_buffer = target_machine
        .write_to_memory_buffer(&module, FileType::Object)
        .unwrap();

    let binary_file = memory_buffer.create_binary_file(Some(&context)).unwrap();

    let mut x_count = 0;
    let mut y_count = 0;
    let mut sections = binary_file.get_sections().unwrap();

    while let Some(section) = sections.next_section() {
        let mut relocations = section.get_relocations();
        let mut offset = 0;

        while let Some(relocation) = relocations.next_relocation() {
            assert_eq!(relocation.get_type().0, 1);
            assert_eq!(*relocation.get_type().1, c"R_X86_64_64");
            assert_eq!(relocation.get_offset(), offset);
            offset += 8;

            let symbol = relocation.get_symbol();
            let name = symbol.get_name().unwrap();

            if name == c"x" {
                x_count += 1;
            } else if name == c"y" {
                y_count += 1;
            } else {
                panic!("unexpected relocation symbol name");
            }
        }
    }

    assert_eq!(x_count, 2);
    assert_eq!(y_count, 2);
}
