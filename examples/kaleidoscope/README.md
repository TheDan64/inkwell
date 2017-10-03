# Kaleidoscope

This example shows how one can implement the [Kaleidoscope programming language](https://llvm.org/docs/tutorial/index.html) using Inkwell.  
It implements every feature up to the [7th chapter](https://llvm.org/docs/tutorial/LangImpl07.html).

When running this example (using the `cargo run --example kaleidoscope` command), a prompt will be displayed; for example:

```
?> 1 + 1
=> 2

?> var a = 5, b = 10 in a * b
=> 50

?> def fib(n) if n < 2 then n else fib(n - 1) + fib(n - 2)

?> fib(40)
=> 102334155

?>
```

Additional arguments can be passed to the produced executable:
- `--dc`: **D**isplay **C**ompiler output
- `--dp`: **D**isplay **P**arser output
- `--dl`: **D**isplay **L**exer output

For example, running with all three switches may lead to the following output:
```
?> 1 + 2 * 2
-> Attempting to parse lexed input:
[Number(1), Op('+'), Number(2), Op('*'), Number(2)]

-> Expression parsed:
Binary { op: '+', left: Number(1), right: Binary { op: '*', left: Number(2), right: Number(2) } }

-> Expression compiled to IR:
define double @anonymous() {
entry:
  ret double 5.000000e+00
}

=> 5
```

Finally, the prompt can be exited by entering "exit" or "quit".