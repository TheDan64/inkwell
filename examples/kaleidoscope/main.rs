//! This is an example of the [Kaleidoscope tutorial](https://llvm.org/docs/tutorial/)
//! made in Rust, using Inkwell.
//! Currently, all features up to the
//! [7th chapter](https://llvm.org/docs/tutorial/MyFirstLanguageFrontend/LangImpl07.html)
//! are available.
//! This example is supposed to be ran as a executable, which launches a REPL.
//! The source code is in the following order:
//! - `implementation_typed_pointers.rs`:
//!     Lexer,
//!     Parser,
//!     Compiler.
//! - `main.rs`:
//!     Program.
//!
//! Both the `Parser` and the `Compiler` may fail, in which case they would return
//! an error represented by `Result<T, &'static str>`, for easier error reporting.

use std::collections::HashMap;
use std::io::{self, Write};

use inkwell::context::Context;
use inkwell::module::Module;
#[llvm_versions(..=15)]
use inkwell::passes::PassManager;
use inkwell::OptimizationLevel;
#[llvm_versions(16..)]
use inkwell::{
    passes::PassBuilderOptions,
    targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine},
};

use inkwell_internals::llvm_versions;

mod implementation_typed_pointers;
pub use implementation_typed_pointers::*;

// ======================================================================================
// PROGRAM ==============================================================================
// ======================================================================================

// macro used to print & flush without printing a new line
macro_rules! print_flush {
    ( $( $x:expr ),* ) => {
        print!( $($x, )* );

        std::io::stdout().flush().expect("Could not flush to standard output.");
    };
}

#[no_mangle]
pub extern "C" fn putchard(x: f64) -> f64 {
    print_flush!("{}", x as u8 as char);
    x
}

#[no_mangle]
pub extern "C" fn printd(x: f64) -> f64 {
    println!("{}", x);
    x
}

// Adding the functions above to a global array,
// so Rust compiler won't remove them.
#[used]
static EXTERNAL_FNS: [extern "C" fn(f64) -> f64; 2] = [putchard, printd];

#[llvm_versions(..=15)]
fn run_passes_on(module: &Module) {
    let fpm = PassManager::create(());

    fpm.add_instruction_combining_pass();
    fpm.add_reassociate_pass();
    fpm.add_gvn_pass();
    fpm.add_cfg_simplification_pass();
    fpm.add_basic_alias_analysis_pass();
    fpm.add_promote_memory_to_register_pass();

    fpm.run_on(module);
}

#[llvm_versions(16..)]
fn run_passes_on(module: &Module) {
    Target::initialize_all(&InitializationConfig::default());
    let target_triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&target_triple).unwrap();
    let target_machine = target
        .create_target_machine(
            &target_triple,
            "generic",
            "",
            OptimizationLevel::None,
            RelocMode::PIC,
            CodeModel::Default,
        )
        .unwrap();

    let passes: &[&str] = &[
        "instcombine",
        "reassociate",
        "gvn",
        "simplifycfg",
        // "basic-aa",
        "mem2reg",
    ];

    module
        .run_passes(passes.join(",").as_str(), &target_machine, PassBuilderOptions::create())
        .unwrap();
}

/// Entry point of the program; acts as a REPL.
pub fn main() {
    let mut display_lexer_output = false;
    let mut display_parser_output = false;
    let mut display_compiler_output = false;

    for arg in std::env::args() {
        match arg.as_str() {
            "--dl" => display_lexer_output = true,
            "--dp" => display_parser_output = true,
            "--dc" => display_compiler_output = true,
            _ => (),
        }
    }

    let context = Context::create();
    let builder = context.create_builder();

    let mut previous_exprs = Vec::new();

    loop {
        println!();
        print_flush!("?> ");

        // Read input from stdin
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Could not read from standard input.");

        if input.starts_with("exit") || input.starts_with("quit") {
            break;
        } else if input.chars().all(char::is_whitespace) {
            continue;
        }

        // Build precedence map
        let mut prec = HashMap::with_capacity(6);

        prec.insert('=', 2);
        prec.insert('<', 10);
        prec.insert('+', 20);
        prec.insert('-', 20);
        prec.insert('*', 40);
        prec.insert('/', 40);

        // Parse and (optionally) display input
        if display_lexer_output {
            println!(
                "-> Attempting to parse lexed input: \n{:?}\n",
                Lexer::new(input.as_str()).collect::<Vec<Token>>()
            );
        }

        // make module
        let module = context.create_module("tmp");

        // recompile every previously parsed function into the new module
        for prev in &previous_exprs {
            Compiler::compile(&context, &builder, &module, prev).expect("Cannot re-add previously compiled function.");
        }

        let (function, is_anonymous) = match Parser::new(input, &mut prec).parse() {
            Ok(fun) => {
                let is_anon = fun.is_anon;

                if display_parser_output {
                    if is_anon {
                        println!("-> Expression parsed: \n{:?}\n", fun.body);
                    } else {
                        println!("-> Function parsed: \n{:?}\n", fun);
                    }
                }

                match Compiler::compile(&context, &builder, &module, &fun) {
                    Ok(function) => {
                        if !is_anon {
                            // only add it now to ensure it is correct
                            previous_exprs.push(fun);
                        }

                        (function, is_anon)
                    },
                    Err(err) => {
                        println!("!> Error compiling function: {}", err);
                        continue;
                    },
                }
            },
            Err(err) => {
                println!("!> Error parsing expression: {}", err);
                continue;
            },
        };

        run_passes_on(&module);

        if display_compiler_output {
            println!("-> Expression compiled to IR:");
            function.print_to_stderr();
        }

        if is_anonymous {
            let ee = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();

            let fn_name = function.get_name().to_str().unwrap();
            let maybe_fn = unsafe { ee.get_function::<unsafe extern "C" fn() -> f64>(fn_name) };
            let compiled_fn = match maybe_fn {
                Ok(f) => f,
                Err(err) => {
                    println!("!> Error during execution: {:?}", err);
                    continue;
                },
            };

            unsafe {
                println!("=> {}", compiled_fn.call());
            }
        }
    }
}
