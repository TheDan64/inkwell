use std::{
    collections::HashSet,
    ffi::CStr,
    iter::{self, FromIterator},
    ops::Deref,
};

#[llvm_versions(12.0..=latest)]
use inkwell::orc2::{
    lljit::ObjectLinkingLayerCreator, CLookupSet, EvaluatedSymbol, JITDylibLookupFlags, LookupKind,
    LookupState, MaterializationUnit, ObjectLayer, SymbolFlags, SymbolMapPair, SymbolMapPairs,
};
#[llvm_versions(13.0..=latest)]
use inkwell::orc2::{
    lljit::ObjectTransformer, IRTransformLayer, MaterializationResponsibility, Materializer,
    SymbolFlagsMapPair, SymbolFlagsMapPairs,
};
use inkwell::{
    builder::Builder,
    context::Context,
    error::LLVMError,
    memory_buffer::{MemoryBuffer, MemoryBufferRef},
    module::{Linkage, Module},
    orc2::{
        lljit::{LLJITBuilder, LLJIT},
        Wrapper, CAPIDefinitionGenerator, DefinitionGenerator, DefinitionGeneratorRef,
        ExecutionSession, JITDylib, JITTargetMachineBuilder, SymbolStringPoolEntry,
        ThreadSafeContext, ThreadSafeModule,
    },
    support::LLVMString,
    targets::{CodeModel, FileType, RelocMode, Target, TargetMachine},
    values::{FunctionValue, IntValue},
    OptimizationLevel,
};

#[test]
fn test_drop_lljit_before_function_use() {
    let thread_safe_context = ThreadSafeContext::create();
    let module = constant_function_module(&thread_safe_context, 64, "main");
    let lljit = LLJIT::create().expect("LLJIT::create failed");
    let main_jd = lljit.get_main_jit_dylib();
    lljit
        .add_module(&main_jd, module)
        .expect("LLJIT::add_module failed");
    drop(thread_safe_context);
    drop(main_jd);
    test_main_function(&lljit);
}

#[llvm_versions(12.0..=latest)]
#[test]
fn test_lljit_add_module_with_rt_default() {
    let thread_safe_context = ThreadSafeContext::create();
    let module = constant_function_module(&thread_safe_context, 64, "main");
    let lljit = LLJIT::create().expect("LLJIT::create failed");
    let main_jd = lljit.get_main_jit_dylib();
    let module_rt = main_jd.create_resource_tracker();
    lljit
        .add_module_with_rt(&module_rt, module)
        .expect("LLJIT::add_module_with_rt failed");
    drop(thread_safe_context);
    drop(main_jd);
    drop(module_rt);
    test_main_function(&lljit);
}

#[test]
fn test_multiple_lljit_instances_single_context() {
    let thread_safe_context = ThreadSafeContext::create();
    let module_1 = constant_function_module(&thread_safe_context, 64, "main_1");
    let module_2 = constant_function_module(&thread_safe_context, 42, "main_2");
    let lljit_1 = LLJIT::create().expect("LLJIT::create failed");
    let lljit_2 = LLJIT::create().expect("LLJIT::create failed");
    let main_jd_1 = lljit_1.get_main_jit_dylib();
    let main_jd_2 = lljit_2.get_main_jit_dylib();
    lljit_1
        .add_module(&main_jd_1, module_1)
        .expect("LLJIT::add_module failed");
    lljit_2
        .add_module(&main_jd_2, module_2)
        .expect("LLJIT::add_module failed");
    drop(thread_safe_context);
    drop(main_jd_1);
    unsafe {
        let function_1 = lljit_1
            .get_function::<unsafe extern "C" fn() -> u64>("main_1")
            .expect("LLJIT::get_function failed");
        let function_2 = lljit_2
            .get_function::<unsafe extern "C" fn() -> u64>("main_2")
            .expect("LLJIT::get_function failed");
        assert_eq!(function_1.call(), 64);
        assert_eq!(function_2.call(), 42);
    }
}

