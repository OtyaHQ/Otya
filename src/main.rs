use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const BYTECODE_HEADER: &str = "OTBC 1";

#[derive(Debug, Clone)]
struct Diagnostic {
    line: usize,
    column: usize,
    message: String,
}

impl Diagnostic {
    fn new(line: usize, column: usize, message: impl Into<String>) -> Self {
        Self { line, column, message: message.into() }
    }

    fn at(token: &Token, message: impl Into<String>) -> Self {
        Self::new(token.line, token.column, message)
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.column, self.message)
    }
}

impl std::error::Error for Diagnostic {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenKind {
    Ident, Number, String, Module, Use, Fn, Let, Mut, Return, True, False,
    LParen, RParen, LBrace, RBrace, Colon, ColonColon, Semicolon, Comma,
    Arrow, Plus, Minus, Star, Slash, Percent, Equal, EqualEqual, Bang,
    BangEqual, Less, LessEqual, Greater, GreaterEqual, Eof,
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    lexeme: String,
    line: usize,
    column: usize,
}

fn lex(source: &str) -> Result<Vec<Token>, Diagnostic> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0;
    let mut line = 1;
    let mut column = 1;

    while index < chars.len() {
        let ch = chars[index];
        if ch.is_whitespace() {
            if ch == '\n' { line += 1; column = 1; } else { column += 1; }
            index += 1;
            continue;
        }
        if ch == '/' && chars.get(index + 1) == Some(&'/') {
            while index < chars.len() && chars[index] != '\n' {
                index += 1;
                column += 1;
            }
            continue;
        }

        let start_line = line;
        let start_column = column;
        if ch.is_ascii_alphabetic() || ch == '_' {
            let start = index;
            while index < chars.len()
                && (chars[index].is_ascii_alphanumeric() || chars[index] == '_')
            {
                index += 1;
                column += 1;
            }
            let text: String = chars[start..index].iter().collect();
            let kind = match text.as_str() {
                "module" => TokenKind::Module,
                "use" => TokenKind::Use,
                "fn" => TokenKind::Fn,
                "let" => TokenKind::Let,
                "mut" => TokenKind::Mut,
                "return" => TokenKind::Return,
                "true" => TokenKind::True,
                "false" => TokenKind::False,
                _ => TokenKind::Ident,
            };
            tokens.push(Token { kind, lexeme: text, line: start_line, column: start_column });
            continue;
        }
        if ch.is_ascii_digit() {
            let start = index;
            while index < chars.len() && chars[index].is_ascii_digit() {
                index += 1;
                column += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Number,
                lexeme: chars[start..index].iter().collect(),
                line: start_line,
                column: start_column,
            });
            continue;
        }
        if ch == '"' {
            index += 1;
            column += 1;
            let mut value = String::new();
            let mut closed = false;
            while index < chars.len() {
                let current = chars[index];
                if current == '"' {
                    index += 1;
                    column += 1;
                    closed = true;
                    break;
                }
                if current == '\n' {
                    return Err(Diagnostic::new(line, column, "newline inside a string literal; use \\n instead"));
                }
                if current == '\\' {
                    index += 1;
                    column += 1;
                    let escaped = chars.get(index).copied().ok_or_else(|| {
                        Diagnostic::new(line, column, "unterminated escape sequence")
                    })?;
                    value.push(match escaped {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '"' => '"',
                        '\\' => '\\',
                        other => return Err(Diagnostic::new(
                            line, column, format!("unsupported escape sequence: \\{}", other)
                        )),
                    });
                    index += 1;
                    column += 1;
                    continue;
                }
                value.push(current);
                index += 1;
                column += 1;
            }
            if !closed {
                return Err(Diagnostic::new(start_line, start_column, "unterminated string literal"));
            }
            tokens.push(Token { kind: TokenKind::String, lexeme: value, line: start_line, column: start_column });
            continue;
        }

        let (kind, width) = match ch {
            '(' => (TokenKind::LParen, 1),
            ')' => (TokenKind::RParen, 1),
            '{' => (TokenKind::LBrace, 1),
            '}' => (TokenKind::RBrace, 1),
            ';' => (TokenKind::Semicolon, 1),
            ',' => (TokenKind::Comma, 1),
            '+' => (TokenKind::Plus, 1),
            '-' if chars.get(index + 1) == Some(&'>') => (TokenKind::Arrow, 2),
            '-' => (TokenKind::Minus, 1),
            '*' => (TokenKind::Star, 1),
            '/' => (TokenKind::Slash, 1),
            '%' => (TokenKind::Percent, 1),
            ':' if chars.get(index + 1) == Some(&':') => (TokenKind::ColonColon, 2),
            ':' => (TokenKind::Colon, 1),
            '=' if chars.get(index + 1) == Some(&'=') => (TokenKind::EqualEqual, 2),
            '=' => (TokenKind::Equal, 1),
            '!' if chars.get(index + 1) == Some(&'=') => (TokenKind::BangEqual, 2),
            '!' => (TokenKind::Bang, 1),
            '<' if chars.get(index + 1) == Some(&'=') => (TokenKind::LessEqual, 2),
            '<' => (TokenKind::Less, 1),
            '>' if chars.get(index + 1) == Some(&'=') => (TokenKind::GreaterEqual, 2),
            '>' => (TokenKind::Greater, 1),
            _ => return Err(Diagnostic::new(line, column, format!("unexpected character: {}", ch))),
        };
        tokens.push(Token {
            kind,
            lexeme: chars[index..index + width].iter().collect(),
            line: start_line,
            column: start_column,
        });
        index += width;
        column += width;
    }

    tokens.push(Token { kind: TokenKind::Eof, lexeme: String::new(), line, column });
    Ok(tokens)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Type { Int, Text, Bool }

