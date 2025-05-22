use std::cell::RefCell;
use std::rc::Rc;

use inkwell::context::Context;
use inkwell::execution_engine::FunctionLookupError;
use inkwell::memory_manager::McjitMemoryManager;
use inkwell::module::Linkage;
use inkwell::targets::{CodeModel, InitializationConfig, Target};
use inkwell::{AddressSpace, IntPredicate, OptimizationLevel};

type Thunk = unsafe extern "C" fn();

#[test]
fn test_get_function_address() {
    let context = Context::create();
    // let module = context.create_module("errors_abound");
    let builder = context.create_builder();
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);

    // FIXME: LLVM's global state is leaking, causing this to fail in `cargo test` but not `cargo test test_get_function_address`
    // nor (most of the time) with `cargo test -- --test-threads LARGE_NUM`
    // assert_eq!(module.create_jit_execution_engine(OptimizationLevel::None), Err("Unable to find target for this triple (no targets are registered)".into()));
    let module = context.create_module("errors_abound");

    Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");

    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();

    unsafe {
        assert_eq!(
            execution_engine.get_function::<Thunk>("errors").unwrap_err(),
            FunctionLookupError::FunctionNotFound
        );
    }

    let module = context.create_module("errors_abound");
    let fn_value = module.add_function("func", fn_type, None);
    let basic_block = context.append_basic_block(fn_value, "entry");

    builder.position_at_end(basic_block);
    builder.build_return(None).unwrap();

    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();

    unsafe {
        assert_eq!(
            execution_engine.get_function::<Thunk>("errors").unwrap_err(),
            FunctionLookupError::FunctionNotFound
        );

        assert!(execution_engine.get_function::<Thunk>("func").is_ok());
    }
}

#[test]
fn test_jit_execution_engine() {
    let context = Context::create();
    let module = context.create_module("main_module");
    let builder = context.create_builder();
    let i32_type = context.i32_type();
    #[cfg(feature = "typed-pointers")]
    let i8_ptr_ptr_type = context
        .i8_type()
        .ptr_type(AddressSpace::default())
        .ptr_type(AddressSpace::default());
    #[cfg(not(feature = "typed-pointers"))]
    let i8_ptr_ptr_type = context.ptr_type(AddressSpace::default());
    let one_i32 = i32_type.const_int(1, false);
    let three_i32 = i32_type.const_int(3, false);
    let fourtytwo_i32 = i32_type.const_int(42, false);
    let fn_type = i32_type.fn_type(&[i32_type.into(), i8_ptr_ptr_type.into()], false);
    let fn_value = module.add_function("main", fn_type, None);
    let main_argc = fn_value.get_first_param().unwrap().into_int_value();
    // let main_argv = fn_value.get_nth_param(1).unwrap();
    let check_argc = context.append_basic_block(fn_value, "check_argc");
    let check_arg3 = context.append_basic_block(fn_value, "check_arg3");
    let error1 = context.append_basic_block(fn_value, "error1");
    let success = context.append_basic_block(fn_value, "success");

    main_argc.set_name("argc");

    // If anything goes wrong, jump to returning 1
    builder.position_at_end(error1);
    builder.build_return(Some(&one_i32)).unwrap();

    // If successful, jump to returning 42
    builder.position_at_end(success);
    builder.build_return(Some(&fourtytwo_i32)).unwrap();

    // See if argc == 3
    builder.position_at_end(check_argc);

    let eq = IntPredicate::EQ;
    let argc_check = builder.build_int_compare(eq, main_argc, three_i32, "argc_cmp").unwrap();

    builder
        .build_conditional_branch(argc_check, check_arg3, error1)
        .unwrap();

    builder.position_at_end(check_arg3);
    builder.build_unconditional_branch(success).unwrap();

    Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");

    let execution_engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .expect("Could not create Execution Engine");

    let main = execution_engine
        .get_function_value("main")
        .expect("Could not find main in ExecutionEngine");

    let ret = unsafe { execution_engine.run_function_as_main(main, &["input", "bar"]) };

    assert_eq!(ret, 1, "unexpected main return code: {}", ret);

    let ret = unsafe { execution_engine.run_function_as_main(main, &["input", "bar", "baz"]) };

    assert_eq!(ret, 42, "unexpected main return code: {}", ret);
}

// #[test]
// fn test_execution_engine_empty_module() {
//     let context = Context::create();
//     let module = context.create_module("fooo");
//     let builder = context.create_builder();

//     let ee = module.create_jit_execution_engine(OptimizationLevel::None); // Segfault?
// }

#[test]
fn test_execution_engine() {
    let context = Context::create();
    let module = context.create_module("main_module");

    assert!(module.create_execution_engine().is_ok());
}

#[test]
fn test_interpreter_execution_engine() {
    let context = Context::create();
    let module = context.create_module("main_module");

    assert!(module.create_interpreter_execution_engine().is_ok());
}

#[test]