#[test]
fn test_lljit_multiple_contexts() {
    let thread_safe_context_1 = ThreadSafeContext::create();
    let module_1 = constant_function_module(&thread_safe_context_1, 64, "main_1");
    let lljit = LLJIT::create().expect("LLJIT::create failed");
    let main_jd = lljit.get_main_jit_dylib();
    lljit
        .add_module(&main_jd, module_1)
        .expect("LLJIT::add_module failed");
    let thread_safe_context_2 = ThreadSafeContext::create();
    let module_2 = constant_function_module(&thread_safe_context_2, 42, "main_2");
    lljit
        .add_module(&main_jd, module_2)
        .expect("LLJIT::add_module failed");
    drop(thread_safe_context_2);
    drop(main_jd);
    unsafe {
        let function_1 = lljit
            .get_function::<unsafe extern "C" fn() -> u64>("main_1")
            .expect("LLJIT::get_function failed");
        let function_2 = lljit
            .get_function::<unsafe extern "C" fn() -> u64>("main_2")
            .expect("LLJIT::get_function failed");
        assert_eq!(function_1.call(), 64);
        assert_eq!(function_2.call(), 42);
    }
}

#[llvm_versions(12.0..=latest)]
#[test]
fn test_lljit_remove_resource_tracker() {
    let thread_safe_context = ThreadSafeContext::create();
    let module = constant_function_module(&thread_safe_context, 64, "main");
    let lljit = LLJIT::create().expect("LLJIT::create failed");
    let main_jd = lljit.get_main_jit_dylib();
    let module_rt = main_jd.create_resource_tracker();
    lljit
        .add_module_with_rt(&module_rt, module)
        .expect("LLJIT::add_module_with_rt failed");
    unsafe {
        let function = lljit
            .get_function::<unsafe extern "C" fn() -> u64>("main")
            .expect("LLJIT::get_function failed");
        assert_eq!(function.call(), 64);
    }
    module_rt.remove().expect("ResourceTracker::remove failed");
    let module = constant_function_module(&thread_safe_context, 42, "main");
    let module_rt = main_jd.create_resource_tracker();
    lljit
        .add_module_with_rt(&module_rt, module)
        .expect("LLJIT::add_module_with_rt failed");
    unsafe {
        let function = lljit
            .get_function::<unsafe extern "C" fn() -> u64>("main")
            .expect("LLJIT::get_function failed");
        assert_eq!(function.call(), 42);
    }
}

#[test]
fn test_lljit_add_object_file() {
    let context = Context::create();
    let object_file = constant_function_object_file(&context, 64, "main");
    drop(context);
    let lljit = LLJIT::create().expect("LLJIT::create failed");
    let main_jd = lljit.get_main_jit_dylib();
    lljit
        .add_object_file(&main_jd, object_file)
        .expect("LLJIT::add_object_file failed");
    drop(main_jd);
    test_main_function(&lljit);
}

#[llvm_versions(12.0..=latest)]
#[test]
fn test_lljit_add_object_file_with_rt() {
    let context = Context::create();
    let object_file = constant_function_object_file(&context, 64, "main");
    drop(context);
    let lljit = LLJIT::create().expect("LLJIT::create failed");
    let main_jd = lljit.get_main_jit_dylib();
    let module_rt = main_jd.create_resource_tracker();
    drop(main_jd);
    lljit
        .add_object_file_with_rt(&module_rt, object_file)
        .expect("LLJIT::add_object_file_with_rt failed");
    drop(module_rt);
    test_main_function(&lljit);
}

#[llvm_versions(13.0..=latest)]
#[test]
fn test_object_linking_layer_add_object_file() {
    let lljit = LLJIT::create().expect("LLJIT::create failed");
    let object_linking_layer = lljit.get_object_linking_layer();
    let context = Context::create();
    let object_file = constant_function_object_file(&context, 64, "main");
    drop(context);
    let main_jd = lljit.get_main_jit_dylib();
    object_linking_layer
        .add_object_file(&main_jd, object_file)
        .expect("ObjectLayer::add_object_file failed");
    drop(main_jd);
    drop(object_linking_layer);
    test_main_function(&lljit);
}

