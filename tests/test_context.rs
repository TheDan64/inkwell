extern crate inkwell;

use self::inkwell::context::Context;
use self::inkwell::types::IntType;

#[test]
fn test_no_context_double_free() {
    let context = Context::create();
    let int = context.i8_type();

    {
        int.get_context();
    }
}

#[test]
fn test_no_context_double_free2() {
    let context = Context::create();
    let int = context.i8_type();
    let context2 = int.get_context();

    fn move_(_: Context) {}

    move_(context);

    context2.i8_type().const_int(0, false);
}

#[test]
fn test_no_context_double_free3() {
    Context::get_global_context();
    Context::get_global_context();
}

#[test]
fn test_get_context_from_contextless_value() {
    let int = IntType::i8_type();
    let context = Context::create();
    let global_context = Context::get_global_context();

    assert_eq!(int.get_context(), global_context);
    assert_ne!(*int.get_context(), context);
    assert_ne!(*global_context, context);
}
