// #[derive(Debug)]
// pub enum Keyword {
//     Auto,
//     Double,
//     Int,
//     Struct,
//     Break,
//     Else,
//     Long,
//     Switch,
//     Case,
//     Enum,
//     Register,
//     Typedef,
//     Char,
//     Extern,
//     Return,
//     Union,
//     Const,
//     Float,
//     Short,
//     Unsigned,
//     Continue,
//     For,
//     Signed,
//     Void,
//     Default,
//     Goto,
//     Sizeof,
//     Volatile,
//     Do,
//     If,
//     Static,
//     While,
// }

use crate::{
    arithmetic::CheckedOperations,
    errors::{FailError, FilePosition, SystemError},
    structs::ParsingState,
};

#[allow(unused)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum UnaryOperator {
    Plus,
    Minus,
    Not,
    BitwiseNot,
    Defined,
    Increment,
    Decrement,
}
#[allow(unused)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BinaryOperator {
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
    ShiftLeft,
    ShiftRight,
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
    ShiftLeftAssign,
    ShiftRightAssign,
}
// Unary

#[allow(unused)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Operator {
    Unary(UnaryOperator),
    Binary(BinaryOperator),
    Conditional,
}

#[derive(Eq, PartialEq)]
pub enum Associativity {
    LeftToRight,
    RightToLeft,
}

pub trait OperatorTrait {
    fn precedence(&self) -> u32;
    fn associativity(&self) -> Associativity;
}

impl OperatorTrait for UnaryOperator {
    fn precedence(&self) -> u32 {
        match self {
            | Self::Defined => 0,
            | Self::Increment | Self::Decrement => 1,
            | Self::Plus | Self::Minus | Self::Not | Self::BitwiseNot => 2,
        }
    }

    fn associativity(&self) -> Associativity {
        match self {
            | Self::Plus | Self::Minus | Self::Not | Self::BitwiseNot => Associativity::RightToLeft,
            | Self::Increment | Self::Defined | Self::Decrement => Associativity::LeftToRight,
        }
    }
}

impl OperatorTrait for BinaryOperator {
    fn precedence(&self) -> u32 {
        match self {
            | Self::Mul | Self::Div | Self::Mod => 3,
            | Self::Add | Self::Sub => 4,
            | Self::ShiftLeft | Self::ShiftRight => 5,
            | Self::LessThan | Self::LessEqual | Self::GreaterThan | Self::GreaterEqual => 6,
            | Self::Eequal | Self::NotEqual => 7,
            | Self::BitwiseAnd => 8,
            | Self::BitwiseXor => 9,
            | Self::BitwiseOr => 10,
            | Self::And => 11,
            | Self::Or => 12,
            | Self::AddAssign
            | Self::SubAssign
            | Self::MulAssign
            | Self::DivAssign
            | Self::ModAssign
            | Self::OrAssign
            | Self::AndAssign
            | Self::XorAssign
            | Self::ShiftLeftAssign
            | Self::ShiftRightAssign => 14,
        }
    }

    fn associativity(&self) -> Associativity {
        match self {
            | Self::AddAssign
            | Self::SubAssign
            | Self::DivAssign
            | Self::ModAssign
            | Self::OrAssign
            | Self::AndAssign
            | Self::XorAssign
            | Self::ShiftLeftAssign
            | Self::ShiftRightAssign => Associativity::RightToLeft,
            | Self::Sub
            | Self::Mul
            | Self::Div
            | Self::Mod
            | Self::BitwiseAnd
            | Self::BitwiseOr
            | Self::Add
            | Self::BitwiseXor
            | Self::And
            | Self::Or
            | Self::ShiftLeft
            | Self::ShiftRight
            | Self::NotEqual
            | Self::Eequal
            | Self::LessThan
            | Self::GreaterThan
            | Self::LessEqual
            | Self::GreaterEqual
            | Self::MulAssign => Associativity::LeftToRight,
        }
    }
}

impl OperatorTrait for Operator {
    fn precedence(&self) -> u32 {
        match self {
            | Self::Unary(op) => op.precedence(),
            | Self::Binary(op) => op.precedence(),
            | Self::Conditional => 13,
        }
    }
    fn associativity(&self) -> Associativity {
        match self {
            | Self::Unary(op) => op.associativity(),
            | Self::Binary(op) => op.associativity(),
            | Self::Conditional => Associativity::RightToLeft,
        }
    }
}