impl Type {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "I32" | "I64" | "Int" => Some(Self::Int),
            "Text" | "String" => Some(Self::Text),
            "Bool" => Some(Self::Bool),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
enum Expr {
    Int(i64),
    Text(String),
    Bool(bool),
    Name(String),
    Unary(TokenKind, Box<Expr>),
    Binary(Box<Expr>, TokenKind, Box<Expr>),
}

#[derive(Debug, Clone)]
enum Statement {
    Let { name: String, mutable: bool, declared_type: Option<Type>, value: Expr },
    Assign { name: String, value: Expr },
    Print(Expr),
    Expression(Expr),
    Return(Option<Expr>),
}

struct Parser { tokens: Vec<Token>, current: usize }

impl Parser {
    fn new(tokens: Vec<Token>) -> Self { Self { tokens, current: 0 } }

    fn parse(mut self) -> Result<Vec<Statement>, Diagnostic> {
        while self.matches(TokenKind::Module) || self.matches(TokenKind::Use) {
            while !self.check(TokenKind::Semicolon) && !self.check(TokenKind::Eof) {
                self.advance();
            }
            self.consume(TokenKind::Semicolon, "expected ';' after declaration")?;
        }
        self.consume(TokenKind::Fn, "expected 'fn main()' entry point")?;
        let function = self.consume(TokenKind::Ident, "expected function name")?.clone();
        if function.lexeme != "main" {
            return Err(Diagnostic::at(&function, "the bootstrap compiler currently supports only fn main()"));
        }
        self.consume(TokenKind::LParen, "expected '(' after main")?;
        self.consume(TokenKind::RParen, "the bootstrap compiler currently supports no function parameters")?;
        if self.matches(TokenKind::Arrow) {
            let return_type = self.consume(TokenKind::Ident, "expected return type after '->'")?.clone();
            if return_type.lexeme != "Unit" {
                return Err(Diagnostic::at(&return_type, "main must return Unit in this compiler version"));
            }
        }
        let statements = self.block()?;
        self.consume(TokenKind::Eof, "unexpected content after main function")?;
        Ok(statements)
    }

    fn block(&mut self) -> Result<Vec<Statement>, Diagnostic> {
        self.consume(TokenKind::LBrace, "expected '{' to start function body")?;
        let mut statements = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
            statements.push(self.statement()?);
        }
        self.consume(TokenKind::RBrace, "expected '}' to close function body")?;
        Ok(statements)
    }

    fn statement(&mut self) -> Result<Statement, Diagnostic> {
        if self.matches(TokenKind::Let) {
            let mutable = self.matches(TokenKind::Mut);
            let name = self.consume(TokenKind::Ident, "expected variable name")?.lexeme.clone();
            let declared_type = if self.matches(TokenKind::Colon) {
                let type_token = self.consume(TokenKind::Ident, "expected a type name")?.clone();
                Some(Type::from_name(&type_token.lexeme).ok_or_else(|| Diagnostic::at(
                    &type_token,
                    format!("unsupported type '{}'; supported types are I32, I64, Text and Bool", type_token.lexeme),
                ))?)
            } else { None };
            self.consume(TokenKind::Equal, "expected '=' in variable declaration")?;
            let value = self.expression(0)?;
            self.consume(TokenKind::Semicolon, "expected ';' after variable declaration")?;
            return Ok(Statement::Let { name, mutable, declared_type, value });
        }
        if self.matches(TokenKind::Return) {
            let value = if self.check(TokenKind::Semicolon) { None } else { Some(self.expression(0)?) };
            self.consume(TokenKind::Semicolon, "expected ';' after return")?;
            return Ok(Statement::Return(value));
        }
        if self.check(TokenKind::Ident) && self.peek_next().kind == TokenKind::Equal {
            let name = self.advance().lexeme.clone();
            self.advance();
            let value = self.expression(0)?;
            self.consume(TokenKind::Semicolon, "expected ';' after assignment")?;
            return Ok(Statement::Assign { name, value });
        }
        if self.check(TokenKind::Ident)
            && self.peek().lexeme == "io"
            && self.peek_next().kind == TokenKind::ColonColon
            && self.tokens.get(self.current + 2).map(|t| t.lexeme.as_str()) == Some("println")
        {
            self.advance();
            self.advance();
            self.advance();
            self.consume(TokenKind::LParen, "expected '(' after io::println")?;
            let value = self.expression(0)?;
            self.consume(TokenKind::RParen, "expected ')' after print value")?;
            self.consume(TokenKind::Semicolon, "expected ';' after print statement")?;
            return Ok(Statement::Print(value));
        }
        let expression = self.expression(0)?;
        self.consume(TokenKind::Semicolon, "expected ';' after expression")?;
        Ok(Statement::Expression(expression))
    }

