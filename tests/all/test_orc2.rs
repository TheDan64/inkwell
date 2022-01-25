use std::ffi::CStr;

#[llvm_versions(12.0..=latest)]
use inkwell::orc2::{lljit::ObjectLinkingLayerCreator, ObjectLayer};
use inkwell::{
    builder::Builder,
    module::{Linkage, Module},
    orc2::{
        lljit::{LLJITBuilder, LLJIT},
        ExecutionSession, JITTargetMachineBuilder, ThreadSafeContext, ThreadSafeModule,
    },
    support::LLVMString,
    values::{FunctionValue, IntValue},
};

#[test]
fn test_drop_lljit_before_function_use() {
    let thread_safe_context = ThreadSafeContext::create();
    let module =
        constant_function_module(&thread_safe_context, 64, "main").expect("invalid test module");
    let lljit = LLJIT::create().expect("LLJIT::create failed");
    let main_jd = lljit.get_main_jit_dylib();
    lljit
        .add_module(&main_jd, module)
        .expect("LLJIT::add_module failed");
    drop(thread_safe_context);
    drop(main_jd);
    unsafe {
        let function = lljit
            .get_function::<unsafe extern "C" fn() -> u64>("main")
            .expect("LLJIT::get_function failed");
        drop(lljit);
        assert_eq!(function.call(), 64);
    }
}

#[llvm_versions(12.0..=latest)]
#[test]
fn test_lljit_add_module_with_rt_default() {
    let thread_safe_context = ThreadSafeContext::create();
    let module =
        constant_function_module(&thread_safe_context, 64, "main").expect("invalid test module");
    let lljit = LLJIT::create().expect("LLJIT::create failed");
    let main_jd = lljit.get_main_jit_dylib();
    let module_rt = main_jd.create_resource_tracker();
    lljit
        .add_module_with_rt(&module_rt, module)
        .expect("LLJIT::add_module_with_rt failed");
    drop(thread_safe_context);
    drop(main_jd);
    drop(module_rt);
    unsafe {
        let function = lljit
            .get_function::<unsafe extern "C" fn() -> u64>("main")
            .expect("LLJIT::get_function failed");
        drop(lljit);
        assert_eq!(function.call(), 64);
    }
}

#[test]
fn test_multiple_lljit_instances_single_context() {
    let thread_safe_context = ThreadSafeContext::create();
    let module_1 =
        constant_function_module(&thread_safe_context, 64, "main_1").expect("invalid test module");
    let module_2 =
        constant_function_module(&thread_safe_context, 42, "main_2").expect("invalid test module");
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
        drop(lljit_1);
        drop(lljit_2);
        assert_eq!(function_1.call(), 64);
        assert_eq!(function_2.call(), 42);
    }
}

#[test]
fn test_lljit_multiple_contexts() {
    let thread_safe_context_1 = ThreadSafeContext::create();
    let module_1 = constant_function_module(&thread_safe_context_1, 64, "main_1")
        .expect("invalid test module");
    let lljit = LLJIT::create().expect("LLJIT::create failed");
    let main_jd = lljit.get_main_jit_dylib();
    lljit
        .add_module(&main_jd, module_1)
        .expect("LLJIT::add_module failed");
    let thread_safe_context_2 = ThreadSafeContext::create();
    let module_2 = constant_function_module(&thread_safe_context_2, 42, "main_2")
        .expect("invalid test module");
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
        drop(lljit);
        assert_eq!(function_1.call(), 64);
        assert_eq!(function_2.call(), 42);
    }
}

#[llvm_versions(12.0..=latest)]
#[test]
fn test_lljit_remove_resource_tracker() {
    let thread_safe_context = ThreadSafeContext::create();
    let module =
        constant_function_module(&thread_safe_context, 64, "main").expect("invalid test module");
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
    let module =
        constant_function_module(&thread_safe_context, 42, "main").expect("invalid test module");
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
    LLJITBuilder::create()
        .set_jit_target_machine_builder(jit_target_machine_builder)
        .build()
        .expect_err("LLJITBuilder::build succeeded");
}

#[llvm_versions(12.0..=latest)]
#[test]
fn test_lljit_builder_set_object_linking_layer_creator() {
    let object_linking_layer_creator: Box<dyn ObjectLinkingLayerCreator> =
        Box::new(SimpleObjectLinkingLayerCreator {});
    let lljit =
        LLJITBuilder::create().set_object_linking_layer_creator(object_linking_layer_creator);
    let lljit = lljit.build().expect("LLJITBuilder::build failed");
    test_basic_lljit_functionality(lljit);
}

#[llvm_versions(12.0..=latest)]
#[derive(Debug)]
struct SimpleObjectLinkingLayerCreator {}

#[llvm_versions(12.0..=latest)]
impl ObjectLinkingLayerCreator for SimpleObjectLinkingLayerCreator {
    fn create_object_linking_layer(
        &self,
        execution_session: ExecutionSession,
        _triple: &CStr,
    ) -> ObjectLayer {
        execution_session
            .create_rt_dyld_object_linking_layer_with_section_memory_manager()
            .into()
    }
}

#[llvm_versions(12.0..=latest)]
#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
#[test]
fn test_jit_target_machin_builder_create_from_target_machine() {
    use inkwell::{
        targets::{CodeModel, RelocMode, Target, TargetTriple},
        OptimizationLevel,
    };

    let target = Target::from_name("x86-64").expect("Target::from_name failed");
    let target_machine = target
        .create_target_machine(
            &TargetTriple::create("x86_64-unknown-linux-gnu"),
            "x86-64",
            "",
            OptimizationLevel::Default,
            RelocMode::Default,
            CodeModel::Default,
        )
        .expect("Target::create_target_machine failed");
    let jit_target_machine_builder =
        JITTargetMachineBuilder::create_from_target_machine(target_machine);
    let lljit = LLJITBuilder::create()
        .set_jit_target_machine_builder(jit_target_machine_builder)
        .build()
        .expect("LLJITBuilder::build failed");
    test_basic_lljit_functionality(lljit);
}

#[llvm_versions(13.0..=latest)]
#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
#[test]
fn test_jit_target_machin_builder_set_target_triple() {
    let jit_target_machine_builder = JITTargetMachineBuilder::detect_host()
        .expect("JITTargetMachineBuilder::detect_host failed");
    jit_target_machine_builder.set_target_triple("x86_64-unknown-linux-gnu");
    let lljit = LLJITBuilder::create()
        .set_jit_target_machine_builder(jit_target_machine_builder)
        .build()
        .expect("LLJITBuilder::build failed");
    test_basic_lljit_functionality(lljit);
}

fn test_basic_lljit_functionality(lljit: LLJIT) {
    let thread_safe_context = ThreadSafeContext::create();
    let module =
        constant_function_module(&thread_safe_context, 64, "main").expect("invalid test module");
    let main_jd = lljit.get_main_jit_dylib();
    lljit
        .add_module(&main_jd, module)
        .expect("LLJIT::add_module failed");
    drop(thread_safe_context);
    drop(main_jd);
    unsafe {
        let function = lljit
            .get_function::<unsafe extern "C" fn() -> u64>("main")
            .expect("LLJIT::get_function failed");
        drop(lljit);
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
) -> Result<ThreadSafeModule<'ctx>, LLVMString> {
    ModuleBuilder::new(thread_safe_context, name)
        .add_contstant_function(name, value)
        .build()
}