// #[llvm_versions(14.0..=latest)]
// #[test]
// fn test_object_linking_layer_add_object_file_with_rt() {
//     let lljit = LLJIT::create().expect("LLJIT::create failed");
//     let object_linking_layer = lljit.get_object_linking_layer();
//     let context = Context::create();
//     let object_file = constant_function_object_file(&context, 64, "main");
//     drop(context);
//     let main_jd = lljit.get_main_jit_dylib();
//     let module_rt = main_jd.create_resource_tracker();
//     drop(main_jd);
//     object_linking_layer.add_object_file_with_rt(&module_rt, object_file)
//         .expect("ObjectLayer::add_object_file_with_rt failed");
//     drop(object_linking_layer);
//     drop(module_rt);
//     test_main_function(&lljit);
// }

#[cfg(target_os = "linux")]
#[test]
fn test_lljit_get_global_prefix() {
    let lljit = LLJIT::create().expect("LLJIT::create failed");
    assert_eq!(lljit.get_global_prefix(), '\0');
}

// TODO: Figure out linking of JITDylibs
// #[test]
// fn test_lljit_replace_function() {
//     let thread_safe_context = ThreadSafeContext::create();
//     let foo_module =
//         constant_function_module(&thread_safe_context, 64, "foo").expect("invalid test module");
//     let bar_module = ModuleBuilder::new(&thread_safe_context, "bar")
//         .add_contstant_function("bar", 42)
//         .add_sum_function("sum", "foo", "bar")
//         .build()
//         .expect("invalid test module");
//     let lljit = LLJIT::create().expect("LLJIT::create failed");
//     let main_jd = lljit.get_main_jit_dylib();
//     let execution_session = lljit.get_execution_session();
//     let foo_jd = execution_session
//         .create_jit_dylib("foo")
//         .expect("ExecutionSession::create_jit_dylib failed");
//     let foo_rt = foo_jd.create_resource_tracker();
//     lljit
//         .add_module_with_rt(&foo_rt, foo_module)
//         .expect("LLJIT::add_module_with_rt failed");
//     let bar_rt = main_jd.create_resource_tracker();
//     lljit
//         .add_module_with_rt(&bar_rt, bar_module)
//         .expect("LLJIT::add_module_with_rt failed");
//     unsafe {
//         let function = lljit
//             .get_function::<unsafe extern "C" fn() -> u64>("sum")
//             .expect("LLJIT::get_function failed");
//         assert_eq!(function.call(), 42 + 64);
//     }

//     foo_rt.remove().expect("ResourceTracker::remove failed");
//     let foo_module =
//         constant_function_module(&thread_safe_context, 32, "foo").expect("invalid test module");
//     let foo_rt = foo_jd.create_resource_tracker();
//     lljit
//         .add_module_with_rt(&foo_rt, foo_module)
//         .expect("LLJIT::add_module_with_rt failed");
//     unsafe {
//         let function = lljit
//             .get_function::<unsafe extern "C" fn() -> u64>("sum")
//             .expect("LLJIT::get_function failed");
//         // assert_eq!(function.call(), 42 + 32);
//     }
// }

#[test]
fn test_default_lljit_builder() {
    let lljit = LLJITBuilder::create()
        .build()
        .expect("LLJITBuilder::build failed");
    test_basic_lljit_functionality(lljit);
}

#[test]
fn test_lljit_builder_set_jit_target_machine_detect_host() {
    let jit_target_machine_builder = JITTargetMachineBuilder::detect_host()
        .expect("JITTargetMachineBuilder::detect_host failed");
    let lljit = LLJITBuilder::create()
        .set_jit_target_machine_builder(jit_target_machine_builder)
        .build()
        .expect("LLJITBuilder::build failed");
    test_basic_lljit_functionality(lljit);
}