    fn expression(&mut self, minimum_precedence: u8) -> Result<Expr, Diagnostic> {
        let mut left = self.unary()?;
        loop {
            let (precedence, operator) = match self.peek().kind {
                TokenKind::EqualEqual => (1, TokenKind::EqualEqual),
                TokenKind::BangEqual => (1, TokenKind::BangEqual),
                TokenKind::Less => (1, TokenKind::Less),
                TokenKind::LessEqual => (1, TokenKind::LessEqual),
                TokenKind::Greater => (1, TokenKind::Greater),
                TokenKind::GreaterEqual => (1, TokenKind::GreaterEqual),
                TokenKind::Plus => (2, TokenKind::Plus),
                TokenKind::Minus => (2, TokenKind::Minus),
                TokenKind::Star => (3, TokenKind::Star),
                TokenKind::Slash => (3, TokenKind::Slash),
                TokenKind::Percent => (3, TokenKind::Percent),
                _ => break,
            };
            if precedence < minimum_precedence { break; }
            self.advance();
            let right = self.expression(precedence + 1)?;
            left = Expr::Binary(Box::new(left), operator, Box::new(right));
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr, Diagnostic> {
        if self.matches(TokenKind::Minus) {
            return Ok(Expr::Unary(TokenKind::Minus, Box::new(self.unary()?)));
        }
        if self.matches(TokenKind::Bang) {
            return Ok(Expr::Unary(TokenKind::Bang, Box::new(self.unary()?)));
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, Diagnostic> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Number => {
                let value = token.lexeme.parse::<i64>().map_err(|_| Diagnostic::at(
                    &token, "integer literal is outside the supported 64-bit range"
                ))?;
                Ok(Expr::Int(value))
            }
            TokenKind::String => Ok(Expr::Text(token.lexeme)),
            TokenKind::True => Ok(Expr::Bool(true)),
            TokenKind::False => Ok(Expr::Bool(false)),
            TokenKind::Ident => Ok(Expr::Name(token.lexeme)),
            TokenKind::LParen => {
                let expression = self.expression(0)?;
                self.consume(TokenKind::RParen, "expected ')' after expression")?;
                Ok(expression)
            }
            _ => Err(Diagnostic::at(&token, "expected an expression")),
        }
    }

    fn matches(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) { self.advance(); true } else { false }
    }
    fn check(&self, kind: TokenKind) -> bool { self.peek().kind == kind }
    fn consume(&mut self, kind: TokenKind, message: &str) -> Result<&Token, Diagnostic> {
        if self.check(kind) { Ok(self.advance()) } else { Err(Diagnostic::at(self.peek(), message)) }
    }
    fn advance(&mut self) -> &Token {
        if !self.check(TokenKind::Eof) { self.current += 1; }
        &self.tokens[self.current - 1]
    }
    fn peek(&self) -> &Token { &self.tokens[self.current] }
    fn peek_next(&self) -> &Token {
        self.tokens.get(self.current + 1).unwrap_or_else(|| self.tokens.last().expect("lexer always adds EOF"))
    }
}

#[derive(Debug, Clone, Copy)]
struct Variable { slot: u16, ty: Type, mutable: bool }

#[derive(Debug, Clone)]
enum Instruction {
    PushInt(i64), PushText(String), PushBool(bool), Load(u16), Store(u16),
    Add, Subtract, Multiply, Divide, Remainder, Negate, Not,
    Equal, NotEqual, Less, LessEqual, Greater, GreaterEqual,
    Print, Pop, Return, Halt,
}

#[derive(Debug, Clone)]
struct Bytecode { local_count: u16, instructions: Vec<Instruction> }

struct Compiler {
    variables: HashMap<String, Variable>,
    next_slot: u16,
    instructions: Vec<Instruction>,
}

impl Compiler {
    fn new() -> Self {
        Self { variables: HashMap::new(), next_slot: 0, instructions: Vec::new() }
    }