fn test_mcjit_execution_engine_with_memory_manager() {
    let mmgr = MockMemoryManager::new();
    let mmgr_for_test = mmgr.clone();

    {
        let context = Context::create();
        let module = context.create_module("main_module");
        let builder = context.create_builder();

        // Define @llvm.experimental.stackmap
        let fn_type = context
            .void_type()
            .fn_type(&[context.i64_type().into(), context.i32_type().into()], true);
        let stackmap_func = module.add_function("llvm.experimental.stackmap", fn_type, Some(Linkage::External));

        // Set up the function
        //
        // `@llvm.experimental.stackmap` a call is present, LLVM will emit a separate stackmap section,
        // causing the `allocate_data_section` callback to be invoked an additional
        // time to handle the stackmap data. Specifically:
        // ```
        // f64 test_fn() {
        // entry:
        //   call void @llvm.experimental.stackmap(i64 12345, i32 0)
        //   ret f64 64.0
        // }
        // ```
        let double = context.f64_type();
        let sig = double.fn_type(&[], false);
        let f = module.add_function("test_fn", sig, None);
        let b = context.append_basic_block(f, "entry");
        builder.position_at_end(b);

        // Create a call to the stackmap intrinsic
        // Stack maps are used by the garbage collector to find roots on the stack
        // See: https://llvm.org/docs/StackMaps.html#intrinsics
        builder
            .build_call(
                stackmap_func,
                &[
                    context.i64_type().const_int(12345, false).into(),
                    context.i32_type().const_int(0, false).into(),
                ],
                "call_stackmap",
            )
            .unwrap();

        // Insert a return statement
        let ret = double.const_float(64.0);
        builder.build_return(Some(&ret)).unwrap();

        module.verify().unwrap();

        let ee = module
            .create_mcjit_execution_engine_with_memory_manager(
                mmgr,
                OptimizationLevel::None,
                CodeModel::Default,
                false,
                false,
            )
            .unwrap();

        unsafe {
            let test_fn = ee.get_function::<unsafe extern "C" fn() -> f64>("test_fn").unwrap();
            let return_value = test_fn.call();
            assert_eq!(return_value, 64.0);
        }
    }
    // ee dropped here. Destroy should be called

    let data = mmgr_for_test.data.borrow();
    assert_eq!(1, data.code_alloc_calls);
    assert!(
        data.data_alloc_calls >= 2,
        "Expected at least 2 calls to allocate_data_section, but got {}. \
         We've observed that LLVM 5 typically calls it 3 times, while LLVM 18 often calls it only 2.",
        data.data_alloc_calls
    );
    assert_eq!(1, data.finalize_calls);
    assert_eq!(1, data.destroy_calls);
}

#[test]
fn test_create_mcjit_engine_when_already_owned() {
    let context = Context::create();
    let module = context.create_module("owned_module");

    // First engine should succeed
    let memory_manager = MockMemoryManager::new();
    let engine_result = module.create_mcjit_execution_engine_with_memory_manager(
        memory_manager,
        OptimizationLevel::None,
        CodeModel::Default,
        false,
        false,
    );
    assert!(engine_result.is_ok());

    // Second engine should fail
    let memory_manager2 = MockMemoryManager::new();
    let second_result = module.create_mcjit_execution_engine_with_memory_manager(
        memory_manager2,
        OptimizationLevel::None,
        CodeModel::Default,
        false,
        false,
    );
    assert!(
        second_result.is_err(),
        "Expected an error when creating a second ExecutionEngine on the same module"
    );
}

#[test]
fn test_add_remove_module() {
    let context = Context::create();
    let module = context.create_module("test");
    let ee = module
        .create_jit_execution_engine(OptimizationLevel::default())
        .unwrap();

    assert!(ee.add_module(&module).is_err());

    let module2 = context.create_module("mod2");

    assert!(ee.remove_module(&module2).is_err());
    assert!(ee.add_module(&module2).is_ok());
    assert!(ee.remove_module(&module).is_ok());
    assert!(ee.remove_module(&module2).is_ok());
}

// REVIEW: Global state pollution access tests cause this to pass when run individually
// but fail when multiple tests are run
// #[test]
// fn test_no_longer_segfaults() {
//     Target::initialize_amd_gpu(&InitializationConfig::default());

//     let context = Context::create();
//     let module = context.create_module("test");

//     assert_eq!(*module.create_jit_execution_engine(OptimizationLevel::default()).unwrap_err(), *CString::new("No available targets are compatible with this triple.").unwrap());

//     // REVIEW: Module is being cloned in err case... should test Module still works as expected...
// }

// #[test]
// fn test_get_function_value() {
//     let context = Context::create();
//     let builder = context.create_builder();
//     let module = context.create_module("errors_abound");
//     // let mut execution_engine = ExecutionEngine::create_jit_from_module(module, 0);
//     let mut execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
//     let void_type = context.void_type();
//     let fn_type = void_type.fn_type(&[], false);
//     let fn_value = module.add_function("func", fn_type, None);
//     let basic_block = context.append_basic_block(&fn_value, "entry");