#[llvm_versions(13.0..=latest)]
#[test]
fn test_lljit_builder_set_invalid_jit_target_machine() {
    let jit_target_machine_builder = JITTargetMachineBuilder::detect_host()
        .expect("JITTargetMachineBuilder::detect_host failed");
    jit_target_machine_builder.set_target_triple("invalid");
    let error = LLJITBuilder::create()
        .set_jit_target_machine_builder(jit_target_machine_builder)
        .build()
        .expect_err("LLJITBuilder::build succeeded");
    let error = error.expect_left("Error is not an LLVMError");
    assert_eq!(
        error.get_message().to_string(),
        "No available targets are compatible with triple \"invalid\""
    );
}

#[llvm_versions(12.0..=latest)]
#[test]
fn test_lljit_builder_set_object_linking_layer_creator() {
    let object_linking_layer_creator: Box<dyn ObjectLinkingLayerCreator> =
        Box::new(SimpleObjectLinkingLayerCreator {});
    let lljit = LLJITBuilder::create()
        .set_object_linking_layer_creator(object_linking_layer_creator)
        .build()
        .expect("LLJITBuilder::build failed");
    test_basic_lljit_functionality(lljit);
}

#[llvm_versions(12.0..=latest)]
#[test]
fn test_lljit_builder_set_object_linking_layer_creator_closure() {
    let object_linking_layer_creator: Box<dyn ObjectLinkingLayerCreator> =
        Box::new(|execution_session: ExecutionSession, _triple: &CStr| {
            execution_session
                .create_rt_dyld_object_linking_layer_with_section_memory_manager()
                .into()
        });
    let lljit = LLJITBuilder::create()
        .set_object_linking_layer_creator(object_linking_layer_creator)
        .build()
        .expect("LLJITBuilder::build failed");
    test_basic_lljit_functionality(lljit);
}

#[llvm_versions(12.0..=latest)]
#[derive(Debug)]
struct SimpleObjectLinkingLayerCreator {}

#[llvm_versions(12.0..=latest)]
impl ObjectLinkingLayerCreator for SimpleObjectLinkingLayerCreator {
    fn create_object_linking_layer(
        &mut self,
        execution_session: ExecutionSession,
        _triple: &CStr,
    ) -> ObjectLayer<'static> {
        execution_session
            .create_rt_dyld_object_linking_layer_with_section_memory_manager()
            .into()
    }
}

#[llvm_versions(12.0..=latest)]
#[test]
fn test_jit_target_machin_builder_create_from_target_machine() {
    let target_machine = get_native_target_machine();
    let jit_target_machine_builder =
        JITTargetMachineBuilder::create_from_target_machine(target_machine);
    let lljit = LLJITBuilder::create()
        .set_jit_target_machine_builder(jit_target_machine_builder)
        .build()
        .expect("LLJITBuilder::build failed");
    test_basic_lljit_functionality(lljit);
}

#[llvm_versions(13.0..=latest)]
#[test]
fn test_jit_target_machin_builder_set_target_triple() {
    let jit_target_machine_builder = JITTargetMachineBuilder::detect_host()
        .expect("JITTargetMachineBuilder::detect_host failed");
    let default_triple = TargetMachine::get_default_triple();
    let target_triple = default_triple
        .as_str()
        .to_str()
        .expect("TargetMachine::get_default_triple returned an invalid string");
    jit_target_machine_builder.set_target_triple(target_triple);
    let lljit = LLJITBuilder::create()
        .set_jit_target_machine_builder(jit_target_machine_builder)
        .build()
        .expect("LLJITBuilder::build failed");
    test_basic_lljit_functionality(lljit);
}

#[llvm_versions(12.0..=latest)]
#[test]
fn test_execution_session_create_rt_dyld_object_linking_layer_with_section_memory_manager() {
    let lljit = LLJIT::create().expect("LLJIT::create failed");
    let execution_session = lljit.get_execution_session();
    // leaks memory
    execution_session.create_rt_dyld_object_linking_layer_with_section_memory_manager();
}

