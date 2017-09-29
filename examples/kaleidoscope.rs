extern crate inkwell;

use std::collections::HashMap;
use std::io;
use std::iter::Peekable;
use std::str::Chars;
use std::ops::DerefMut;

use inkwell::builder::Builder;
use inkwell::types::FloatType;
use inkwell::values::{BasicValue, FloatValue, FunctionValue};

use Token::*;


// ======================================================================================
// LEXER ================================================================================
// ======================================================================================

/// Represents a primitive syntax token.
#[derive(Clone)]
pub enum Token {
    EOF,
    Comment,
    LParen,
    RParen,
    Comma,
    Def,
    Extern,
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
                    let ch = *chars.peek();

                    if ch.is_none() {
                        return Ok(Token::EOF);
                    }

                    let ch = ch.unwrap();

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
                    let ch = chars.peek();

                    if ch.is_none() {
                        break;
                    }

                    let ch = *ch.unwrap();

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
    Binary(char, Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>)
}

/// Defines the prototype (name and parameters) of a function.
#[derive(Debug)]
pub struct Prototype {
    pub name: String,
    pub args: Vec<String>
}

/// Defines a user-defined function.
#[derive(Debug)]
pub struct Function {
    pub prototype: Prototype,
    pub body: Expr
}

/// Represents the `Expr` parser.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    prec: HashMap<&'static str, i32>
}

