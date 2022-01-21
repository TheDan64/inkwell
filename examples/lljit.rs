use inkwell::orc2::{Function, LLVMError, ThreadSafeContext, ThreadSafeModule, LLJIT};

fn main() {
    if let Err(error) = run() {
        println!("{:#?}", error);
        println!("{:?}", error.get_type_id());
        println!("{:?}", error.get_message());
    }
}

fn run() -> Result<(), LLVMError> {
    let thread_safe_context = ThreadSafeContext::create();
    let foo_module = constant_function_module(&thread_safe_context, 42, "foo");
    let jit = LLJIT::create(None)?;
    let main_dylib = jit.get_main_jit_dylib();
    let foo_module_rt = main_dylib.create_resource_tracker();
    jit.add_module_with_rt(&foo_module_rt, &foo_module)?;
    let foo_function: Function<'_, unsafe extern "C" fn() -> u64> = jit.get_function("foo")?;
    drop(jit);
    drop(foo_module_rt);
    drop(main_dylib);
    drop(foo_module);
    drop(thread_safe_context);
    unsafe {
        println!("foo(): {}", foo_function.call());
    }
    Ok(())
}

fn constant_function_module<'ctx>(
    thread_safe_context: &'ctx ThreadSafeContext,
    number: u64,
    name: &str,
) -> ThreadSafeModule<'ctx> {
    let context = thread_safe_context.context();
    let module = context.create_module(name);
    let function_type = context.i64_type().fn_type(&vec![], false);
    let builder = context.create_builder();
    let function = module.add_function(name, function_type, None);
    let entry_bb = context.append_basic_block(function, "entry");
    builder.position_at_end(entry_bb);
    builder.build_return(Some(&context.i64_type().const_int(number, false)));
    module.print_to_stderr();
    thread_safe_context.create_module(module)
}