impl Operator {
    pub const fn max_precedence() -> u32 {
        15
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Bracing {
    LeftParenthesis,
    RightParenthesis,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NonOpSymbol {
    Interrogation,
    Colon,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreprocessorToken {
    Operator(Operator),
    Bracing(Bracing),
    LiteralString(String),
    LiteralNumber(i32),
    Macro(String),
    NonOpSymbol(NonOpSymbol),
}

fn is_not_operator(ch: char) -> bool {
    const OPERATORS: [char; 26] = [
        ' ', '!', '+', '-', '*', '/', '%', '&', '|', '^', '<', '>', '(', ')', '{', '}', '[', ']', '=', ',', ';', ':', '?', '~', '#', '\\',
    ];
    !OPERATORS.contains(&ch)
}

fn token_from_str(token_str: &str, current_position: &FilePosition) -> Option<PreprocessorToken> {
    if token_str.is_empty() {
        return None;
    };
    let no_operator = token_str.chars().all(is_not_operator);

    if let Ok(number) = token_str.parse::<f32>() {
        #[allow(clippy::cast_possible_truncation, clippy::as_conversions)]
        no_operator.then_some(PreprocessorToken::LiteralNumber(number as i32))
    } else {
        use BinaryOperator as BOp;
        use Operator as Op;
        use PreprocessorToken as Tok;
        use UnaryOperator as UOp;
        Some(match token_str {
            | "(" => Tok::Bracing(Bracing::LeftParenthesis),
            | ")" => Tok::Bracing(Bracing::RightParenthesis),
            | "[" => Tok::Bracing(Bracing::LeftBracket),
            | "]" => Tok::Bracing(Bracing::RightBracket),
            | "{" => Tok::Bracing(Bracing::LeftBrace),
            | "}" => Tok::Bracing(Bracing::RightBrace),
            | "?" => Tok::NonOpSymbol(NonOpSymbol::Interrogation),
            | ":" => Tok::NonOpSymbol(NonOpSymbol::Colon),
            | "*" => Tok::Operator(Op::Binary(BOp::Mul)),
            | "/" => Tok::Operator(Op::Binary(BOp::Div)),
            | "%" => Tok::Operator(Op::Binary(BOp::Mod)),
            | "&" => Tok::Operator(Op::Binary(BOp::BitwiseAnd)),
            | "|" => Tok::Operator(Op::Binary(BOp::BitwiseOr)),
            | "^" => Tok::Operator(Op::Binary(BOp::BitwiseXor)),
            | "<" => Tok::Operator(Op::Binary(BOp::LessThan)),
            | ">" => Tok::Operator(Op::Binary(BOp::GreaterThan)),
            | ">>" => Tok::Operator(Op::Binary(BOp::ShiftRight)),
            | "<<" => Tok::Operator(Op::Binary(BOp::ShiftLeft)),
            | "!=" => Tok::Operator(Op::Binary(BOp::NotEqual)),
            | "==" => Tok::Operator(Op::Binary(BOp::Eequal)),
            | "+=" => Tok::Operator(Op::Binary(BOp::AddAssign)),
            | "-=" => Tok::Operator(Op::Binary(BOp::SubAssign)),
            | "*=" => Tok::Operator(Op::Binary(BOp::MulAssign)),
            | "/=" => Tok::Operator(Op::Binary(BOp::DivAssign)),
            | "%=" => Tok::Operator(Op::Binary(BOp::ModAssign)),
            | "<=" => Tok::Operator(Op::Binary(BOp::LessEqual)),
            | ">=" => Tok::Operator(Op::Binary(BOp::GreaterEqual)),
            | "&=" => Tok::Operator(Op::Binary(BOp::AndAssign)),
            | "|=" => Tok::Operator(Op::Binary(BOp::OrAssign)),
            | "^=" => Tok::Operator(Op::Binary(BOp::XorAssign)),
            | "&&" => Tok::Operator(Op::Binary(BOp::And)),
            | "||" => Tok::Operator(Op::Binary(BOp::Or)),
            | ">>=" => Tok::Operator(Op::Binary(BOp::ShiftRightAssign)),
            | "<<=" => Tok::Operator(Op::Binary(BOp::ShiftLeftAssign)),
            | "!" => Tok::Operator(Op::Unary(UOp::Not)),
            | "+" => Tok::Operator(Op::Unary(UOp::Plus)),
            | "-" => Tok::Operator(Op::Unary(UOp::Minus)),
            | "++" => Tok::Operator(Op::Unary(UOp::Increment)),
            | "--" => Tok::Operator(Op::Unary(UOp::Decrement)),
            | "~" => Tok::Operator(Op::Unary(UOp::BitwiseNot)),
            | "defined" => Tok::Operator(Op::Unary(UOp::Defined)),
            | _ => {
                if (token_str.starts_with('\"')
                    && token_str
                        .char_indices()
                        .skip(1)
                        .all(|(i, ch)| ch != '\"' || i == (token_str.len().checked_sub_unwrap(1, current_position))))
                    || (token_str.starts_with('\'')
                        && token_str
                            .char_indices()
                            .skip(1)
                            .all(|(i, ch)| ch != '\'' || i == (token_str.len().checked_sub_unwrap(1, current_position))))
                {
                    PreprocessorToken::LiteralString(
                        token_str
                            .get(1..token_str.len().checked_sub_unwrap(1, current_position))
                            .unwrap_or_else(|| {
                                SystemError::CompilationError("Found string but could not parse the delimiters.").fail_with_panic(current_position)
                            })
                            .to_owned(),
                    )
                } else if no_operator {
                    PreprocessorToken::Macro(token_str.to_owned())
                } else {
                    return None;
                }
            },
        })
    }
}

#[rustfmt::skip]
pub fn parse_preprocessor(string: &str, state: &mut ParsingState) -> Vec<PreprocessorToken> {
    let mut tokens: Vec<PreprocessorToken> = vec![];
    let mut current_token = String::new();
    string.chars().for_each(|ch| {
        let mut new_token = current_token.clone();
        new_token.push(ch);
        if token_from_str(&new_token, &state.current_position).is_some() {
            current_token = new_token;
        } else if let Some(token) = token_from_str(&current_token, &state.current_position) {
            tokens.push(token);
            current_token.clear();
            current_token.push(ch);
        } else {
            current_token.clear();
            current_token.push(ch);
        }
        state.current_position.col = state.current_position.col.checked_add_unwrap(1, &state.current_position);
    });
    if let Some(token) = token_from_str(&current_token, &state.current_position) {
        tokens.push(token);
    }
    tokens
}
