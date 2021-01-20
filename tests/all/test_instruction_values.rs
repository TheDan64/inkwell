use inkwell::context::Context;
use inkwell::values::{BasicValue, InstructionOpcode::*};
use inkwell::{AddressSpace, AtomicOrdering, AtomicRMWBinOp, FloatPredicate, IntPredicate};

#[test]
fn test_operands() {
    let context = Context::create();
    let module = context.create_module("ivs");
    let builder = context.create_builder();
    let void_type = context.void_type();
    let f32_type = context.f32_type();
    let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    let fn_type = void_type.fn_type(&[f32_ptr_type.into()], false);

    let function = module.add_function("take_f32_ptr", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");

    builder.position_at_end(basic_block);

    let arg1 = function.get_first_param().unwrap().into_pointer_value();
    let f32_val = f32_type.const_float(::std::f64::consts::PI);
    let store_instruction = builder.build_store(arg1, f32_val);
    let free_instruction = builder.build_free(arg1);
    let return_instruction = builder.build_return(None);

    assert_eq!(store_instruction.get_opcode(), Store);
    assert_eq!(free_instruction.get_opcode(), Call);
    assert_eq!(return_instruction.get_opcode(), Return);

    assert!(arg1.as_instruction_value().is_none());

    // Test operands
    assert_eq!(store_instruction.get_num_operands(), 2);
    assert_eq!(free_instruction.get_num_operands(), 2);

    let store_operand0 = store_instruction.get_operand(0).unwrap();
    let store_operand1 = store_instruction.get_operand(1).unwrap();

    assert_eq!(store_operand0.left().unwrap(), f32_val); // f32 const
    assert_eq!(store_operand1.left().unwrap(), arg1); // f32* arg1
    assert!(store_instruction.get_operand(2).is_none());
    assert!(store_instruction.get_operand(3).is_none());
    assert!(store_instruction.get_operand(4).is_none());

    let free_operand0 = free_instruction.get_operand(0).unwrap().left().unwrap();
    let free_operand1 = free_instruction.get_operand(1).unwrap().left().unwrap();
    let free_operand0_instruction = free_operand0.as_instruction_value().unwrap();

    assert!(free_operand0.is_pointer_value()); // (implictly casted) i8* arg1
    assert!(free_operand1.is_pointer_value()); // Free function ptr
    assert_eq!(free_operand0_instruction.get_opcode(), BitCast);
    assert_eq!(free_operand0_instruction.get_operand(0).unwrap().left().unwrap(), arg1);
    assert!(free_operand0_instruction.get_operand(1).is_none());
    assert!(free_operand0_instruction.get_operand(2).is_none());
    assert!(free_instruction.get_operand(2).is_none());
    assert!(free_instruction.get_operand(3).is_none());
    assert!(free_instruction.get_operand(4).is_none());

    assert!(module.verify().is_ok());

    assert!(free_instruction.set_operand(0, arg1));

    // Module is no longer valid because free takes an i8* not f32*
    assert!(module.verify().is_err());

    assert!(free_instruction.set_operand(0, free_operand0));

    assert!(module.verify().is_ok());

    // No-op, free only has two (0-1) operands
    assert!(!free_instruction.set_operand(2, free_operand0));

    assert!(module.verify().is_ok());

    assert_eq!(return_instruction.get_num_operands(), 0);
    assert!(return_instruction.get_operand(0).is_none());
    assert!(return_instruction.get_operand(1).is_none());
    assert!(return_instruction.get_operand(2).is_none());

    // Test Uses
    let bitcast_use_value = free_operand0_instruction
        .get_first_use()
        .unwrap()
        .get_used_value()
        .left()
        .unwrap();
    let free_call_param = free_instruction.get_operand(0).unwrap().left().unwrap();

    assert_eq!(bitcast_use_value, free_call_param);

    // These instructions/calls don't return any ir value so they aren't used anywhere
    assert!(store_instruction.get_first_use().is_none());
    assert!(free_instruction.get_first_use().is_none());
    assert!(return_instruction.get_first_use().is_none());

    // arg1 (%0) has two uses:
    //   store float 0x400921FB60000000, float* %0
    //   %1 = bitcast float* %0 to i8*
    let arg1_first_use = arg1.get_first_use().unwrap();
    let arg1_second_use = arg1_first_use.get_next_use().unwrap();

    // However their operands are used
    let store_operand_use0 = store_instruction.get_operand_use(0).unwrap();
    let store_operand_use1 = store_instruction.get_operand_use(1).unwrap();

    assert!(store_operand_use0.get_next_use().is_none());
    assert!(store_operand_use1.get_next_use().is_none());
    assert_eq!(store_operand_use1, arg1_second_use);

    assert_eq!(store_operand_use0.get_user().into_instruction_value(), store_instruction);
    assert_eq!(store_operand_use1.get_user().into_instruction_value(), store_instruction);
    assert_eq!(store_operand_use0.get_used_value().left().unwrap(), f32_val);
    assert_eq!(store_operand_use1.get_used_value().left().unwrap(), arg1);

    assert!(store_instruction.get_operand_use(2).is_none());
    assert!(store_instruction.get_operand_use(3).is_none());
    assert!(store_instruction.get_operand_use(4).is_none());
    assert!(store_instruction.get_operand_use(5).is_none());
    assert!(store_instruction.get_operand_use(6).is_none());

    let free_operand_use0 = free_instruction.get_operand_use(0).unwrap();
    let free_operand_use1 = free_instruction.get_operand_use(1).unwrap();

    assert!(free_operand_use0.get_next_use().is_none());
    assert!(free_operand_use1.get_next_use().is_none());
    assert!(free_instruction.get_operand_use(2).is_none());
    assert!(free_instruction.get_operand_use(3).is_none());
    assert!(free_instruction.get_operand_use(4).is_none());
    assert!(free_instruction.get_operand_use(5).is_none());
    assert!(free_instruction.get_operand_use(6).is_none());

    assert!(module.verify().is_ok());
}

#[test]
fn test_basic_block_operand() {
    let context = Context::create();
    let module = context.create_module("ivs");
    let builder = context.create_builder();
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let function = module.add_function("bb_op", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");
    let basic_block2 = context.append_basic_block(function, "exit");

    builder.position_at_end(basic_block);

    let branch_instruction = builder.build_unconditional_branch(basic_block2);
    let bb_operand = branch_instruction.get_operand(0).unwrap().right().unwrap();

    assert_eq!(bb_operand, basic_block2);

    let bb_operand_use = branch_instruction.get_operand_use(0).unwrap();

    assert_eq!(bb_operand_use.get_used_value().right().unwrap(), basic_block2);

    builder.position_at_end(basic_block2);
    builder.build_return(None);

    assert!(module.verify().is_ok());
}

#[test]
fn test_get_next_use() {
    let context = Context::create();
    let module = context.create_module("ivs");
    let builder = context.create_builder();
    let f32_type = context.f32_type();
    let fn_type = f32_type.fn_type(&[f32_type.into()], false);
    let function = module.add_function("take_f32", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");

    builder.position_at_end(basic_block);

    let arg1 = function.get_first_param().unwrap().into_float_value();
    let f32_val = f32_type.const_float(::std::f64::consts::PI);
    let add_pi0 = builder.build_float_add(arg1, f32_val, "add_pi");
    let add_pi1 = builder.build_float_add(add_pi0, f32_val, "add_pi");

    builder.build_return(Some(&add_pi1));

    // f32_val constant appears twice, so there are two uses (first, next)
    let first_use = f32_val.get_first_use().unwrap();

    assert_eq!(first_use.get_user(), add_pi1.as_instruction_value().unwrap());
    assert_eq!(first_use.get_next_use().map(|x| x.get_user().into_float_value()), Some(add_pi0));
    assert!(arg1.get_first_use().is_some());
    assert!(module.verify().is_ok());
}


#[test]
fn test_instructions() {
    let context = Context::create();
    let module = context.create_module("testing");
    let builder = context.create_builder();

    let void_type = context.void_type();
    let i64_type = context.i64_type();
    let f32_type = context.f32_type();
    let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    let fn_type = void_type.fn_type(&[f32_ptr_type.into(), f32_type.into()], false);

    let function = module.add_function("free_f32", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");

    builder.position_at_end(basic_block);

    let arg1 = function.get_first_param().unwrap().into_pointer_value();
    let arg2 = function.get_nth_param(1).unwrap().into_float_value();

    assert!(arg1.get_first_use().is_none());
    assert!(arg2.get_first_use().is_none());

    let f32_val = f32_type.const_float(::std::f64::consts::PI);

    let store_instruction = builder.build_store(arg1, f32_val);
    let ptr_val = builder.build_ptr_to_int(arg1, i64_type, "ptr_val");
    let ptr = builder.build_int_to_ptr(ptr_val, f32_ptr_type, "ptr");
    let icmp = builder.build_int_compare(IntPredicate::EQ, ptr_val, ptr_val, "icmp");
    let f32_sum = builder.build_float_add(arg2, f32_val, "f32_sum");
    let fcmp = builder.build_float_compare(FloatPredicate::OEQ, f32_sum, arg2, "fcmp");
    let free_instruction = builder.build_free(arg1);
    let return_instruction = builder.build_return(None);

    assert_eq!(store_instruction.get_opcode(), Store);
    assert_eq!(ptr_val.as_instruction().unwrap().get_opcode(), PtrToInt);
    assert_eq!(ptr.as_instruction().unwrap().get_opcode(), IntToPtr);
    assert_eq!(icmp.as_instruction().unwrap().get_opcode(), ICmp);
    assert_eq!(ptr.as_instruction().unwrap().get_icmp_predicate(), None);
    assert_eq!(icmp.as_instruction().unwrap().get_icmp_predicate().unwrap(), IntPredicate::EQ);
    assert_eq!(f32_sum.as_instruction().unwrap().get_opcode(), FAdd);
    assert_eq!(fcmp.as_instruction().unwrap().get_opcode(), FCmp);
    assert_eq!(f32_sum.as_instruction().unwrap().get_fcmp_predicate(), None);
    assert_eq!(icmp.as_instruction().unwrap().get_fcmp_predicate(), None);
    assert_eq!(fcmp.as_instruction().unwrap().get_fcmp_predicate().unwrap(), FloatPredicate::OEQ);
    assert_eq!(free_instruction.get_opcode(), Call);
    assert_eq!(return_instruction.get_opcode(), Return);

    // test instruction cloning
    let instruction_clone = return_instruction.clone();

    assert_eq!(instruction_clone.get_opcode(), return_instruction.get_opcode());
    assert_ne!(instruction_clone, return_instruction);

    // test copying
    let instruction_clone_copy = instruction_clone;

    assert_eq!(instruction_clone, instruction_clone_copy);
}

#[llvm_versions(10.0..=latest)]
#[test]
fn test_volatile_atomicrmw_cmpxchg() {
    let context = Context::create();
    let module = context.create_module("testing");
    let builder = context.create_builder();

    let void_type = context.void_type();
    let i32_type = context.i32_type();
    let i32_ptr_type = i32_type.ptr_type(AddressSpace::Generic);
    let fn_type = void_type.fn_type(&[i32_ptr_type.into(), i32_type.into()], false);

    let function = module.add_function("mem_inst", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");

    builder.position_at_end(basic_block);

    let arg1 = function.get_first_param().unwrap().into_pointer_value();
    let arg2 = function.get_nth_param(1).unwrap().into_int_value();

    assert!(arg1.get_first_use().is_none());
    assert!(arg2.get_first_use().is_none());

    let i32_val = i32_type.const_int(7, false);

    let atomicrmw = builder
        .build_atomicrmw(AtomicRMWBinOp::Add, arg1, arg2, AtomicOrdering::Unordered)
        .unwrap()
        .as_instruction_value()
        .unwrap();
    let cmpxchg = builder
        .build_cmpxchg(
            arg1,
            arg2,
            i32_val,
            AtomicOrdering::Monotonic,
            AtomicOrdering::Monotonic,
        )
        .unwrap()
        .as_instruction_value()
        .unwrap();

    assert_eq!(atomicrmw.get_volatile().unwrap(), false);
    assert_eq!(cmpxchg.get_volatile().unwrap(), false);
    atomicrmw.set_volatile(true).unwrap();
    cmpxchg.set_volatile(true).unwrap();
    assert_eq!(atomicrmw.get_volatile().unwrap(), true);
    assert_eq!(cmpxchg.get_volatile().unwrap(), true);
    atomicrmw.set_volatile(false).unwrap();
    cmpxchg.set_volatile(false).unwrap();
    assert_eq!(atomicrmw.get_volatile().unwrap(), false);
    assert_eq!(cmpxchg.get_volatile().unwrap(), false);
}

#[llvm_versions(3.6..=10.0)]
#[test]
fn test_mem_instructions() {
    let context = Context::create();
    let module = context.create_module("testing");
    let builder = context.create_builder();

    let void_type = context.void_type();
    let f32_type = context.f32_type();
    let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    let fn_type = void_type.fn_type(&[f32_ptr_type.into(), f32_type.into()], false);

    let function = module.add_function("mem_inst", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");

    builder.position_at_end(basic_block);

    let arg1 = function.get_first_param().unwrap().into_pointer_value();
    let arg2 = function.get_nth_param(1).unwrap().into_float_value();

    assert!(arg1.get_first_use().is_none());
    assert!(arg2.get_first_use().is_none());

    let f32_val = f32_type.const_float(::std::f64::consts::PI);

    let store_instruction = builder.build_store(arg1, f32_val);
    let load = builder.build_load(arg1, "");
    let load_instruction = load.as_instruction_value().unwrap();

    assert_eq!(store_instruction.get_volatile().unwrap(), false);
    assert_eq!(load_instruction.get_volatile().unwrap(), false);
    store_instruction.set_volatile(true).unwrap();
    load_instruction.set_volatile(true).unwrap();
    assert_eq!(store_instruction.get_volatile().unwrap(), true);
    assert_eq!(load_instruction.get_volatile().unwrap(), true);
    store_instruction.set_volatile(false).unwrap();
    load_instruction.set_volatile(false).unwrap();
    assert_eq!(store_instruction.get_volatile().unwrap(), false);
    assert_eq!(load_instruction.get_volatile().unwrap(), false);

    assert_eq!(store_instruction.get_alignment().unwrap(), 0);
    assert_eq!(load_instruction.get_alignment().unwrap(), 0);
    assert!(store_instruction.set_alignment(16).is_ok());
    assert!(load_instruction.set_alignment(16).is_ok());
    assert_eq!(store_instruction.get_alignment().unwrap(), 16);
    assert_eq!(load_instruction.get_alignment().unwrap(), 16);
    assert!(store_instruction.set_alignment(0).is_ok());
    assert!(load_instruction.set_alignment(0).is_ok());
    assert_eq!(store_instruction.get_alignment().unwrap(), 0);
    assert_eq!(load_instruction.get_alignment().unwrap(), 0);

    assert!(store_instruction.set_alignment(14).is_err());
    assert_eq!(store_instruction.get_alignment().unwrap(), 0);

    let fadd_instruction = builder.build_float_add(load.into_float_value(), f32_val, "").as_instruction_value().unwrap();
    assert!(fadd_instruction.get_volatile().is_err());
    assert!(fadd_instruction.set_volatile(false).is_err());
    assert!(fadd_instruction.get_alignment().is_err());
    assert!(fadd_instruction.set_alignment(16).is_err());
}

#[llvm_versions(11.0..=latest)]
#[test]
fn test_mem_instructions() {
    let context = Context::create();
    let module = context.create_module("testing");
    let builder = context.create_builder();

    let void_type = context.void_type();
    let f32_type = context.f32_type();
    let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    let fn_type = void_type.fn_type(&[f32_ptr_type.into(), f32_type.into()], false);

    let function = module.add_function("mem_inst", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");

    builder.position_at_end(basic_block);

    let arg1 = function.get_first_param().unwrap().into_pointer_value();
    let arg2 = function.get_nth_param(1).unwrap().into_float_value();

    assert!(arg1.get_first_use().is_none());
    assert!(arg2.get_first_use().is_none());

    let f32_val = f32_type.const_float(::std::f64::consts::PI);

    let store_instruction = builder.build_store(arg1, f32_val);
    let load = builder.build_load(arg1, "");
    let load_instruction = load.as_instruction_value().unwrap();

    assert_eq!(store_instruction.get_volatile().unwrap(), false);
    assert_eq!(load_instruction.get_volatile().unwrap(), false);
    store_instruction.set_volatile(true).unwrap();
    load_instruction.set_volatile(true).unwrap();
    assert_eq!(store_instruction.get_volatile().unwrap(), true);
    assert_eq!(load_instruction.get_volatile().unwrap(), true);
    store_instruction.set_volatile(false).unwrap();
    load_instruction.set_volatile(false).unwrap();
    assert_eq!(store_instruction.get_volatile().unwrap(), false);
    assert_eq!(load_instruction.get_volatile().unwrap(), false);

    assert_eq!(store_instruction.get_alignment().unwrap(), 4);
    assert_eq!(load_instruction.get_alignment().unwrap(), 4);
    assert!(store_instruction.set_alignment(16).is_ok());
    assert!(load_instruction.set_alignment(16).is_ok());
    assert_eq!(store_instruction.get_alignment().unwrap(), 16);
    assert_eq!(load_instruction.get_alignment().unwrap(), 16);
    assert!(store_instruction.set_alignment(4).is_ok());
    assert!(load_instruction.set_alignment(4).is_ok());
    assert_eq!(store_instruction.get_alignment().unwrap(), 4);
    assert_eq!(load_instruction.get_alignment().unwrap(), 4);

    assert!(store_instruction.set_alignment(14).is_err());
    assert_eq!(store_instruction.get_alignment().unwrap(), 4);

    let fadd_instruction = builder.build_float_add(load.into_float_value(), f32_val, "").as_instruction_value().unwrap();
    assert!(fadd_instruction.get_volatile().is_err());
    assert!(fadd_instruction.set_volatile(false).is_err());
    assert!(fadd_instruction.get_alignment().is_err());
    assert!(fadd_instruction.set_alignment(16).is_err());
}

#[llvm_versions(3.8..=latest)]
#[test]
fn test_atomic_ordering_mem_instructions() {
    let context = Context::create();
    let module = context.create_module("testing");
    let builder = context.create_builder();

    let void_type = context.void_type();
    let f32_type = context.f32_type();
    let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    let fn_type = void_type.fn_type(&[f32_ptr_type.into(), f32_type.into()], false);

    let function = module.add_function("mem_inst", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");

    builder.position_at_end(basic_block);

    let arg1 = function.get_first_param().unwrap().into_pointer_value();
    let arg2 = function.get_nth_param(1).unwrap().into_float_value();

    assert!(arg1.get_first_use().is_none());
    assert!(arg2.get_first_use().is_none());

    let f32_val = f32_type.const_float(::std::f64::consts::PI);

    let store_instruction = builder.build_store(arg1, f32_val);
    let load = builder.build_load(arg1, "");
    let load_instruction = load.as_instruction_value().unwrap();

    assert_eq!(store_instruction.get_atomic_ordering().unwrap(), AtomicOrdering::NotAtomic);
    assert_eq!(load_instruction.get_atomic_ordering().unwrap(), AtomicOrdering::NotAtomic);
    assert!(store_instruction.set_atomic_ordering(AtomicOrdering::Monotonic).is_ok());
    assert_eq!(store_instruction.get_atomic_ordering().unwrap(), AtomicOrdering::Monotonic);
    assert!(store_instruction.set_atomic_ordering(AtomicOrdering::Release).is_ok());
    assert!(load_instruction.set_atomic_ordering(AtomicOrdering::Acquire).is_ok());

    assert!(store_instruction.set_atomic_ordering(AtomicOrdering::Acquire).is_err());
    assert!(store_instruction.set_atomic_ordering(AtomicOrdering::AcquireRelease).is_err());
    assert!(load_instruction.set_atomic_ordering(AtomicOrdering::AcquireRelease).is_err());
    assert!(load_instruction.set_atomic_ordering(AtomicOrdering::Release).is_err());

    let fadd_instruction = builder.build_float_add(load.into_float_value(), f32_val, "").as_instruction_value().unwrap();
    assert!(fadd_instruction.get_atomic_ordering().is_err());
    assert!(fadd_instruction.set_atomic_ordering(AtomicOrdering::NotAtomic).is_err());
}

#[test]
fn test_metadata_kinds() {
    let context = Context::create();

    let i8_type = context.i8_type();
    let f32_type = context.f32_type();
    let ptr_type = i8_type.ptr_type(AddressSpace::Generic);
    let struct_type = context.struct_type(&[i8_type.into(), f32_type.into()], false);
    let vector_type = i8_type.vec_type(2);

    let i8_value = i8_type.const_zero();
    let i8_array_value = i8_type.const_array(&[i8_value]);
    let f32_value = f32_type.const_zero();
    let ptr_value = ptr_type.const_null();
    let struct_value = struct_type.get_undef();
    let vector_value = vector_type.const_zero();

    let md_string = context.metadata_string("lots of metadata here");
    context.metadata_node(&[
        i8_array_value.into(),
        i8_value.into(),
        f32_value.into(),
        ptr_value.into(),
        struct_value.into(),
        vector_value.into(),
        md_string.into(),
    ]);
}