//     builder.position_at_end(basic_block);
//     builder.build_return(None);

//     assert_eq!(execution_engine.get_function_value("errors"), Err(FunctionLookupError::JITNotEnabled));

//     // Regain ownership of module
//     let module = execution_engine.remove_module(&module).unwrap();

//     Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");

//     let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();

//     assert_eq!(execution_engine.get_function_value("errors"), Err(FunctionLookupError::FunctionNotFound));

//     assert!(execution_engine.get_function_value("func").is_ok());
// }

// No longer necessary with explicit lifetimes
// #[test]
// fn test_previous_double_free() {
//     Target::initialize_native(&InitializationConfig::default()).unwrap();

//     let context = Context::create();
//     let module = context.create_module("sum");
//     let _ee = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();

//     drop(context);
//     drop(module);
// }

// #[test]
// fn test_previous_double_free2() {
//     Target::initialize_native(&InitializationConfig::default()).unwrap();

//     let _execution_engine = {
//         let context = Context::create();
//         let module = context.create_module("sum");

//         module.create_jit_execution_engine(OptimizationLevel::None).unwrap()
//     };
// }

/// A mock memory manager that allocates memory in fixed-size pages for testing.
#[derive(Debug, Clone)]
struct MockMemoryManager {
    data: Rc<RefCell<MockMemoryManagerData>>,
}

#[derive(Debug)]
struct MockMemoryManagerData {
    fixed_capacity_bytes: usize,
    fixed_page_size: usize,

    code_buff_ptr: std::ptr::NonNull<u8>,
    code_offset: usize,

    data_buff_ptr: std::ptr::NonNull<u8>,
    data_offset: usize,

    /// Count call to callbacks for testing
    code_alloc_calls: usize,
    data_alloc_calls: usize,
    finalize_calls: usize,
    destroy_calls: usize,
}

impl MockMemoryManager {
    pub fn new() -> Self {
        let capacity_bytes = 128 * 1024;
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize };

        let code_buff_ptr = unsafe {
            std::ptr::NonNull::new_unchecked(libc::mmap(
                std::ptr::null_mut(),
                capacity_bytes,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            ) as *mut u8)
        };

        let data_buff_ptr = unsafe {
            std::ptr::NonNull::new_unchecked(libc::mmap(
                std::ptr::null_mut(),
                capacity_bytes,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            ) as *mut u8)
        };

        Self {
            data: Rc::new(RefCell::new(MockMemoryManagerData {
                fixed_capacity_bytes: capacity_bytes,
                fixed_page_size: page_size,

                code_buff_ptr,
                code_offset: 0,

                data_buff_ptr,
                data_offset: 0,

                code_alloc_calls: 0,
                data_alloc_calls: 0,
                finalize_calls: 0,
                destroy_calls: 0,
            })),
        }
    }
}

impl McjitMemoryManager for MockMemoryManager {
    fn allocate_code_section(
        &mut self,
        size: libc::size_t,
        _alignment: libc::c_uint,
        _section_id: libc::c_uint,
        _section_name: &str,
    ) -> *mut u8 {
        let mut data = self.data.borrow_mut();
        data.code_alloc_calls += 1;

        let alloc_size = size.div_ceil(data.fixed_page_size) * data.fixed_page_size;
        let ptr = unsafe { data.code_buff_ptr.as_ptr().add(data.code_offset) };
        data.code_offset += alloc_size;

        ptr
    }

    fn allocate_data_section(
        &mut self,
        size: libc::size_t,
        _alignment: libc::c_uint,
        _section_id: libc::c_uint,
        _section_name: &str,
        _is_read_only: bool,
    ) -> *mut u8 {
        let mut data = self.data.borrow_mut();

        data.data_alloc_calls += 1;

        let alloc_size = size.div_ceil(data.fixed_page_size) * data.fixed_page_size;
        let ptr = unsafe { data.data_buff_ptr.as_ptr().add(data.data_offset) };
        data.data_offset += alloc_size;

        ptr
    }

    fn finalize_memory(&mut self) -> Result<(), String> {
        let mut data = self.data.borrow_mut();

        data.finalize_calls += 1;

        unsafe {
            libc::mprotect(
                data.code_buff_ptr.as_ptr() as *mut libc::c_void,
                data.fixed_capacity_bytes,
                libc::PROT_READ | libc::PROT_EXEC,
            );
            libc::mprotect(
                data.data_buff_ptr.as_ptr() as *mut libc::c_void,
                data.fixed_capacity_bytes,
                libc::PROT_READ | libc::PROT_WRITE,
            );
        }

        Ok(())
    }

    fn destroy(&mut self) {
        let mut data = self.data.borrow_mut();

        data.destroy_calls += 1;

        unsafe {
            libc::munmap(
                data.code_buff_ptr.as_ptr() as *mut libc::c_void,
                data.fixed_capacity_bytes,
            );
            libc::munmap(
                data.data_buff_ptr.as_ptr() as *mut libc::c_void,
                data.fixed_capacity_bytes,
            );
        }
    }
}