#[llvm_versions(13.0..=latest)]
#[test]
fn test_object_transformer_modify_buffer() {
    let thread_safe_context = ThreadSafeContext::create();
    let module = constant_function_module(&thread_safe_context, 42, "main");
    let object_buffer = constant_function_object_file(thread_safe_context.context(), 64, "main");
    let lljit = LLJIT::create().expect("LLJIT::create failed");
    let object_transformer: Box<dyn ObjectTransformer> =
        Box::new(move |mut buffer: MemoryBufferRef| {
            buffer.set_memory_buffer(MemoryBuffer::create_from_memory_range_copy(
                object_buffer.as_slice(),
                "new memory buffer",
            ));
            Ok(())
        });
    lljit.set_object_transformer(object_transformer);
    let main_jd = lljit.get_main_jit_dylib();
    lljit
        .add_module(&main_jd, module)
        .expect("LLJIT::add_module failed");
    test_main_function(&lljit);
}

#[llvm_versions(13.0..=latest)]
#[test]
fn test_object_transform_layer_error() {
    let thread_safe_context = ThreadSafeContext::create();
    let module = constant_function_module(&thread_safe_context, 64, "main");
    let lljit = LLJIT::create().expect("LLJIT::create failed");
    let object_transformer: Box<dyn ObjectTransformer> =
        Box::new(|_buffer: MemoryBufferRef| Err(LLVMError::new_string_error("test error")));
    lljit.set_object_transformer(object_transformer);
    let main_jd = lljit.get_main_jit_dylib();
    lljit
        .add_module(&main_jd, module)
        .expect("LLJIT::add_module failed");
    test_main_function(&lljit);
}

#[llvm_versions(13.0..=latest)]
#[test]
fn test_materialization_unit_with_materializer() {
    let lljit = LLJIT::create().expect("LLJIT::create failed");
    let context = ThreadSafeContext::create();
    let ir_transform_layer = lljit.get_ir_transform_layer();

    let materializer = TestMaterializer {
        context: &context,
        ir_transform_layer,
    };

    test_test_materializer(&lljit, materializer);
}

#[llvm_versions(13.0..=latest)]
#[derive(Debug, Clone)]
struct TestMaterializer<'ctx, 'jit> {
    context: &'ctx ThreadSafeContext,
    ir_transform_layer: IRTransformLayer<'jit>,
}

#[llvm_versions(13.0..=latest)]
impl TestMaterializer<'_, '_> {
    fn define_symbols(lljit: &LLJIT) -> SymbolFlagsMapPairs {
        SymbolFlagsMapPairs::from_iter(
            vec!["main", "fourty_two"]
                .into_iter()
                .map(|name| lljit.mangle_and_intern(name))
                .zip(iter::repeat(SymbolFlags::new(0, 0))),
        )
    }
}

#[llvm_versions(13.0..=latest)]
impl<'ctx, 'jit> Materializer for TestMaterializer<'ctx, 'jit> {
    fn materialize(&mut self, materialization_responsibility: MaterializationResponsibility) {
        let mut module_builder = ModuleBuilder::new(self.context, "simple_materializer");
        for symbol in materialization_responsibility.get_symbols().names_iter() {
            let name = symbol.get_string().to_string_lossy();
            match name.deref() {
                "fourty_two" => module_builder = module_builder.add_contstant_function(&name, 42),
                _ => module_builder = module_builder.add_contstant_function(&name, 64),
            };
        }
        match module_builder.build() {
            Ok(module) => self
                .ir_transform_layer
                .emit(materialization_responsibility, module),
            Err(_) => materialization_responsibility.fail_materialization(),
        }
    }

    fn discard(&mut self, _jit_dylib: &JITDylib, _symbol: &SymbolStringPoolEntry) {}
}

