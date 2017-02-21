use std::str::{self, FromStr};
use std::collections::VecDeque;

use pest::prelude::*;

#[derive(Debug, PartialEq, Clone)]
pub struct Program(Vec<Statement>);

impl IntoIterator for Program {
    type Item = Statement;
    type IntoIter = ::std::vec::IntoIter<Statement>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromStr for Program {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parser = Rdp::new(StringInput::new(input));

        println!("{:?}", (parser.program(), parser.end()));
        println!("{:#?}", parser.queue());
        println!("{:?}", parser.stack());
        let (expected, pos) = parser.expected();
        let (line, col) = parser.input().line_col(pos);
        println!("Expected: {:?} at line {} col {}", expected, line, col);
        unimplemented!();
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Comment(String),
    Declaration {
        pattern: Pattern,
        type_def: TypeDefinition,
        expr: Option<Expression>,
    },
    Assignment {
        lhs: Identifier,
        expr: Expression,
    },
    WhileLoop {
        condition: Expression,
        body: Block,
    },
    ConditionGroup {
        // Condition expression and body block
        branches: Vec<(Expression, Block)>,
        // "default" or "else" branch body
        default: Option<Block>,
    },
    Expression {
        expr: Expression,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Pattern {
    Identifier(Identifier),
}

#[derive(Debug, PartialEq, Clone)]
pub enum TypeDefinition {
    Name {
        name: Identifier,
    },
    Array {
        type_def: Box<TypeDefinition>,
        //TODO: This probably isn't the right type since this should accept anything and then get
        // statically checked to ensure the correct number was put here
        size: Option<Number>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    StringLiteral(String),
    Identifier(Identifier),
}

pub type Identifier = String;
pub type Block = Vec<Statement>;
pub type Number = i32;

impl_rdp! {
    grammar! {
        program = _{ statement* ~ eoi }

        statement = _{ declaration | assignment | while_loop | for_loop | conditional | (expr ~ semi) | comment }

        comment = @{ block_comment | line_comment }
        line_comment = _{ ["//"] ~ (!(["\r"] | ["\n"]) ~ any)* ~ (["\n"] | ["\r\n"] | ["\r"] | eoi) }
        block_comment = _{ ["/*"] ~ ((!(["*/"]) ~ any) | block_comment)* ~ ["*/"] }

        assignment = { identifier ~ ["="] ~ expr ~ semi}
        declaration = { ["let"] ~ pattern ~ [":"] ~ type_def ~ (["="] ~ expr)? ~ semi}
        pattern = { identifier }

        type_def = _{ identifier | array_type }
        array_type = { ["["] ~ type_def ~ semi ~ array_size ~ ["]"] }
        array_size = _{ unspecified | number }
        unspecified = { ["_"] }

        while_loop = { ["while"] ~ expr ~ block }
        for_loop = { ["for"] ~ pattern ~ ["in"] ~ expr ~ block }

        expr = {
            { block | group | constant | func_call | call_chain | conditional | bool_not | string_literal | range | number }

            // Ordered from lowest precedence to highest precedence
            bool_or = {< ["||"] }
            bool_and = {< ["&&"] }
            // NOTE: Order matters! { ["<"] | ["<="] } will never match "<="
            comparison = { ["=="] | ["!="] | [">="] | ["<="] | [">"] | ["<"] }
            concatenation = {< ["++"] }
            term = { ["+"] | ["-"] }
            factor = { ["/"] | ["*"] | ["%"] }
            pow = { ["**"] }
        }

        conditional = { ["if"] ~ expr ~ block ~ (["else"] ~ conditional)? ~ (["else"] ~ block)? }

        // This allows {} and {statement; statement; statement;} and {statement; expr} and {expr}
        block = { ["{"] ~ statement* ~ expr? ~ ["}"] }

        group = { ["("] ~ expr ~ [")"] }
        bool_not = { ["!"] ~ expr }
        range = { number ~ (comma ~ number)? ~ [".."] ~ number }

        func_call = { identifier ~ func_args }

        call_chain = { identifier ~ prop_get_call* }
        prop_get_call = { ["."] ~ identifier ~ func_args? }

        // This allows () and (func_arg, func_arg) and (func_arg) and (func_arg,)
        func_args = { ["("] ~ (func_arg ~ comma)* ~ func_arg? ~ [")"] }
        func_arg = { expr }

        string_literal = { ["\""] ~ literal_char* ~ ["\""] }
        literal_char = _{ escape_sequence | (!["\""] ~ any) }
        escape_sequence = _{ ["\\\\"] | ["\\\""] | ["\\\'"] | ["\\n"] | ["\\r"] | ["\\t"] | ["\\0"] | ["\\f"] | ["\\v"] | ["\\e"] }

        identifier = @{ !keyword ~ (alpha | ["_"]) ~ (alphanumeric | ["_"])* }
        alpha = _{ ['a'..'z'] | ['A'..'Z'] }
        alphanumeric = _{ alpha | ['0'..'9'] }

        number = @{ (["-"] | ["+"])? ~ positive_integer }
        positive_integer = _{ ["0"] | (nonzero ~ digit*) }
        // Allow "_" in numbers for grouping: 1_000_000 == 1000000
        digit = _{ ["0"] | nonzero | ["_"] }
        nonzero = _{ ['1'..'9'] }

        constant = @{ ["true"] | ["false"] }

        whitespace = _{ [" "] | ["\t"] | ["\u{000C}"] | ["\r"] | ["\n"] }
        // NOTE: When changing this code, make sure you don't have a subset of a word before
        // another word. For example: { ["type"] | ["typeof"] } will never match "typeof"
        keyword = @{
            ["abstract"] | ["as"] | ["become"] | ["break"] | ["byte"] | ["class"] | ["clear"] |
            ["const"] | ["continue"] | ["do"] | ["else"] | ["enum"] | ["eval"] | ["export"] |
            ["extern"] | ["false"] | ["final"] | ["fn"] | ["for"] | ["if"] | ["impl"] | ["import"] |
            ["in"] | ["let"] | ["loop"] | ["match"] | ["mod"] | ["move"] | ["mut"] | ["of"] |
            ["out"] | ["pub"] | ["raw"] | ["ref"] | ["return"] | ["self"] | ["static"] |
            ["struct"] | ["super"] | ["trait"] | ["true"] | ["typeof"] | ["type"] | ["unsafe"] |
            ["use"] | ["where"] | ["while"] | ["yield"]
        }

        // These are separate rules because we can use the generated rules and tokens to provide
        // better error messages
        comma = { [","] }
        semi = { [";"] }
    }

    process! {
        parse_program(&self) -> Program {
            (list: _program()) => {
                Program(list.into_iter().collect())
            },
        }

        _program(&self) -> VecDeque<Statement> {
            (head: _statement(), mut tail: _program()) => {
                tail.push_front(head);

                tail
            },
            () => {
                VecDeque::new()
            },
        }

        _statement(&self) -> Statement {
            (&text: comment) => Statement::Comment(text.to_owned()),
            (_: declaration, pattern: _pattern(), type_def: _type_def(), expr: _optional_expr(), _: semi) => {
                Statement::Declaration {pattern: pattern, type_def: type_def, expr: expr}
            },
        }

        _pattern(&self) -> Pattern {
            (_: pattern, ident: _identifier()) => {
                Pattern::Identifier(ident)
            },
        }

        _type_def(&self) -> TypeDefinition {
            (ident: _identifier()) => {
                TypeDefinition::Name {name: ident}
            },
            (_: array_type, type_def: _type_def(), _: semi, size: _array_size()) => {
                TypeDefinition::Array {type_def: Box::new(type_def), size: size}
            },
        }

        _array_size(&self) -> Option<Number> {
            (_: unspecified) => {
                None
            },
            (n: _number()) => {
                Some(n)
            }
        }

        _optional_expr(&self) -> Option<Expression> {
            (e: _expr()) => Some(e),
            () => None,
        }

        _expr(&self) -> Expression {
            () => unimplemented!(),
        }

        _identifier(&self) -> Identifier {
            (&ident: identifier) => {
                ident.into()
            }
        }

        _number(&self) -> Number {
            (&s: number) => {
                // If our grammar is correct, we are guarenteed that this will work
                s.parse().unwrap()
            }
        }
    }
}
