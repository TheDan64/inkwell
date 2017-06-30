# Inkwell(s)

**I**t's a **N**ew **K**ind of **W**rapper for **E**xposing **LL**VM (*S*afely)

Inkwell aims to help you pen your own programming languages by safely wrapping llvm-sys. It provides a more strongly typed interface than LLVM itself so that certain types of errors can be caught at compile time instead of at LLVM's runtime. The ultimate goal is to make LLVM safer on the rust end and easier to use. Inkwell currently compiles on stable, though this may be subject to change in the future.