#[llvm_versions(13.0..=latest)]
#[test]
fn test_materialization_responsibility_fail_materializing() {
    let lljit = LLJIT::create().expect("LLJIT::create failed");
    let materializer: Box<dyn Materializer> = Box::new((
        |materialization_responsibility: MaterializationResponsibility| {
            materialization_responsibility.fail_materialization();
        },
        |_jit_dylib: &JITDylib, _symbol: &SymbolStringPoolEntry| {},
    ));
    let materialization_unit = MaterializationUnit::create(
        "test materialization unit",
        SymbolFlagsMapPairs::new(vec![SymbolFlagsMapPair::new(
            lljit.mangle_and_intern("main"),
            SymbolFlags::new(0, 0),
        )]),
        None,
        materializer,
    );
    let main_jd = lljit.get_main_jit_dylib();
    main_jd
        .define(materialization_unit)
        .expect("JITDylib::define failed");

    let error = lljit
        .get_function_address("main")
        .expect_err("LLJIT::get_function_address did not fail");
    assert_eq!(
        error.get_message().to_string(),
        "Failed to materialize symbols: { (main, { main }) }"
    );
}

#[llvm_versions(13.0..=latest)]
#[test]
fn test_unused_materialization_unit() {
    let lljit = LLJIT::create().expect("LLJIT::create failed");
    let context = ThreadSafeContext::create();

    let ir_transform_layer = lljit.get_ir_transform_layer();
    let materializer = TestMaterializer {
        context: &context,
        ir_transform_layer,
    };

    let symbols = TestMaterializer::define_symbols(&lljit);
    let materialization_unit = MaterializationUnit::create(
        "test_materialization_unit",
        symbols,
        None,
        Box::new(materializer),
    );

    let main_jd = lljit.get_main_jit_dylib();
    main_jd
        .define(materialization_unit)
        .expect("JITDylib::define failed");
}

#[llvm_versions(13.0..=latest)]
#[test]
fn test_materialization_responsibility_replace() {
    let lljit = LLJIT::create().expect("LLJIT::create failed");
    let context = ThreadSafeContext::create();
    let ir_transform_layer = lljit.get_ir_transform_layer();

    let materializer = MaterializeOnlyRequested {
        static_initializer_symbol: None,
        materializer: TestMaterializer {
            context: &context,
            ir_transform_layer,
        },
    };

    test_test_materializer(&lljit, materializer);
}

#[llvm_versions(13.0..=latest)]
#[derive(Debug, Clone)]
struct MaterializeOnlyRequested<M> {
    materializer: M,
    static_initializer_symbol: Option<SymbolStringPoolEntry>,
}

#[llvm_versions(13.0..=latest)]
impl<M> Materializer for MaterializeOnlyRequested<M>
where
    M: Materializer + Clone,
{
    fn materialize(&mut self, materialization_responsibility: MaterializationResponsibility) {
        let requested_symbols: HashSet<_> = HashSet::from_iter(
            materialization_responsibility
                .get_requested_symbols()
                .as_ref()
                .into_iter()
                .cloned(),
        );
        let not_requested_symbols: Vec<_> = materialization_responsibility
            .get_symbols()
            .into_iter()
            .filter(|symbol| !requested_symbols.contains(symbol.get_name()))
            .collect();

        // Create new materialization unit for not requested symbols
        if !not_requested_symbols.is_empty() {
            let materialization_unit = MaterializationUnit::create(
                "compile_only_requested",
                SymbolFlagsMapPairs::new(not_requested_symbols),
                self.static_initializer_symbol.clone(),
                Box::new(self.clone()),
            );
            if let Err(error) = materialization_responsibility.replace(materialization_unit) {
                // Log error...
                eprintln!("{}", error.get_message());
                materialization_responsibility.fail_materialization();
                return;
            }
        }
        self.materializer
            .materialize(materialization_responsibility);
    }

    fn discard(&mut self, _jit_dylib: &JITDylib, _symbol: &SymbolStringPoolEntry) {}
}