    fn compile(mut self, statements: Vec<Statement>) -> Result<Bytecode, Diagnostic> {
        for statement in statements { self.statement(statement)?; }
        self.instructions.push(Instruction::Halt);
        Ok(Bytecode { local_count: self.next_slot, instructions: self.instructions })
    }

    fn statement(&mut self, statement: Statement) -> Result<(), Diagnostic> {
        match statement {
            Statement::Let { name, mutable, declared_type, value } => {
                if self.variables.contains_key(&name) {
                    return Err(Diagnostic::new(1, 1, format!("variable '{}' is already declared in main", name)));
                }
                let actual_type = self.expression(&value)?;
                if let Some(expected) = declared_type {
                    if expected != actual_type {
                        return Err(Diagnostic::new(
                            1, 1, format!("variable '{}' expects {:?}, but its value has type {:?}", name, expected, actual_type)
                        ));
                    }
                }
                let slot = self.next_slot;
                self.next_slot = self.next_slot.checked_add(1)
                    .ok_or_else(|| Diagnostic::new(1, 1, "too many local variables"))?;
                self.variables.insert(name, Variable { slot, ty: actual_type, mutable });
                self.instructions.push(Instruction::Store(slot));
            }
            Statement::Assign { name, value } => {
                let variable = *self.variables.get(&name)
                    .ok_or_else(|| Diagnostic::new(1, 1, format!("unknown variable '{}'", name)))?;
                if !variable.mutable {
                    return Err(Diagnostic::new(
                        1, 1, format!("cannot assign to immutable variable '{}'; declare it with 'let mut'", name)
                    ));
                }
                let actual_type = self.expression(&value)?;
                if variable.ty != actual_type {
                    return Err(Diagnostic::new(
                        1, 1, format!("assignment to '{}' expects {:?}, but received {:?}", name, variable.ty, actual_type)
                    ));
                }
                self.instructions.push(Instruction::Store(variable.slot));
            }
            Statement::Print(value) => {
                self.expression(&value)?;
                self.instructions.push(Instruction::Print);
            }
            Statement::Expression(value) => {
                self.expression(&value)?;
                self.instructions.push(Instruction::Pop);
            }
            Statement::Return(value) => {
                if let Some(expression) = value {
                    self.expression(&expression)?;
                    self.instructions.push(Instruction::Pop);
                }
                self.instructions.push(Instruction::Return);
            }
        }
        Ok(())
    }

    fn expression(&mut self, expression: &Expr) -> Result<Type, Diagnostic> {
        match expression {
            Expr::Int(value) => { self.instructions.push(Instruction::PushInt(*value)); Ok(Type::Int) }
            Expr::Text(value) => { self.instructions.push(Instruction::PushText(value.clone())); Ok(Type::Text) }
            Expr::Bool(value) => { self.instructions.push(Instruction::PushBool(*value)); Ok(Type::Bool) }
            Expr::Name(name) => {
                let variable = *self.variables.get(name)
                    .ok_or_else(|| Diagnostic::new(1, 1, format!("unknown variable '{}'", name)))?;
                self.instructions.push(Instruction::Load(variable.slot));
                Ok(variable.ty)
            }
            Expr::Unary(operator, value) => {
                let ty = self.expression(value)?;
                match operator {
                    TokenKind::Minus if ty == Type::Int => {
                        self.instructions.push(Instruction::Negate);
                        Ok(Type::Int)
                    }
                    TokenKind::Bang if ty == Type::Bool => {
                        self.instructions.push(Instruction::Not);
                        Ok(Type::Bool)
                    }
                    TokenKind::Minus => Err(Diagnostic::new(1, 1, "unary '-' requires an integer")),
                    TokenKind::Bang => Err(Diagnostic::new(1, 1, "unary '!' requires a boolean")),
                    _ => Err(Diagnostic::new(1, 1, "unsupported unary operator")),
                }
            }
            Expr::Binary(left, operator, right) => {
                let left_type = self.expression(left)?;
                let right_type = self.expression(right)?;
                let instruction = match operator {
                    TokenKind::Plus if left_type == Type::Int && right_type == Type::Int => Instruction::Add,
                    TokenKind::Minus if left_type == Type::Int && right_type == Type::Int => Instruction::Subtract,
                    TokenKind::Star if left_type == Type::Int && right_type == Type::Int => Instruction::Multiply,
                    TokenKind::Slash if left_type == Type::Int && right_type == Type::Int => Instruction::Divide,
                    TokenKind::Percent if left_type == Type::Int && right_type == Type::Int => Instruction::Remainder,
                    TokenKind::EqualEqual if left_type == right_type => Instruction::Equal,
                    TokenKind::BangEqual if left_type == right_type => Instruction::NotEqual,
                    TokenKind::Less if left_type == Type::Int && right_type == Type::Int => Instruction::Less,
                    TokenKind::LessEqual if left_type == Type::Int && right_type == Type::Int => Instruction::LessEqual,
                    TokenKind::Greater if left_type == Type::Int && right_type == Type::Int => Instruction::Greater,
                    TokenKind::GreaterEqual if left_type == Type::Int && right_type == Type::Int => Instruction::GreaterEqual,
                    _ => return Err(Diagnostic::new(
                        1, 1, format!("operator '{}' does not support operands {:?} and {:?}", operator_name(*operator), left_type, right_type)
                    )),
                };
                let result_type = match operator {
                    TokenKind::EqualEqual | TokenKind::BangEqual | TokenKind::Less
                    | TokenKind::LessEqual | TokenKind::Greater | TokenKind::GreaterEqual => Type::Bool,
                    _ => Type::Int,
                };
                self.instructions.push(instruction);
                Ok(result_type)
            }
        }
    }
}

fn operator_name(kind: TokenKind) -> &'static str {
    match kind {
        TokenKind::Plus => "+", TokenKind::Minus => "-", TokenKind::Star => "*",
        TokenKind::Slash => "/", TokenKind::Percent => "%", TokenKind::EqualEqual => "==",
        TokenKind::BangEqual => "!=", TokenKind::Less => "<", TokenKind::LessEqual => "<=",
        TokenKind::Greater => ">", TokenKind::GreaterEqual => ">=", _ => "?",
    }
}

