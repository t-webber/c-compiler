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
            Self::Defined => 0,
            Self::Increment | Self::Decrement => 1,
            Self::Plus | Self::Minus | Self::Not | Self::BitwiseNot => 2,
        }
    }

    fn associativity(&self) -> Associativity {
        match self {
            Self::Plus | Self::Minus | Self::Not | Self::BitwiseNot => Associativity::RightToLeft,
            Self::Increment | Self::Defined | Self::Decrement => Associativity::LeftToRight,
        }
    }
}

impl OperatorTrait for BinaryOperator {
    fn precedence(&self) -> u32 {
        match self {
            Self::Mul | Self::Div | Self::Mod => 3,
            Self::Add | Self::Sub => 4,
            Self::ShiftLeft | Self::ShiftRight => 5,
            Self::LessThan | Self::LessEqual | Self::GreaterThan | Self::GreaterEqual => 6,
            Self::Eequal | Self::NotEqual => 7,
            Self::BitwiseAnd => 8,
            Self::BitwiseXor => 9,
            Self::BitwiseOr => 10,
            Self::And => 11,
            Self::Or => 12,
            Self::AddAssign
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
            Self::AddAssign
            | Self::SubAssign
            | Self::DivAssign
            | Self::ModAssign
            | Self::OrAssign
            | Self::AndAssign
            | Self::XorAssign
            | Self::ShiftLeftAssign
            | Self::ShiftRightAssign => Associativity::RightToLeft,
            Self::Sub
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
            Self::Unary(op) => op.precedence(),
            Self::Binary(op) => op.precedence(),
            Self::Conditional => 13,
        }
    }
    fn associativity(&self) -> Associativity {
        match self {
            Self::Unary(op) => op.associativity(),
            Self::Binary(op) => op.associativity(),
            Self::Conditional => Associativity::RightToLeft,
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
        ' ', '!', '+', '-', '*', '/', '%', '&', '|', '^', '<', '>', '(', ')', '{', '}', '[', ']',
        '=', ',', ';', ':', '?', '~', '#', '\\',
    ];
    !OPERATORS.contains(&ch)
}

#[rustfmt::skip]
fn token_from_str(token_str: &str) -> Option<PreprocessorToken> {
    if token_str.is_empty() {
        return None;
    }
    let no_operator = token_str.chars().all(is_not_operator);
    
    let token = if let Ok(number) = token_str.parse::<f32>() {
        if no_operator {
            #[allow(clippy::cast_possible_truncation, clippy::as_conversions)]
            PreprocessorToken::LiteralNumber(number as i32)
        } else {
            return None;
        }
    } else {
        match token_str {
            "?" => PreprocessorToken::NonOpSymbol(NonOpSymbol::Interrogation),
            ":" => PreprocessorToken::NonOpSymbol(NonOpSymbol::Colon),
            "*" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::Mul)),
            "/" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::Div)),
            "%" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::Mod)),
            "&" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::BitwiseAnd)),
            "|" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::BitwiseOr)),
            "^" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::BitwiseXor)),
            "<" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::LessThan)),
            ">" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::GreaterThan)),
            ">>" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::ShiftRight)),
            "<<" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::ShiftLeft)),
            "!=" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::NotEqual)),
            "==" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::Eequal)),
            "+=" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::AddAssign)),
            "-=" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::SubAssign)),
            "*=" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::MulAssign)),
            "/=" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::DivAssign)),
            "%=" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::ModAssign)),
            "<=" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::LessEqual)),
            ">=" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::GreaterEqual)),
            "&=" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::AndAssign)),
            "|=" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::OrAssign)),
            "^=" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::XorAssign)),
            "&&" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::And)),
            "||" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::Or)),
            ">>=" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::ShiftRightAssign)),
            "<<=" => PreprocessorToken::Operator(Operator::Binary(BinaryOperator::ShiftLeftAssign)),
            "(" => PreprocessorToken::Bracing(Bracing::LeftParenthesis),
            ")" => PreprocessorToken::Bracing(Bracing::RightParenthesis),
            "[" => PreprocessorToken::Bracing(Bracing::LeftBracket),
            "]" => PreprocessorToken::Bracing(Bracing::RightBracket),
            "{" => PreprocessorToken::Bracing(Bracing::LeftBrace),
            "}" => PreprocessorToken::Bracing(Bracing::RightBrace),
            "!" => PreprocessorToken::Operator(Operator::Unary(UnaryOperator::Not)),
            "+" =>  PreprocessorToken::Operator(Operator::Unary(UnaryOperator::Plus)),
            "-" => PreprocessorToken::Operator(Operator::Unary(UnaryOperator::Minus)),
            "++" => PreprocessorToken::Operator(Operator::Unary(UnaryOperator::Increment)),
            "--" => PreprocessorToken::Operator(Operator::Unary(UnaryOperator::Decrement)),
            "~" => PreprocessorToken::Operator(Operator::Unary(UnaryOperator::BitwiseNot)),
            "defined" => PreprocessorToken::Operator(Operator::Unary(UnaryOperator::Defined)),
            _ => {
                if (token_str.starts_with('\"')
                    && token_str
                        .char_indices()
                        .skip(1)
                        .all(|(i, ch)| ch != '\"' || i == (token_str.len() - 1)))
                    || (token_str.starts_with('\'')
                        && token_str
                            .char_indices()
                            .skip(1)
                            .all(|(i, ch)| ch != '\'' || i == (token_str.len() - 1)))
                {
                    PreprocessorToken::LiteralString(
                        token_str
                            .get(1..token_str.len() - 1)
                            .expect("Catastrophic failure")
                            .to_owned(),
                    )
                } else if no_operator {
                    PreprocessorToken::Macro(token_str.to_owned())
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
    let mut current_token = String::new();
    string.chars().for_each(|ch| {
        let mut new_token = current_token.clone();
        new_token.push(ch);
        // println!("Current = {current_token:?} and new = {new_token:?}");
        if token_from_str(&new_token).is_some() {
            current_token = new_token;
            // println!("Chose new");
        } else if let Some(token) = token_from_str(&current_token) {
            tokens.push(token);
            current_token.clear();
            current_token.push(ch);
            // println!("Chose current");
        } else {
            current_token.clear();
            current_token.push(ch);
            // println!("Chose none");
        }
    });
    if let Some(token) = token_from_str(&current_token) {
        tokens.push(token);
    }
    tokens
}