#[llvm_versions(13.0..=latest)]
#[test]
fn test_materialization_responsibility_delegate() {
    let lljit = LLJIT::create().expect("LLJIT::create failed");
    let context = ThreadSafeContext::create();
    let ir_transform_layer = lljit.get_ir_transform_layer();

    let materializer = MultiThreadedMaterializer {
        materializer: TestMaterializer {
            context: &context,
            ir_transform_layer,
        },
    };

    test_test_materializer(&lljit, materializer);
}

#[llvm_versions(13.0..=latest)]
struct MultiThreadedMaterializer<M> {
    materializer: M,
}

#[llvm_versions(13.0..=latest)]
impl<M> Materializer for MultiThreadedMaterializer<M>
where
    M: Materializer + Clone,
{
    fn materialize(&mut self, materialization_responsibility: MaterializationResponsibility) {
        for symbol in materialization_responsibility.get_symbols() {
            let thread_responsibility =
                match materialization_responsibility.delegate(vec![symbol.name()]) {
                    Err(error) => {
                        eprintln!("{}", error.get_message());
                        materialization_responsibility.fail_materialization();
                        return;
                    }
                    Ok(ok) => ok,
                };
            let mut materializer = self.materializer.clone();
            materializer.materialize(thread_responsibility);
            // TODO: Make multithreaded
            // thread::spawn(move || {
            //     materializer.materialize(thread_responsibility);
            // });
        }
    }

    fn discard(&mut self, _jit_dylib: &JITDylib, _symboll: &SymbolStringPoolEntry) {}
}

#[llvm_versions(12.0..=latest)]
#[test]
fn test_jit_dylib_add_generator() {
    let thread_safe_context = ThreadSafeContext::create();
    let lljit = LLJIT::create().expect("LLJIT::create failed");
    let main_jd = lljit.get_main_jit_dylib();

    let mut capi_definition_generator = Wrapper::new(
        |definition_generator: DefinitionGeneratorRef,
         lookup_state: &mut LookupState,
         lookup_kind: LookupKind,
         jit_dylib: JITDylib,
         jit_dylib_lookup_flags: JITDylibLookupFlags,
         c_lookup_set: CLookupSet| {
            let module = constant_function_module(&thread_safe_context, 64, "main");
            lljit.add_module(&jit_dylib, module)
        },
    );
    let definition_generator = DefinitionGenerator::create_custom_capi_definition_generator(
        &mut capi_definition_generator,
    );

    main_jd.add_generator(definition_generator);
    test_main_function(&lljit);
}

#[llvm_versions(13.0..=latest)]
fn test_test_materializer(lljit: &LLJIT, materializer: impl Materializer) {
    let symbols = TestMaterializer::define_symbols(&lljit);
    let materialization_unit =
        MaterializationUnit::create("test_materializer", symbols, None, Box::new(materializer));

    let main_jd = lljit.get_main_jit_dylib();
    main_jd
        .define(materialization_unit)
        .expect("JITDylib::define failed");
    test_main_function(&lljit);
    unsafe {
        let function = lljit
            .get_function::<unsafe extern "C" fn() -> u64>("fourty_two")
            .expect("LLJIT::get_function failed");
        assert_eq!(function.call(), 42);
    }
}

fn test_basic_lljit_functionality(lljit: LLJIT) {
    let thread_safe_context = ThreadSafeContext::create();
    let module = constant_function_module(&thread_safe_context, 64, "main");
    let main_jd = lljit.get_main_jit_dylib();
    lljit
        .add_module(&main_jd, module)
        .expect("LLJIT::add_module failed");
    drop(thread_safe_context);
    drop(main_jd);
    test_main_function(&lljit);
}

fn test_main_function(lljit: &LLJIT) {
    unsafe {
        let function = lljit
            .get_function::<unsafe extern "C" fn() -> u64>("main")
            .expect("LLJIT::get_function failed");
        assert_eq!(function.call(), 64);
    }
}

