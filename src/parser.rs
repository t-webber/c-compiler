#![allow(unused)]
use std::{
    fmt::Debug,
    ops::{AddAssign, MulAssign, SubAssign},
};

#[derive(Debug)]
pub enum Keyword {
    Auto,
    Double,
    Int,
    Struct,
    Break,
    Else,
    Long,
    Switch,
    Case,
    Enum,
    Register,
    Typedef,
    Char,
    Extern,
    Return,
    Union,
    Const,
    Float,
    Short,
    Unsigned,
    Continue,
    For,
    Signed,
    Void,
    Default,
    Goto,
    Sizeof,
    Volatile,
    Do,
    If,
    Static,
    While,
}

#[derive(Debug, Clone)]
pub enum Operator {
    // Unary
    Plus,
    Minus,
    Not,

    // Binary
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    And,
    Or,
    Xor,
    ShiftLeft,
    ShiftRight,
    Increment,
    Decrement,
    NotEqual,
    Eequal,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    OrAssign,
    AndAssign,
    XorAssign,
    Arrow,

    ShiftLeftAssign,
    ShiftRightAssign,
}

#[derive(Debug)]
pub enum Token {
    Keyword(Keyword),
    Operator(Operator),
    Identifier,
    Constant,
    String,
    SpecialSymbol,
}

#[derive(Debug, Clone)]
pub enum Bracing {
    LeftParenthesis,
    RightParenthesis,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
}

#[derive(Debug, Clone)]
pub enum PreprocessorToken {
    DefinedOperator,
    Operator(Operator),
    Bracing(Bracing),
    LiteralString(String),
    LiteralNumber(f32),
    Macro(String),
}

fn is_not_operator(c: char) -> bool {
    const OPERATORS: [char; 18] = [
        ' ', '!', '+', '-', '*', '/', '%', '&', '|', '^', '<', '>', '(', ')', '{', '}', '[', ']',
    ];
    !OPERATORS.contains(&c)
}

fn token_from_str(token_str: &str) -> Option<PreprocessorToken> {
    if token_str.is_empty() {
        return None;
    }
    let no_operator = token_str.chars().all(is_not_operator);

    let token = if let Ok(number) = token_str.parse::<f32>() {
        if no_operator {
            PreprocessorToken::LiteralNumber(number)
        } else {
            return None;
        }
    } else {
        match token_str {
            "!" => PreprocessorToken::Operator(Operator::Not),
            "+" => PreprocessorToken::Operator(Operator::Add),
            "-" => PreprocessorToken::Operator(Operator::Sub),
            "*" => PreprocessorToken::Operator(Operator::Mul),
            "/" => PreprocessorToken::Operator(Operator::Div),
            "%" => PreprocessorToken::Operator(Operator::Mod),
            "&" => PreprocessorToken::Operator(Operator::BitwiseAnd),
            "|" => PreprocessorToken::Operator(Operator::BitwiseOr),
            "^" => PreprocessorToken::Operator(Operator::BitwiseXor),
            "<" => PreprocessorToken::Operator(Operator::LessThan),
            ">" => PreprocessorToken::Operator(Operator::GreaterThan),
            "++" => PreprocessorToken::Operator(Operator::Increment),
            "--" => PreprocessorToken::Operator(Operator::Decrement),
            ">>" => PreprocessorToken::Operator(Operator::ShiftRight),
            "<<" => PreprocessorToken::Operator(Operator::ShiftLeft),
            "!=" => PreprocessorToken::Operator(Operator::NotEqual),
            "==" => PreprocessorToken::Operator(Operator::Eequal),
            "+=" => PreprocessorToken::Operator(Operator::AddAssign),
            "-=" => PreprocessorToken::Operator(Operator::SubAssign),
            "*=" => PreprocessorToken::Operator(Operator::MulAssign),
            "/=" => PreprocessorToken::Operator(Operator::DivAssign),
            "%=" => PreprocessorToken::Operator(Operator::ModAssign),
            "<=" => PreprocessorToken::Operator(Operator::LessEqual),
            ">=" => PreprocessorToken::Operator(Operator::GreaterEqual),
            "&=" => PreprocessorToken::Operator(Operator::AndAssign),
            "|=" => PreprocessorToken::Operator(Operator::OrAssign),
            "^=" => PreprocessorToken::Operator(Operator::XorAssign),
            "&&" => PreprocessorToken::Operator(Operator::And),
            "||" => PreprocessorToken::Operator(Operator::Or),
            "->" => PreprocessorToken::Operator(Operator::Arrow),
            ">>=" => PreprocessorToken::Operator(Operator::ShiftRightAssign),
            "<<=" => PreprocessorToken::Operator(Operator::ShiftLeftAssign),
            "(" => PreprocessorToken::Bracing(Bracing::LeftParenthesis),
            ")" => PreprocessorToken::Bracing(Bracing::RightParenthesis),
            "[" => PreprocessorToken::Bracing(Bracing::LeftBracket),
            "]" => PreprocessorToken::Bracing(Bracing::RightBracket),
            "{" => PreprocessorToken::Bracing(Bracing::LeftBrace),
            "}" => PreprocessorToken::Bracing(Bracing::RightBrace),

            "defined" => PreprocessorToken::DefinedOperator,
            _ => {
                if (token_str.starts_with('\"') && token_str.char_indices().skip(1).all(|(i, c)| c != '\"' || i==(token_str.len()-1)))
                    || (token_str.starts_with('\'') && token_str.char_indices().skip(1).all(|(i, c)| c != '\'' || i==(token_str.len()-1)))
                {
                    PreprocessorToken::LiteralString(
                        token_str
                            .get(1..token_str.len() - 1)
                            .expect("Catastrophic failure")
                            .to_string(),
                    )
                } else if token_str.chars().all(is_not_operator) {
                    PreprocessorToken::Macro(token_str.to_string())
                } else {
                    return None;
                }
            }
        }
    };
    Some(token)
}

#[rustfmt::skip]
pub fn parse_preprocessor(string: &str) -> Vec<PreprocessorToken> {
    let mut tokens: Vec<PreprocessorToken> = vec![];
    let mut current_token: String = String::new();
    string.chars().for_each(|c| {
        let new_token = current_token.clone() + &String::from(c);
        if let Some(token) = token_from_str(&new_token) {
            current_token = new_token;
        } else if let Some(token) = token_from_str(&current_token) {
            tokens.push(token);
            current_token.clear();
            current_token.push(c);
        } else {
            current_token.clear();
        }
    });
    if let Some(token) = token_from_str(&current_token) {
        tokens.push(token);
    }
    tokens
}

/*
Tokens (23)

++
--
>>
<<
!=
==
+=
-=
*=
/=
%=
<=
>=
&=
|=
^=
&&
||
()
[]
->
>>=
<<=
 */