impl Parser {
    /// Creates a new parser, given an input `str` and a `HashMap` binding
    /// an operator and its precedence in binary expressions.
    pub fn new(input: String, op_precedence: HashMap<&'static str, i32>) -> Parser {
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
            Ident(ref id) if id == "def" => self.parse_def(),
            Ident(ref id) if id == "extern" => self.parse_extern(),
            _ => self.parse_toplevel_expr()
        }
    }

    /// Returns the current `Token`.
    pub fn curr(&self) -> Token {
        self.tokens[self.pos].clone()
    }

    /// Returns the current `Token`, or `None` if the end of the input has been reached.
    pub fn current(&self) -> Option<Token> {
        if self.pos >= self.tokens.len() {
            None
        } else {
            Some(self.tokens[self.pos].clone())
        }
    }

    /// Advances the position, and returns whether or not a new
    /// `Token` is available.
    pub fn advance(&mut self) -> bool {
        let npos = self.pos + 1;

        self.pos = npos;

        npos < self.tokens.len()
    }

    /// Returns a value indicating whether or not the `Parser`
    /// has reached the end of the input.
    pub fn at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    /// Returns the precedence of the current `Token`, or 0 if it is not recognized as a binary operator.
    fn get_tok_precedence(&self) -> i32 {
        if let Some(Token::Ident(id)) = self.current() {
            *self.prec.get(id.as_str()).unwrap_or(&100)
        } else {
            -1
        }
    }

    /// Parses the prototype of a function, whether external or user-defined.
    fn parse_prototype(&mut self) -> Result<Prototype, &'static str> {
        let id = match self.curr() {
            Ident(id) => id,
            _ => { return Err("Expected identifier in prototype declaration.") }
        };

        self.advance();

        match self.curr() {
            LParen => (),
            _ => { return Err("Expected '(' character in prototype declaration.") }
        }

        self.advance();

        let mut args = vec![];

        loop {
            match self.curr() {
                Ident(name) => args.push(name),
                _ => { return Err("Expected identifier in parameter declaration.") }
            }

            self.advance();

            match self.curr() {
                RParen => break,
                Comma => (),
                _ => { return Err("Expected ',' or ')' character in prototype declaration.") }
            }
        }

        Ok(Prototype { name: id, args: args })
    }

    /// Parses a user-defined function.
    fn parse_def(&mut self) -> Result<Function, &'static str> {
        // Eat 'def' keyword
        self.pos += 1;

        // Parse signature of function
        let proto = self.parse_prototype();

        if let Err(err) = proto {
            return Err(err);
        }

        // Parse body of function
        let body = self.parse_expr();

        if let Err(err) = body {
            return Err(err);
        }

        // Return new function
        Ok(Function { prototype: proto.unwrap(), body: body.unwrap() })
    }

    /// Parses an external function declaration.
    fn parse_extern(&mut self) -> Result<Function, &'static str> {
        // Eat 'extern' keyword
        self.pos += 1;

        // Parse signature of extern function
        let proto = self.parse_prototype();

        if let Err(err) = proto {
            return Err(err);
        }

        // Return signature of extern function
        Ok(Function { prototype: proto.unwrap(), body: Expr::Number(std::f64::NAN) })
    }

    /// Parses any expression.
    fn parse_expr(&mut self) -> Result<Expr, &'static str> {
        match self.parse_primary() {
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
        match self.curr() {
            LParen => (),
            _ => { return Err("Expected '(' character at start of parenthesized expression.") }
        }

        self.advance();

        let expr = self.parse_expr();

        if expr.is_err() {
            return expr;
        }

        match self.curr() {
            RParen => (),
            _ => { return Err("Expected ')' character at end of parenthesized expression.") }
        }

        self.advance();

        Ok(expr.unwrap())
    }

    /// Parses an expression that starts with an identifier (either a variable or a function call).
    fn parse_id_expr(&mut self) -> Result<Expr, &'static str> {
        let id = match self.curr() {
            Ident(id) => id,
            _ => { return Err("Expected identifier."); }
        };

        self.advance();

        match self.curr() {
            LParen => {
                let mut args = vec![];
                
                loop {
                    self.advance();

                    match self.curr() {
                        RParen => break,

                        _ => {
                            match self.parse_expr() {
                                Ok(expr) => args.push(expr),
                                err => { return err; }
                            }

                            self.advance();

                            match self.curr() {
                                Comma => (),
                                _ => { return Err("Expected ',' character in function call."); }
                            }
                        }
                    }
                }

                self.advance();

                Ok(Expr::Call(id, args))
            },

            _ => Ok(Expr::Variable(id))
        }
    }

    /// Parses a binary expression, given its left-hand expression.
    fn parse_binary_expr(&mut self, prec: i32, left: Expr) -> Result<Expr, &'static str> {
        let mut left = left;

        loop {
            let curr_prec = self.get_tok_precedence();

            if curr_prec == -1 || curr_prec < prec {
                return Ok(left);
            }

            let op = match self.curr() {
                Op(op) => op,
                _ => { return Err("Invalid operator."); }
            };

            self.advance();

            let mut right = self.parse_primary();

            if right.is_err() {
                return right;
            }

            let next_prec = self.get_tok_precedence();

            if curr_prec < next_prec {
                right = self.parse_binary_expr(curr_prec + 1, right.unwrap());

                if right.is_err() {
                    return right;
                }
            }
            
            left = Expr::Binary(op, Box::new(left), Box::new(right.unwrap()));
        }
    }

    /// Parses a primary expression (an identifier, a number or a parenthesized expression).
    fn parse_primary(&mut self) -> Result<Expr, &'static str> {
        match self.curr() {
            Ident(_) => self.parse_id_expr(),
            Number(_) => self.parse_nb_expr(),
            LParen => self.parse_paren_expr(),
            _ => Err("Unknown expression.")
        }
    }

    /// Parses a top-level expression and makes an anonymous function out of it,
    /// for easier compilation.
    fn parse_toplevel_expr(&mut self) -> Result<Function, &'static str> {
        match self.parse_expr() {
            Ok(expr) => {
                Ok(Function {
                    prototype: Prototype { name: "anonymous".to_string(), args: vec![] },
                    body: expr
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
pub struct Compiler {
    pub variables: HashMap<String, FloatValue>,
    pub functions: HashMap<String, FunctionValue>,
    pub builder: Builder
}

impl Compiler {
    /// Compiles the specified `Expr` into a LLVM `FloatValue`.
    pub fn compile(&self, expr: &Expr) -> Result<FloatValue, &'static str> {
        match expr {
            &Expr::Number(nb) => Ok(FloatType::f64_type().const_float(nb)),
            &Expr::Variable(ref name) => {
                match self.variables.get(name.as_str()) {
                    Some(var) => Ok(*var),
                    None => Err("Could not find a matching variable.")
                }
            },

            &Expr::Binary(op, ref left, ref right) => {
                let lhs = self.compile(&left)?;
                let rhs = self.compile(&right)?;

                match op {
                    '+' => Ok(self.builder.build_float_add(&lhs, &rhs, "tmp")),
                    '-' => Ok(self.builder.build_float_sub(&lhs, &rhs, "tmp")),
                    '*' => Ok(self.builder.build_float_mul(&lhs, &rhs, "tmp")),
                    _ => Err("Unimplemented operator.")
                }
            },

            &Expr::Call(ref name, ref args) => {
                match self.functions.get(name.as_str()) {
                    Some(fun) => {
                        let args: Vec<FloatValue> = args.iter().map(|expr| self.compile(expr).unwrap()).collect();
                        let mut argsv: Vec<&BasicValue> = args.iter().by_ref().map(|val| val as &BasicValue).collect();

                        match self.builder.build_call(&fun, argsv.as_slice(), "tmp", false).left() {
                            Some(value) => Ok(value.into_float_value()),
                            None => Err("Invalid call produced.")
                        }
                    },
                    None => Err("Unknown function.")
                }
            }
        }
    }
}


// ======================================================================================
// PROGRAM ==============================================================================
// ======================================================================================

/// Entry point of the program; acts as a REPL.
pub fn main() {
    loop {
        println!("> ");

        // Read input from stdin
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Could not read from standard input.");

        // Build precedence map
        let mut prec = HashMap::with_capacity(4);

        prec.insert("<", 10);
        prec.insert("+", 20);
        prec.insert("-", 20);
        prec.insert("*", 40);

        // Parse input
        match Parser::new(input, prec).parse() {
            Ok(expr) => println!("Expression parsed: {:?}", expr),
            Err(err) => println!("Error parsing expression: {}", err)
        }
    }
}
