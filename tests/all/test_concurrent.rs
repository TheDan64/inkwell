use inkwell::context::Context;
use inkwell::concurrent::{CompilerPool, CompilerJob};
use inkwell::module::Module;

struct MyJob {
    id: usize,
}

impl CompilerJob for MyJob {
    fn execute<'ctx>(self: Box<Self>, local_ctx: &'ctx Context) -> Module<'ctx> {
        let module = local_ctx.create_module(&format!("local_{}", self.id));
        let i32_type = local_ctx.i32_type();
        let fn_type = i32_type.fn_type(&[], false);
        let fn_val = module.add_function(&format!("do_work_{}", self.id), fn_type, None);
        
        let builder = local_ctx.create_builder();
        let block = local_ctx.append_basic_block(fn_val, "entry");
        builder.position_at_end(block);
        
        let ret_val = i32_type.const_int(self.id as u64, false);
        builder.build_return(Some(&ret_val)).unwrap();
        
        module
    }
}

#[test]
fn test_concurrent_compiler_pool() {
    let master_context = Context::create();
    let master_module = master_context.create_module("master");

    let mut jobs: Vec<Box<dyn CompilerJob>> = Vec::new();

    // Spawn 3 identical independent generation tasks
    for i in 0..3 {
        jobs.push(Box::new(MyJob { id: i }));
    }

    assert!(CompilerPool::execute_and_link(&master_context, &master_module, jobs).is_ok());

    // Verify all 3 functions made it into the master module successfully
    assert!(master_module.get_function("do_work_0").is_some());
    assert!(master_module.get_function("do_work_1").is_some());
    assert!(master_module.get_function("do_work_2").is_some());
}
