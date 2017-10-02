//! This is an example of the [Kaleidoscope tutorial](https://llvm.org/docs/tutorial/)
//! made in Rust, using Inkwell.
//! Currently, all features up to the [4th chapter](https://llvm.org/docs/tutorial/LangImpl04.html)
//! are available.

extern crate inkwell;

use std::collections::HashMap;
use std::io::{self, Write};
use std::iter::Peekable;
use std::str::Chars;
use std::ops::DerefMut;

use inkwell::FloatPredicate;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::targets::{InitializationConfig, Target};
use inkwell::types::BasicType;
use inkwell::values::{BasicValue, FloatValue, FunctionValue, PhiValue};

use Token::*;

const ANONYMOUS_FUNCTION_NAME: &str = "anonymous";


// ======================================================================================
// LEXER ================================================================================
// ======================================================================================

/// Represents a primitive syntax token.
#[derive(Debug, Clone)]
pub enum Token {
    EOF,
    Comment,
    LParen,
    RParen,
    Comma,
    Def, Extern,
    If, Then, Else,
    For, In,
    Unary, Binary,

    Ident(String),
    Number(f64),
    Op(char)
}

/// Defines an error encountered by the `Lexer`.
pub struct LexError {
    pub error: &'static str,
    pub index: usize
}

impl LexError {
    pub fn new(msg: &'static str) -> LexError {
        LexError { error: msg, index: 0 }
    }

    pub fn with_index(msg: &'static str, index: usize) -> LexError {
        LexError { error: msg, index: index }
    }
}

/// Defines the result of a lexing operation; namely a
/// `Token` on success, or a `LexError` on failure.
pub type LexResult = Result<Token, LexError>;

/// Defines a lexer which transforms an input `String` into
/// a `Token` stream.
pub struct Lexer<'a> {
    input: &'a str,
    chars: Box<Peekable<Chars<'a>>>,
    pos: usize
}

impl<'a> Lexer<'a> {
    /// Creates a new `Lexer`, given its source `input`.
    pub fn new(input: &'a str) -> Lexer<'a> {
        Lexer { input: input, chars: Box::new(input.chars().peekable()), pos: 0 }
    }

    /// Lexes and returns the next `Token` from the source code.
    pub fn lex(&mut self) -> LexResult {
        let chars = self.chars.deref_mut();
        let src = self.input;

        let mut pos = self.pos;

        // Skip whitespaces
        loop {
            {
                let ch = chars.peek();

                if ch.is_none() {
                    self.pos = pos;

                    return Ok(Token::EOF);
                }

                if !ch.unwrap().is_whitespace() {
                    break;
                }
            }

            {
                chars.next();
                pos += 1;
            }
        }

        let start = pos;
        let next = chars.next();

        if next.is_none() {
            return Ok(Token::EOF);
        }

        pos += 1;

        // Actually get the next token.
        let result = match next.unwrap() {
            '(' => Ok(Token::LParen),
            ')' => Ok(Token::RParen),
            ',' => Ok(Token::Comma),

            '#' => {
                // Comment
                loop {
                    let ch = chars.next();
                    pos += 1;

                    if ch == Some('\n') {
                        break;
                    }
                }

                Ok(Token::Comment)
            },

            ch @ '.' | ch @ '0' ... '9' => {
                // Parse number literal
                loop {
                    let ch = match chars.peek() {
                        Some(ch) => *ch,
                        None => { return Ok(Token::EOF); }
                    };

                    // Parse float.
                    if ch != '.' && !ch.is_digit(16) {
                        break;
                    }

                    chars.next();
                    pos += 1;
                }
                
                Ok(Token::Number(src[start..pos].parse().unwrap()))
            },

            'a' ... 'z' | 'A' ... 'Z' | '_' => {
                // Parse identifier                
                loop {
                    let ch = match chars.peek() {
                        Some(ch) => *ch,
                        None => { return Ok(Token::EOF); }
                    };

                    // A word-like identifier only contains underscores and alphanumeric characters.
                    if ch != '_' && !ch.is_alphanumeric() {
                        break;
                    }

                    chars.next();
                    pos += 1;
                }

                match &src[start..pos] {
                    "def" => Ok(Token::Def),
                    "extern" => Ok(Token::Extern),
                    "if" => Ok(Token::If),
                    "then" => Ok(Token::Then),
                    "else" => Ok(Token::Else),
                    "for" => Ok(Token::For),
                    "in" => Ok(Token::In),
                    "unary" => Ok(Token::Unary),
                    "binary" => Ok(Token::Binary),

                    ident => Ok(Token::Ident(ident.to_string()))
                }
            },

            op => {
                // Parse operator
                Ok(Token::Op(op))
            }
        };

        // Update stored position, and return
        self.pos = pos;

        result
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    /// Lexes the next `Token` and returns it.
    /// On EOF or failure, `None` will be returned.
    fn next(&mut self) -> Option<Self::Item> {
        match self.lex() {
            Ok(EOF) | Err(_) => None,
            Ok(token) => Some(token)
        }
    }
}


// ======================================================================================
// PARSER ===============================================================================
// ======================================================================================

/// Defines a primitive expression.
#[derive(Debug)]
pub enum Expr {
    Number(f64),
    Variable(String),

    Binary { 
        op: char,
        left: Box<Expr>,
        right: Box<Expr>
    },

    Call {
        fn_name: String,
        args: Vec<Expr>
    },

    Conditional {
        cond: Box<Expr>,
        consequence: Box<Expr>,
        alternative: Box<Expr>
    },

    For {
        var_name: String,
        start: Box<Expr>,
        end: Box<Expr>,
        step: Option<Box<Expr>>,
        body: Box<Expr>
    }
}

/// Defines the prototype (name and parameters) of a function.
#[derive(Debug)]
pub struct Prototype {
    pub name: String,
    pub args: Vec<String>,
    pub is_op: bool,
    pub prec: usize
}

/// Defines a user-defined function.
#[derive(Debug)]
pub struct Function {
    pub prototype: Prototype,
    pub body: Expr,
    pub is_anon: bool
}

/// Represents the `Expr` parser.
pub struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    prec: &'a mut HashMap<char, i32>
}

impl<'a> Parser<'a> {
    /// Creates a new parser, given an input `str` and a `HashMap` binding
    /// an operator and its precedence in binary expressions.
    pub fn new(input: String, op_precedence: &'a mut HashMap<char, i32>) -> Self {
        let mut lexer = Lexer::new(input.as_str());
        let tokens = lexer.by_ref().collect();

        Parser {
            tokens: tokens,
            prec: op_precedence,
            pos: 0
        }
    }