fn compile_source(source: &str) -> Result<Bytecode, Diagnostic> {
    Compiler::new().compile(Parser::new(lex(source)?).parse()?)
}

impl Bytecode {
    fn serialize(&self) -> String {
        let mut output = format!("{}\nLOCALS {}\n", BYTECODE_HEADER, self.local_count);
        for instruction in &self.instructions {
            let line = match instruction {
                Instruction::PushInt(v) => format!("PUSH_INT {}", v),
                Instruction::PushText(v) => format!("PUSH_TEXT {}", encode_hex(v.as_bytes())),
                Instruction::PushBool(v) => format!("PUSH_BOOL {}", if *v { 1 } else { 0 }),
                Instruction::Load(v) => format!("LOAD {}", v),
                Instruction::Store(v) => format!("STORE {}", v),
                Instruction::Add => "ADD".to_string(),
                Instruction::Subtract => "SUB".to_string(),
                Instruction::Multiply => "MUL".to_string(),
                Instruction::Divide => "DIV".to_string(),
                Instruction::Remainder => "REM".to_string(),
                Instruction::Negate => "NEG".to_string(),
                Instruction::Not => "NOT".to_string(),
                Instruction::Equal => "EQ".to_string(),
                Instruction::NotEqual => "NE".to_string(),
                Instruction::Less => "LT".to_string(),
                Instruction::LessEqual => "LE".to_string(),
                Instruction::Greater => "GT".to_string(),
                Instruction::GreaterEqual => "GE".to_string(),
                Instruction::Print => "PRINT".to_string(),
                Instruction::Pop => "POP".to_string(),
                Instruction::Return => "RETURN".to_string(),
                Instruction::Halt => "HALT".to_string(),
            };
            output.push_str(&line);
            output.push('\n');
        }
        output
    }

