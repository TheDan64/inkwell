use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{EngineModule, ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::support::LLVMString;

use std::error::Error;

/// Convenience type alias for the `sum` function.
///
/// Calling this is innately `unsafe` because there's no guarantee it doesn't
/// do `unsafe` operations internally.
type SumFunc = unsafe extern "C" fn(u64, u64, u64) -> u64;

struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    fn compile_sum(&self) -> Option<()> {
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[i64_type.into(), i64_type.into(), i64_type.into()], false);
        let function = self.module.add_function("sum", fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(basic_block);

        let x = function.get_nth_param(0)?.into_int_value();
        let y = function.get_nth_param(1)?.into_int_value();
        let z = function.get_nth_param(2)?.into_int_value();

        let sum = self.builder.build_int_add(x, y, "sum").unwrap();
        let sum = self.builder.build_int_add(sum, z, "sum").unwrap();

        self.builder.build_return(Some(&sum)).unwrap();

        Some(())
    }

    /// Hands the finished module to a JIT engine, which owns it from now on.
    fn into_execution_engine(self) -> Result<(ExecutionEngine<'ctx>, EngineModule<'ctx>), LLVMString> {
        self.module.create_jit_execution_engine(OptimizationLevel::None)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let context = Context::create();
    let codegen = CodeGen {
        context: &context,
        module: context.create_module("sum"),
        builder: context.create_builder(),
    };

    codegen.compile_sum().ok_or("Unable to build `sum`")?;

    let (execution_engine, _module) = codegen.into_execution_engine()?;
    let sum: JitFunction<SumFunc> = unsafe { execution_engine.get_function("sum")? };

    let x = 1u64;
    let y = 2u64;
    let z = 3u64;

    unsafe {
        println!("{} + {} + {} = {}", x, y, z, sum.call(x, y, z));
        assert_eq!(sum.call(x, y, z), x + y + z);
    }

    Ok(())
}