    /// Parses the content of the parser.
    pub fn parse(&mut self) -> Result<Function, &'static str> {
        match self.curr() {
            Def => self.parse_def(),
            Extern => self.parse_extern(),
            _ => self.parse_toplevel_expr()
        }
    }

    /// Returns the current `Token`, without performing safety checks beforehand.
    pub fn curr(&self) -> Token {
        self.tokens[self.pos].clone()
    }

    /// Returns the current `Token`, or an error that
    /// indicates that the end of the file has been unexpectedly reached if it is the case.
    pub fn current(&self) -> Result<Token, &'static str> {
        if self.pos >= self.tokens.len() {
            Err("Unexpected end of file.")
        } else {
            Ok(self.tokens[self.pos].clone())
        }
    }

    /// Advances the position, and returns an empty `Result` whose error
    /// indicates that the end of the file has been unexpectedly reached.
    /// This allows to use the `self.advance()?;` syntax.
    pub fn advance(&mut self) -> Result<(), &'static str> {
        let npos = self.pos + 1;

        self.pos = npos;

        if npos < self.tokens.len() {
            Ok(())
        } else {
            Err("Unexpected end of file.")
        }
    }

    /// Returns a value indicating whether or not the `Parser`
    /// has reached the end of the input.
    pub fn at_end(&self) -> bool {
        self.pos > self.tokens.len() - 2
    }

    /// Returns the precedence of the current `Token`, or 0 if it is not recognized as a binary operator.
    fn get_tok_precedence(&self) -> i32 {
        if let Ok(Op(op)) = self.current() {
            *self.prec.get(&op).unwrap_or(&100)
        } else {
            -1
        }
    }

    /// Parses the prototype of a function, whether external or user-defined.
    fn parse_prototype(&mut self) -> Result<Prototype, &'static str> {
        let (id, is_operator, precedence) = match self.curr() {
            Ident(id) => {
                self.advance()?;

                (id, false, 0)
            },

            Binary => {
                self.advance()?;

                let op = match self.curr() {
                    Op(ch) => ch,
                    _ => { return Err("Expected operator in custom operator declaration."); }
                };

                self.advance()?;

                let mut name = String::from("binary");

                name.push(op);

                let prec = if let Number(prec) = self.curr() {
                    self.advance()?;

                    prec as usize
                } else {
                    0
                };

                self.prec.insert(op, prec as i32);

                (name, true, prec)
            },

            Unary => {
                self.advance()?;

                let op = match self.curr() {
                    Op(ch) => ch,
                    _ => { return Err("Expected operator in custom operator declaration."); }
                };

                let mut name = String::from("unary");

                name.push(op);

                self.advance()?;

                (name, true, 0)
            },

            _ => { return Err("Expected identifier in prototype declaration.") }
        };

        match self.curr() {
            LParen => (),
            _ => { return Err("Expected '(' character in prototype declaration.") }
        }

        self.advance()?;

        if let RParen = self.curr() {
            self.advance();
            return Ok(Prototype { name: id, args: vec![], is_op: is_operator, prec: precedence });
        }

        let mut args = vec![];

        loop {
            match self.curr() {
                Ident(name) => args.push(name),
                _ => { return Err("Expected identifier in parameter declaration.") }
            }

            self.advance()?;

            match self.curr() {
                RParen => {
                    self.advance();
                    break;
                },
                Comma => {
                    self.advance();   
                },
                _ => { return Err("Expected ',' or ')' character in prototype declaration.") }
            }
        }

        Ok(Prototype { name: id, args: args, is_op: is_operator, prec: precedence })
    }

    /// Parses a user-defined function.
    fn parse_def(&mut self) -> Result<Function, &'static str> {
        // Eat 'def' keyword
        self.pos += 1;

        // Parse signature of function
        let proto = self.parse_prototype()?;

        // Parse body of function
        let body = self.parse_expr()?;

        // Return new function
        Ok(Function { prototype: proto, body: body, is_anon: false })
    }

    /// Parses an external function declaration.
    fn parse_extern(&mut self) -> Result<Function, &'static str> {
        // Eat 'extern' keyword
        self.pos += 1;

        // Parse signature of extern function
        let proto = self.parse_prototype()?;

        // Return signature of extern function
        Ok(Function { prototype: proto, body: Expr::Number(std::f64::NAN), is_anon: false })
    }

    /// Parses any expression.
    fn parse_expr(&mut self) -> Result<Expr, &'static str> {
        match self.parse_unary_expr() {
            Ok(left) => self.parse_binary_expr(0, left),
            err => err
        }
    }

    /// Parses a literal number.
    fn parse_nb_expr(&mut self) -> Result<Expr, &'static str> {
        // Simply convert Token::Number to Expr::Number
        match self.curr() {
            Number(nb) => {
                self.advance();
                Ok(Expr::Number(nb))
            },
            _ => Err("Expected number literal.")
        }
    }

    /// Parses an expression enclosed in parenthesis.
    fn parse_paren_expr(&mut self) -> Result<Expr, &'static str> {
        match self.current()? {
            LParen => (),
            _ => { return Err("Expected '(' character at start of parenthesized expression.") }
        }

        self.advance()?;

        let expr = self.parse_expr()?;

        match self.current()? {
            RParen => (),
            _ => { return Err("Expected ')' character at end of parenthesized expression.") }
        }

        self.advance();

        Ok(expr)
    }

    /// Parses an expression that starts with an identifier (either a variable or a function call).
    fn parse_id_expr(&mut self) -> Result<Expr, &'static str> {
        let id = match self.curr() {
            Ident(id) => id,
            _ => { return Err("Expected identifier."); }
        };

        if let Err(_) = self.advance() {
            return Ok(Expr::Variable(id));
        }

        match self.curr() {
            LParen => {
                self.advance()?;

                if let RParen = self.curr() {
                    return Ok(Expr::Call { fn_name: id, args: vec![] });
                }

                let mut args = vec![];
                
                loop {
                    args.push(self.parse_expr()?);

                    match self.current()? {
                        Comma => (),
                        RParen => break,
                        _ => { return Err("Expected ',' character in function call."); }
                    }

                    self.advance()?;
                }

                self.advance();

                Ok(Expr::Call { fn_name: id, args: args })
            },

            _ => Ok(Expr::Variable(id))
        }
    }

    /// Parses an unary expression.
    fn parse_unary_expr(&mut self) -> Result<Expr, &'static str> {
        let op = match self.current()? {
            Op(ch) => {
                self.advance()?;
                ch
            },
            _ => { return self.parse_primary(); }
        };

        let mut name = String::from("unary");

        name.push(op);

        Ok(Expr::Call { fn_name: name, args: vec![ self.parse_unary_expr()? ] })
    }

    /// Parses a binary expression, given its left-hand expression.
    fn parse_binary_expr(&mut self, prec: i32, left: Expr) -> Result<Expr, &'static str> {
        let mut left = left;

        loop {
            let curr_prec = self.get_tok_precedence();

            if curr_prec < prec || self.at_end() {
                return Ok(left);
            }

            let op = match self.curr() {
                Op(op) => op,
                _ => { return Err("Invalid operator."); }
            };

            self.advance()?;

            let mut right = self.parse_unary_expr()?;

            let next_prec = self.get_tok_precedence();

            if curr_prec < next_prec {
                right = self.parse_binary_expr(curr_prec + 1, right)?;
            }
            
            left = Expr::Binary {
                op: op,
                left: Box::new(left),
                right: Box::new(right)
            };
        }
    }

    /// Parses a conditional if..then..else expression.
    fn parse_conditional_expr(&mut self) -> Result<Expr, &'static str> {
        // eat 'if' token
        self.advance()?;

        let cond = self.parse_expr()?;

        // eat 'then' token
        match self.current() {
            Ok(Then) => { self.advance()? },
            _ => { return Err("Expected 'then' keyword."); }
        }

        let then = self.parse_expr()?;

        // eat 'else' token
        match self.current() {
            Ok(Else) => { self.advance()? },
            _ => { return Err("Expected 'else' keyword."); }
        }

        let otherwise = self.parse_expr()?;

        Ok(Expr::Conditional {
            cond: Box::new(cond),
            consequence: Box::new(then),
            alternative: Box::new(otherwise)
        })
    }

    /// Parses a loop for..in.. expression.
    fn parse_for_expr(&mut self) -> Result<Expr, &'static str> {
        // eat 'for' token
        self.advance()?;

        let name = match self.curr() {
            Ident(n) => n,
            _ => { return Err("Expected identifier in for loop."); }
        };

        // eat identifier
        self.advance()?;

        // eat '=' token
        match self.curr() {
            Op('=') => { self.advance()?; },
            _ => { return Err("Expected '=' character in for loop."); }
        }

        let start = self.parse_expr()?;

        // eat ',' token
        match self.current()? {
            Comma => { self.advance()?; },
            _ => { return Err("Expected ',' character in for loop."); }
        }

        let end = self.parse_expr()?;

        // parse (optional) step expression
        let step = match self.current()? {
            Comma => {
                self.advance()?;

                Some(self.parse_expr()?)
            },

            _ => None
        };

        // eat 'in' token
        match self.current()? {
            In => { self.advance()?; },
            _ => { return Err("Expected 'in' keyword in for loop."); }
        }

        let body = self.parse_expr()?;

        Ok(Expr::For {
            var_name: name,
            start: Box::new(start),
            end: Box::new(end),
            step: step.map(Box::new),
            body: Box::new(body)
        })
    }

    /// Parses a primary expression (an identifier, a number or a parenthesized expression).
    fn parse_primary(&mut self) -> Result<Expr, &'static str> {
        match self.curr() {
            Ident(_) => self.parse_id_expr(),
            Number(_) => self.parse_nb_expr(),
            LParen => self.parse_paren_expr(),
            If => self.parse_conditional_expr(),
            For => self.parse_for_expr(),
            _ => Err("Unknown expression.")
        }
    }

    /// Parses a top-level expression and makes an anonymous function out of it,
    /// for easier compilation.
    fn parse_toplevel_expr(&mut self) -> Result<Function, &'static str> {
        match self.parse_expr() {
            Ok(expr) => {
                Ok(Function {
                    prototype: Prototype {
                        name: ANONYMOUS_FUNCTION_NAME.to_string(),
                        args: vec![],
                        is_op: false,
                        prec: 0
                    },
                    body: expr,
                    is_anon: true
                })
            },

            Err(err) => Err(err)
        }
    }
}