    fn deserialize(source: &str) -> Result<Self, String> {
        let mut lines = source.lines();
        if lines.next() != Some(BYTECODE_HEADER) {
            return Err("invalid bytecode header; expected OTBC 1".to_string());
        }
        let local_line = lines.next().ok_or_else(|| "missing LOCALS header".to_string())?;
        let local_count = local_line.strip_prefix("LOCALS ")
            .ok_or_else(|| "invalid LOCALS header".to_string())?
            .parse::<u16>().map_err(|_| "invalid local variable count".to_string())?;
        let mut instructions = Vec::new();
        for (index, line) in lines.enumerate() {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            let opcode = parts[0];
            let argument = parts.get(1).copied().unwrap_or("");
            let bytecode_line = index + 3;
            let instruction = match opcode {
                "PUSH_INT" => Instruction::PushInt(argument.parse().map_err(|_| format!("invalid integer at bytecode line {}", bytecode_line))?),
                "PUSH_TEXT" => Instruction::PushText(String::from_utf8(decode_hex(argument)
                    .map_err(|error| format!("invalid text at bytecode line {}: {}", bytecode_line, error))?)
                    .map_err(|_| format!("text at bytecode line {} is not UTF-8", bytecode_line))?),
                "PUSH_BOOL" => match argument {
                    "0" => Instruction::PushBool(false),
                    "1" => Instruction::PushBool(true),
                    _ => return Err(format!("invalid boolean at bytecode line {}", bytecode_line)),
                },
                "LOAD" => Instruction::Load(argument.parse().map_err(|_| format!("invalid LOAD at bytecode line {}", bytecode_line))?),
                "STORE" => Instruction::Store(argument.parse().map_err(|_| format!("invalid STORE at bytecode line {}", bytecode_line))?),
                "ADD" => Instruction::Add, "SUB" => Instruction::Subtract,
                "MUL" => Instruction::Multiply, "DIV" => Instruction::Divide,
                "REM" => Instruction::Remainder, "NEG" => Instruction::Negate,
                "NOT" => Instruction::Not, "EQ" => Instruction::Equal,
                "NE" => Instruction::NotEqual, "LT" => Instruction::Less,
                "LE" => Instruction::LessEqual, "GT" => Instruction::Greater,
                "GE" => Instruction::GreaterEqual, "PRINT" => Instruction::Print,
                "POP" => Instruction::Pop, "RETURN" => Instruction::Return,
                "HALT" => Instruction::Halt,
                _ => return Err(format!("unknown opcode '{}' at bytecode line {}", opcode, bytecode_line)),
            };
            instructions.push(instruction);
        }
        if !matches!(instructions.last(), Some(Instruction::Halt) | Some(Instruction::Return)) {
            return Err("bytecode must end with HALT or RETURN".to_string());
        }
        Ok(Self { local_count, instructions })
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    if value.len() % 2 != 0 { return Err("hex text must have an even number of characters".to_string()); }
    let mut output = Vec::with_capacity(value.len() / 2);
    for pair in value.as_bytes().chunks_exact(2) {
        let high = (pair[0] as char).to_digit(16).ok_or_else(|| "non-hexadecimal character".to_string())?;
        let low = (pair[1] as char).to_digit(16).ok_or_else(|| "non-hexadecimal character".to_string())?;
        output.push(((high << 4) | low) as u8);
    }
    Ok(output)
}

#[derive(Debug, Clone, PartialEq)]
enum Value { Int(i64), Text(String), Bool(bool) }

fn pop_value(stack: &mut Vec<Value>) -> Result<Value, String> {
    stack.pop().ok_or_else(|| "runtime stack underflow".to_string())
}
fn pop_int(stack: &mut Vec<Value>) -> Result<i64, String> {
    match pop_value(stack)? {
        Value::Int(value) => Ok(value),
        _ => Err("runtime type mismatch: expected integer".to_string()),
    }
}

fn execute<W: Write>(bytecode: &Bytecode, output: &mut W) -> Result<(), String> {
    let mut stack: Vec<Value> = Vec::new();
    let mut locals: Vec<Option<Value>> = vec![None; bytecode.local_count as usize];
    for instruction in &bytecode.instructions {
        match instruction {
            Instruction::PushInt(value) => stack.push(Value::Int(*value)),
            Instruction::PushText(value) => stack.push(Value::Text(value.clone())),
            Instruction::PushBool(value) => stack.push(Value::Bool(*value)),
            Instruction::Load(slot) => {
                let value = locals.get(*slot as usize).and_then(Option::as_ref).cloned()
                    .ok_or_else(|| format!("read of uninitialized or invalid local slot {}", slot))?;
                stack.push(value);
            }
            Instruction::Store(slot) => {
                let value = pop_value(&mut stack)?;
                let destination = locals.get_mut(*slot as usize)
                    .ok_or_else(|| format!("invalid local slot {}", slot))?;
                *destination = Some(value);
            }
            Instruction::Add => {
                let r = pop_int(&mut stack)?; let l = pop_int(&mut stack)?;
                stack.push(Value::Int(l.checked_add(r).ok_or_else(|| "integer overflow in '+'".to_string())?));
            }
            Instruction::Subtract => {
                let r = pop_int(&mut stack)?; let l = pop_int(&mut stack)?;
                stack.push(Value::Int(l.checked_sub(r).ok_or_else(|| "integer overflow in '-'".to_string())?));
            }
            Instruction::Multiply => {
                let r = pop_int(&mut stack)?; let l = pop_int(&mut stack)?;
                stack.push(Value::Int(l.checked_mul(r).ok_or_else(|| "integer overflow in '*'".to_string())?));
            }
            Instruction::Divide => {
                let r = pop_int(&mut stack)?; let l = pop_int(&mut stack)?;
                let result = l.checked_div(r).ok_or_else(|| {
                    if r == 0 { "division by zero".to_string() } else { "integer overflow in '/'".to_string() }
                })?;
                stack.push(Value::Int(result));
            }
            Instruction::Remainder => {
                let r = pop_int(&mut stack)?; let l = pop_int(&mut stack)?;
                let result = l.checked_rem(r).ok_or_else(|| {
                    if r == 0 { "remainder by zero".to_string() } else { "integer overflow in '%'".to_string() }
                })?;
                stack.push(Value::Int(result));
            }
            Instruction::Negate => {
                let value = pop_int(&mut stack)?;
                stack.push(Value::Int(value.checked_neg().ok_or_else(|| "integer overflow in unary '-'".to_string())?));
            }
            Instruction::Not => match pop_value(&mut stack)? {
                Value::Bool(value) => stack.push(Value::Bool(!value)),
                _ => return Err("runtime type mismatch: expected boolean".to_string()),
            },
            Instruction::Equal | Instruction::NotEqual => {
                let right = pop_value(&mut stack)?;
                let left = pop_value(&mut stack)?;
                let equal = left == right;
                stack.push(Value::Bool(if matches!(instruction, Instruction::Equal) { equal } else { !equal }));
            }
            Instruction::Less | Instruction::LessEqual | Instruction::Greater | Instruction::GreaterEqual => {
                let right = pop_int(&mut stack)?;
                let left = pop_int(&mut stack)?;
                let result = match instruction {
                    Instruction::Less => left < right,
                    Instruction::LessEqual => left <= right,
                    Instruction::Greater => left > right,
                    Instruction::GreaterEqual => left >= right,
                    _ => unreachable!(),
                };
                stack.push(Value::Bool(result));
            }
            Instruction::Print => {
                match pop_value(&mut stack)? {
                    Value::Int(value) => writeln!(output, "{}", value),
                    Value::Text(value) => writeln!(output, "{}", value),
                    Value::Bool(value) => writeln!(output, "{}", value),
                }.map_err(|error| format!("output error: {}", error))?;
            }
            Instruction::Pop => { pop_value(&mut stack)?; }
            Instruction::Return | Instruction::Halt => return Ok(()),
        }
    }
    Ok(())
}

fn resolve_source(input: &Path) -> Result<PathBuf, String> {
    if input.is_dir() {
        let candidate = input.join("src").join("main.ot");
        if candidate.is_file() { return Ok(candidate); }
        return Err(format!("no src/main.ot found in project directory '{}'", input.display()));
    }
    if !input.is_file() { return Err(format!("source path '{}' does not exist", input.display())); }
    Ok(input.to_path_buf())
}

fn output_path(source: &Path) -> PathBuf {
    let parent = source.parent().unwrap_or_else(|| Path::new("."));
    let root = if parent.file_name().and_then(|name| name.to_str()) == Some("src") {
        parent.parent().unwrap_or(parent)
    } else { parent };
    let stem = source.file_stem().and_then(|name| name.to_str()).unwrap_or("main");
    root.join(".otya").join("build").join(format!("{}.otbc", stem))
}

fn load_bytecode(path: &Path) -> Result<Bytecode, String> {
    let content = fs::read_to_string(path).map_err(|error| format!("cannot read '{}': {}", path.display(), error))?;
    Bytecode::deserialize(&content)
}

fn create_project(name: &str) -> Result<(), String> {
    if name.is_empty() || !name.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_') {
        return Err("project name must contain only letters, numbers, '-' or '_'".to_string());
    }
    let root = PathBuf::from(name);
    if root.exists() { return Err(format!("path '{}' already exists", root.display())); }
    fs::create_dir_all(root.join("src")).map_err(|error| format!("cannot create project: {}", error))?;
    let manifest = format!(
        "format = 1\n\n[package]\nname = \"{}\"\nversion = \"0.1.0\"\nlanguage = \"0.1\"\nsource = \"src/main.ot\"\n",
        name
    );
    fs::write(root.join("otya.toml"), manifest).map_err(|error| format!("cannot write otya.toml: {}", error))?;
    fs::write(
        root.join("src").join("main.ot"),
        "module main;\n\nuse std::io;\n\nfn main() {\n    io::println(\"Hello from Otya!\");\n}\n",
    ).map_err(|error| format!("cannot write src/main.ot: {}", error))?;
    println!("Created Otya project '{}'.", name);
    println!("Next: cd {} && otya check", name);
    Ok(())
}

