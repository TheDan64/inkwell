extern crate inkwell;

use self::inkwell::context::Context;

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