#[derive(Debug)]
struct ModuleBuilder<'ctx> {
    thread_safe_context: &'ctx ThreadSafeContext,
    module: Module<'ctx>,
}
impl<'ctx> ModuleBuilder<'ctx> {
    fn new(thread_safe_context: &'ctx ThreadSafeContext, name: &str) -> Self {
        ModuleBuilder {
            thread_safe_context,
            module: thread_safe_context.context().create_module(name),
        }
    }

    fn build(self) -> Result<ThreadSafeModule<'ctx>, LLVMString> {
        self.module.verify()?;
        Ok(self.thread_safe_context.create_module(self.module))
    }

    fn add_contstant_function(self, name: &str, value: u64) -> Self {
        let context = self.thread_safe_context.context();
        let function_type = context.i64_type().fn_type(&vec![], false);
        let builder = context.create_builder();
        let function = self.module.add_function(name, function_type, None);
        let entry_bb = context.append_basic_block(function, "entry");
        builder.position_at_end(entry_bb);
        builder.build_return(Some(&context.i64_type().const_int(value, false)));
        self
    }

    fn add_sum_function(self, name: &str, value_func_1: &str, value_func_2: &str) -> Self {
        let context = self.thread_safe_context.context();
        let function_type = context.i64_type().fn_type(&vec![], false);
        let builder = context.create_builder();
        let function = self.module.add_function(name, function_type, None);
        let entry_bb = context.append_basic_block(function, "entry");
        builder.position_at_end(entry_bb);
        let value_1 = self.function_call_without_arguments(&builder, value_func_1);
        let value_2 = self.function_call_without_arguments(&builder, value_func_2);
        let sum: IntValue = builder.build_int_add(value_1, value_2, "sum");
        builder.build_return(Some(&sum));
        self
    }

    fn lookup_function_without_arguments(&self, name: &str) -> FunctionValue<'ctx> {
        match self.module.get_function(name) {
            Some(function) => function,
            None => {
                let function_type = self
                    .thread_safe_context
                    .context()
                    .i64_type()
                    .fn_type(&vec![], false);
                self.module
                    .add_function(name, function_type, Some(Linkage::External))
            }
        }
    }

    fn function_call_without_arguments(
        &self,
        builder: &Builder<'ctx>,
        name: &str,
    ) -> IntValue<'ctx> {
        let function = self.lookup_function_without_arguments(name);
        builder
            .build_call(function, &vec![], "func_call")
            .try_as_basic_value()
            .expect_left("function return value mismatch")
            .into_int_value()
    }
}

fn constant_function_module<'ctx>(
    thread_safe_context: &'ctx ThreadSafeContext,
    value: u64,
    name: &str,
) -> ThreadSafeModule<'ctx> {
    ModuleBuilder::new(thread_safe_context, name)
        .add_contstant_function(name, value)
        .build()
        .expect("invalid test module")
}

fn constant_function_object_file(context: &Context, value: u64, name: &str) -> MemoryBuffer {
    let module = context.create_module(name);
    let function_type = context.i64_type().fn_type(&vec![], false);
    let builder = context.create_builder();
    let function = module.add_function(name, function_type, None);
    let entry_bb = context.append_basic_block(function, "entry");
    builder.position_at_end(entry_bb);
    builder.build_return(Some(&context.i64_type().const_int(value, false)));
    let target_machine = get_native_target_machine();
    target_machine
        .write_to_memory_buffer(&module, FileType::Object)
        .expect("TargetMachine::write_to_memory_buffer failed")
}

fn get_native_target_machine() -> TargetMachine {
    let target_tripple = TargetMachine::get_default_triple();
    let target_cpu_features_llvm_string = TargetMachine::get_host_cpu_features();
    let target_cpu_features = target_cpu_features_llvm_string
        .to_str()
        .expect("TargetMachine::get_host_cpu_features returned invalid string");
    let target = Target::from_triple(&target_tripple).expect("Target::from_triple failed");
    target
        .create_target_machine(
            &target_tripple,
            "",
            target_cpu_features,
            OptimizationLevel::Default,
            RelocMode::Default,
            CodeModel::Default,
        )
        .expect("Target::create_target_machine failed")
}