fn print_usage() {
    println!(
        "Otya {} — bootstrap compiler\n\nUSAGE:\n  otya new <name>\n  otya check [file.ot|project-directory]\n  otya build [file.ot|project-directory]\n  otya run [file.ot|file.otbc|project-directory]\n  otya doctor\n  otya --version\n\nThe current compiler supports one fn main(), variables, integer arithmetic,\nbooleans, text, comparisons, and io::println. Unsupported syntax is rejected.",
        VERSION
    );
}

fn run_cli() -> Result<(), String> {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if arguments.is_empty() || arguments[0] == "help" || arguments[0] == "--help" {
        print_usage(); return Ok(());
    }
    if arguments[0] == "--version" || arguments[0] == "-V" {
        println!("otya {}", VERSION); return Ok(());
    }
    match arguments[0].as_str() {
        "new" => {
            if arguments.len() != 2 { return Err("usage: otya new <name>".to_string()); }
            create_project(&arguments[1])
        }
        "doctor" => {
            if arguments.len() != 1 { return Err("usage: otya doctor".to_string()); }
            println!("Otya CLI: {}", VERSION);
            println!("Host: {}-{}", env::consts::OS, env::consts::ARCH);
            println!("Compiler: Rust bootstrap");
            println!("Runtime: Otya bytecode VM");
            println!("System engine integration: not installed in this prototype");
            Ok(())
        }
        "check" | "build" | "run" => {
            if arguments.len() > 2 { return Err(format!("too many arguments for '{}'", arguments[0])); }
            let input = PathBuf::from(arguments.get(1).map(String::as_str).unwrap_or("."));
            if arguments[0] == "run" && input.extension().and_then(|ext| ext.to_str()) == Some("otbc") {
                let bytecode = load_bytecode(&input)?;
                return execute(&bytecode, &mut io::stdout().lock());
            }
            let source_path = resolve_source(&input)?;
            let source = fs::read_to_string(&source_path)
                .map_err(|error| format!("cannot read '{}': {}", source_path.display(), error))?;
            let bytecode = compile_source(&source).map_err(|error| {
                format!("{}:{}: {}", source_path.display(), error.line, error)
            })?;
            match arguments[0].as_str() {
                "check" => { println!("Checked {} successfully.", source_path.display()); Ok(()) }
                "build" => {
                    let destination = output_path(&source_path);
                    if let Some(parent) = destination.parent() {
                        fs::create_dir_all(parent).map_err(|error| format!("cannot create build directory: {}", error))?;
                    }
                    fs::write(&destination, bytecode.serialize()).map_err(|error| format!("cannot write bytecode: {}", error))?;
                    println!("Built {} -> {}", source_path.display(), destination.display());
                    Ok(())
                }
                "run" => execute(&bytecode, &mut io::stdout().lock()),
                _ => unreachable!(),
            }
        }
        other => Err(format!("unknown command '{}'; run 'otya --help'", other)),
    }
}