// ======================================================================================
// COMPILER =============================================================================
// ======================================================================================

/// Defines the `Expr` compiler.
pub struct Compiler<'a> {
    pub context: &'a Context,
    pub builder: &'a Builder,
    pub fpm: &'a PassManager,
    pub module: &'a Module,
    pub function: &'a Function,
    pub fn_value: FunctionValue,
    pub variables: HashMap<String, FloatValue>
}

fn to_float_value(phi: PhiValue) -> FloatValue {
    unsafe { std::mem::transmute(phi) }
}

impl<'a> Compiler<'a> {
    /// Gets a defined function given its name.
    pub fn get_function(&self, name: &str) -> Option<FunctionValue> {
        self.module.get_function(name)
    }

    /// Compiles the specified `Expr` into an LLVM `BasicValue`.
    /// Note: This method currently returns a `Box`, since it returns either `PhiValue` or `FloatValue`.
    pub fn compile_expr(&mut self, expr: &Expr) -> Result<FloatValue, &'static str> {
        match expr {
            &Expr::Number(nb) => Ok(self.context.f64_type().const_float(nb)),

            &Expr::Variable(ref name) => {
                match self.variables.get(name.as_str()) {
                    Some(var) => Ok(*var),
                    None => Err("Could not find a matching variable.")
                }
            },

            &Expr::Binary { op, ref left, ref right } => {
                let lhs = self.compile_expr(&left)?;
                let rhs = self.compile_expr(&right)?;

                match op {
                    '+' => Ok(self.builder.build_float_add(&lhs, &rhs, "tmpadd")),
                    '-' => Ok(self.builder.build_float_sub(&lhs, &rhs, "tmpsub")),
                    '*' => Ok(self.builder.build_float_mul(&lhs, &rhs, "tmpmul")),
                    '/' => Ok(self.builder.build_float_div(&lhs, &rhs, "tmpdiv")),

                    '<' => Ok({
                        let cmp = self.builder.build_float_compare(&FloatPredicate::ULT, &lhs, &rhs, "tmpcmp");

                        self.builder.build_unsigned_int_to_float(&cmp, &self.context.f64_type(), "tmpbool")
                    }),
                    '>' => Ok({
                        let cmp = self.builder.build_float_compare(&FloatPredicate::ULT, &rhs, &lhs, "tmpcmp");

                        self.builder.build_unsigned_int_to_float(&cmp, &self.context.f64_type(), "tmpbool")
                    }),

                    custom => {
                        let mut name = String::from("binary");

                        name.push(custom);

                        match self.get_function(name.as_str()) {
                            Some(fun) => {
                                match self.builder.build_call(&fun, &[ &lhs, &rhs ], "tmpbin", false).left() {
                                    Some(value) => Ok(value.into_float_value()),
                                    None => Err("Invalid call produced.")
                                }
                            },

                            None => Err("Undefined binary operator.")
                        }
                    }
                }
            },

            &Expr::Call { ref fn_name, ref args } => {
                match self.get_function(fn_name.as_str()) {
                    Some(fun) => {
                        let mut compiled_args = Vec::with_capacity(args.len());

                        for arg in args {
                            compiled_args.push(self.compile_expr(arg)?);
                        }

                        let argsv: Vec<&BasicValue> = compiled_args.iter().by_ref().map(|val| val as &BasicValue).collect();

                        match self.builder.build_call(&fun, argsv.as_slice(), "tmp", false).left() {
                            Some(value) => Ok(value.into_float_value()),
                            None => Err("Invalid call produced.")
                        }
                    },
                    None => Err("Unknown function.")
                }
            },

            &Expr::Conditional { ref cond, ref consequence, ref alternative } => {
                let parent = self.fn_value;
                let zero_const = self.context.f64_type().const_float(0.0);

                // create condition by comparing without 0.0 and returning an int
                let cond = self.compile_expr(&cond)?;
                let cond = self.builder.build_float_compare(&FloatPredicate::ONE, &cond, &zero_const, "ifcond");

                // build branch
                let then_bb = self.context.append_basic_block(&parent, "then");
                let else_bb = self.context.append_basic_block(&parent, "else");
                let cont_bb = self.context.append_basic_block(&parent, "ifcont");

                self.builder.build_conditional_branch(&cond, &then_bb, &else_bb);

                // build then block
                self.builder.position_at_end(&then_bb);
                let then_val = self.compile_expr(&consequence)?;
                self.builder.build_unconditional_branch(&cont_bb);

                let then_bb = self.builder.get_insert_block().unwrap();

                // build else block
                self.builder.position_at_end(&else_bb);
                let else_val = self.compile_expr(&alternative)?;
                self.builder.build_unconditional_branch(&cont_bb);

                let else_bb = self.builder.get_insert_block().unwrap();

                // emit merge block
                self.builder.position_at_end(&cont_bb);
                
                let phi = self.builder.build_phi(&self.context.f64_type(), "iftmp");

                phi.add_incoming(&[
                    (&then_val, &then_bb),
                    (&else_val, &else_bb)
                ]);

                Ok(to_float_value(phi))
            },

            &Expr::For { ref var_name, ref start, ref end, ref step, ref body } => {
                let parent = self.fn_value;
                let zero_const = self.context.f64_type().const_float(0.0);

                let start = self.compile_expr(&start)?;
                let preheader_bb = self.builder.get_insert_block().unwrap();
                let loop_bb = self.context.append_basic_block(&parent, "loop");

                // go from current block to loop block
                self.builder.build_unconditional_branch(&loop_bb);

                self.builder.position_at_end(&loop_bb);

                let variable = self.builder.build_phi(&self.context.f64_type(), var_name);
                
                variable.add_incoming(&[
                    (&start, &preheader_bb)
                ]);

                let old_val = match self.variables.get(var_name.as_str()) {
                    Some(val) => Some(*val),
                    None => None
                };

                self.variables.insert(var_name.to_owned(), to_float_value(variable));

                self.compile_expr(&body)?;

                let step = match step {
                    &Some(ref step) => self.compile_expr(&step)?,
                    &None => self.context.f64_type().const_float(1.0)
                };

                let variable_f = to_float_value(variable);
                let next_var = self.builder.build_float_add(&variable_f, &step, "nextvar");

                let end_cond = self.compile_expr(&end)?;
                let end_cond = self.builder.build_float_compare(&FloatPredicate::ONE, &end_cond, &zero_const, "loopcond");

                let loop_end_bb = self.builder.get_insert_block().unwrap();
                let after_bb = self.context.append_basic_block(&parent, "afterloop");

                self.builder.build_conditional_branch(&end_cond, &loop_bb, &after_bb);
                self.builder.position_at_end(&after_bb);

                variable.add_incoming(&[
                    (&next_var, &loop_end_bb)
                ]);

                self.variables.remove(var_name);

                if let Some(val) = old_val {
                    self.variables.insert(var_name.to_owned(), val);
                }

                Ok(zero_const)
            }
        }
    }

    /// Compiles the specified `Prototype` into an extern LLVM `FunctionValue`.
    fn compile_prototype(&self, proto: &Prototype) -> Result<FunctionValue, &'static str> {
        let ret_type: &BasicType = &self.context.f64_type();
        let args_types = std::iter::repeat(ret_type).take(proto.args.len()).collect::<Vec<&BasicType>>();
        let args_types = args_types.as_slice();

        let fn_type = self.context.f64_type().fn_type(args_types, false);
        let fn_val = self.module.add_function(proto.name.as_str(), &fn_type, None);

        // set arguments names
        for (i, arg) in fn_val.params().enumerate() {
            arg.into_float_value().set_name(proto.args[i].as_str());
        }

        // finally return built prototype
        Ok(fn_val)
    }

    /// Compiles the specified `Function` into an LLVM `FunctionValue`.
    fn compile_fn(&mut self) -> Result<FunctionValue, &'static str> {
        let proto = &self.function.prototype;
        let function = self.compile_prototype(proto)?;
        let entry = self.context.append_basic_block(&function, "entry");

        self.builder.position_at_end(&entry);

        // build variables map
        {
            // this is in its own scope to borrow as shortly as possible
            let variables = &mut self.variables;

            variables.reserve(proto.args.len());

            for (i, arg) in function.params().enumerate() {
                variables.insert(proto.args[i].clone(), arg.into_float_value());
            }
        }

        // update fields
        self.fn_value = function;

        // compile body
        let body = self.compile_expr(&self.function.body)?;

        self.builder.build_return(Some(&body));

        // return the whole thing after verification and optimization
        if function.verify(true) {
            self.fpm.run_on_function(&function);

            Ok(function)
        } else {
            unsafe {
                function.delete();
            }

            Err("Invalid generated function.")
        }
    }

    pub fn compile(context: &'a Context, builder: &'a Builder, pass_manager: &'a PassManager, module: &'a Module, function: &Function) -> Result<FunctionValue, &'static str> {
        let mut compiler = Compiler {
            context: context,
            builder: builder,
            fpm: pass_manager,
            module: module,
            function: function,
            fn_value: unsafe { std::mem::transmute(0usize) },
            variables: HashMap::new()
        };

        compiler.compile_fn()
    }
}


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

