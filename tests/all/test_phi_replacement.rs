use inkwell::context::Context;

#[test]
fn test_phi_replacement() {
    let context = Context::create();
    let module = context.create_module("phi");
    let builder = context.create_builder();

    let bool_type = context.bool_type();
    let fn_type = bool_type.fn_type(&[], false);
    let function = module.add_function("do_stuff", fn_type, None);
    let entry_block = context.append_basic_block(function, "entry");
    let return_block = context.append_basic_block(function, "return");
    builder.position_at_end(entry_block);

    let phi = builder.build_phi(bool_type, "phi_node");

    builder.position_at_end(return_block);
    builder.build_return(Some(&phi.as_basic_value()));

    builder.position_at_end(entry_block);
    let return_value = builder.build_and(
        bool_type.const_int(0, false),
        bool_type.const_int(1, false),
        "and",
    );
    phi.replace_all_uses_with(return_value);
    phi.as_instruction().erase_from_basic_block();
    builder.build_unconditional_branch(return_block);

    module.verify().unwrap();
}