fn main() {
    if let Err(error) = run_cli() {
        eprintln!("otya: {}", error);
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_source(source: &str) -> Result<String, String> {
        let bytecode = compile_source(source).map_err(|error| error.to_string())?;
        let mut output = Vec::new();
        execute(&bytecode, &mut output)?;
        String::from_utf8(output).map_err(|error| error.to_string())
    }

    #[test]
    fn compiles_and_runs_hello_world() {
        let source = r#"
            module hello;
            use std::io;
            fn main() { io::println("Hello from Otya!"); }
        "#;
        assert_eq!(run_source(source).unwrap(), "Hello from Otya!\n");
    }

    #[test]
    fn compiles_variables_and_integer_arithmetic() {
        let source = r#"
            use std::io;
            fn main() {
                let mut total: I32 = 10;
                total = total + 5 * 2;
                io::println(total);
            }
        "#;
        assert_eq!(run_source(source).unwrap(), "20\n");
    }

    #[test]
    fn supports_boolean_comparisons() {
        let source = r#"
            use std::io;
            fn main() {
                let passed: Bool = 4 >= 3;
                io::println(passed);
            }
        "#;
        assert_eq!(run_source(source).unwrap(), "true\n");
    }

    #[test]
    fn rejects_assignment_to_immutable_binding() {
        let error = compile_source("fn main() { let answer = 42; answer = 7; }")
            .unwrap_err().to_string();
        assert!(error.contains("immutable"));
    }

    #[test]
    fn rejects_unknown_variables() {
        let error = compile_source("fn main() { io::println(missing); }")
            .unwrap_err().to_string();
        assert!(error.contains("unknown variable"));
    }

    #[test]
    fn rejects_mixed_type_arithmetic() {
        let error = compile_source(r#"fn main() { let bad = "text" + 1; }"#)
            .unwrap_err().to_string();
        assert!(error.contains("does not support operands"));
    }

    #[test]
    fn catches_division_by_zero_at_runtime() {
        let bytecode = compile_source("use std::io; fn main() { io::println(10 / 0); }").unwrap();
        let error = execute(&bytecode, &mut Vec::new()).unwrap_err();
        assert!(error.contains("division by zero"));
    }

    #[test]
    fn bytecode_round_trip_preserves_program() {
        let bytecode = compile_source(r#"use std::io; fn main() { io::println("round trip"); }"#).unwrap();
        let decoded = Bytecode::deserialize(&bytecode.serialize()).unwrap();
        let mut output = Vec::new();
        execute(&decoded, &mut output).unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), "round trip\n");
    }

    #[test]
    fn rejects_unterminated_strings() {
        let error = compile_source(r#"fn main() { io::println("unfinished); }"#)
            .unwrap_err().to_string();
        assert!(error.contains("unterminated string"));
    }

    #[test]
    fn rejects_unsupported_entry_points() {
        let error = compile_source("fn start() {}").unwrap_err().to_string();
        assert!(error.contains("only fn main()"));
    }
}