pub extern fn putchard(x: f64) -> f64 {
    print_flush!("{}", x as u8 as char);
    x
}

pub extern fn printd(x: f64) -> f64 {
    println!("Fn called");
    println!("{}", x);
    x
}

/// Entry point of the program; acts as a REPL.
pub fn main() {
    // use self::inkwell::support::add_symbol;

    let show_lexed_input = true;
    let show_parsed_input = false;
    let show_compiled_input = true;

    Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target.");

    // add_symbol("printd", &printd, printd as *const ());
    // add_symbol("putchard", &putchard, putchard as *const ());
    
    let context = Context::create();
    let module = context.create_module("repl");
    let builder = context.create_builder();

    // Create FPM
    let fpm = PassManager::create_for_function(&module);

    fpm.add_instruction_combining_pass();
    fpm.add_reassociate_pass();
    fpm.add_gvn_pass();
    fpm.add_cfg_simplification_pass();
    fpm.add_basic_alias_analysis_pass();

    fpm.initialize();

    let mut previous_exprs = Vec::new();

    loop {
        print_flush!("> ");

        // Read input from stdin
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Could not read from standard input.");

        if input.starts_with("exit") {
            break;
        }

        // Build precedence map
        let mut prec = HashMap::with_capacity(4);

        prec.insert('<', 10);
        prec.insert('+', 20);
        prec.insert('-', 20);
        prec.insert('*', 40);
        prec.insert('/', 40);

        // Parse and (optionally) display input
        if show_lexed_input {
            println!("Attempting to parse lexed input: {:?}", Lexer::new(input.as_str()).collect::<Vec<Token>>());
        }

        // make module
        let module = context.create_module("tmp");
        let mut printd_fn = None;
        let mut putchard_fn = None;

        for prev in previous_exprs.iter() {
            let fun = Compiler::compile(&context, &builder, &fpm, &module, prev).expect("Cannot re-add previously compiled function.");

            match fun.get_name().to_str().unwrap() {
                "printd" => {
                    println!("Setting printd to {:?}", fun);
                    printd_fn = Some(fun);
                },

                "putchard" => {
                    putchard_fn = Some(fun);
                },

                _ => ()
            }
        }

        let (name, is_anonymous) = match Parser::new(input, &mut prec).parse() {
            Ok(fun) => {
                if show_parsed_input {
                    println!("Expression parsed: {:?}", fun);
                }
                
                match Compiler::compile(&context, &builder, &fpm, &module, &fun) {
                    Ok(function) => { 
                        if show_compiled_input {
                            // Not printing a new line since LLVM automatically
                            // prefixes the generated string with one
                            print_flush!("Expression compiled to IR:");
                            function.print_to_stderr();
                        }

                        let fn_name = function.get_name().to_str().unwrap();

                        if fn_name.starts_with(ANONYMOUS_FUNCTION_NAME) {
                            (fn_name.to_string(), true)
                        } else {
                            previous_exprs.push(fun);

                            (fn_name.to_string(), false)
                        }
                    },
                    Err(err) => {
                        println!("Error compiling function: {}", err);
                        continue;
                    }
                }
            },
            Err(err) => {
                println!("Error parsing expression: {}", err);
                continue;
            }
        };

        if is_anonymous {
            let mut ee = module.create_jit_execution_engine(0).unwrap();

            // 2017-02-10 <6A> I still can't add my own functions with either add_global_mapping or add_symbol.
            //                 However, importing extern functions such as cos(x) or sin(x) works.

            // if let Some(fun) = printd_fn {
            //     println!("Setting global mapping for {:p} {:p} {:p}", &printd, &mut printd, *printd);
            //     ee.add_global_mapping(&fun, unsafe { std::mem::transmute(&mut printd) });
            // }

            // if let Some(fun) = putchard_fn {
            //     ee.add_global_mapping(&fun, unsafe { std::mem::transmute(&putchard) });
            // }

            let addr = match ee.get_function_address(name.as_str()) {
                Ok(addr) => addr,
                Err(err) => {
                    println!("Error during execution: {:?}", err);
                    continue;
                }
            };

            let compiled_fn: extern "C" fn() -> f64 = unsafe { std::mem::transmute(addr) };

            println!("Result: {}", compiled_fn());
        }
    }
}
