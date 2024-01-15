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

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    // Unary
    // Plus,
    // Minus,
    Not,
    BitwiseNot,

    // Binary
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Conditional,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    And,
    Or,
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

    ShiftLeftAssign,
    ShiftRightAssign,

    Defined,
}

// enum Associativity {
//     LeftToRight,
//     RightToLeft,
// }

impl Operator {
    pub fn max_precedence() -> u32 {
        14
    }

    pub fn precedence(&self) -> u32 {
        match self {
            Operator::Defined => 0,

            // Operator::Increment => 1,
            // Operator::Decrement => 1,
            // ()
            // []
            // . ->
            // (type){elt}
            //
            // prefix increment / decrement
            Operator::Plus => 2,
            Operator::Minus => 2,
            Operator::Not => 2,
            Operator::BitwiseNot => 2,
            // (cast)
            // * & sizeof _alignof
            //
            Operator::Mul => 3,
            Operator::Div => 3,
            Operator::Mod => 3,
            //
            Operator::Add => 4,
            Operator::Sub => 4,
            //
            Operator::ShiftLeft => 5,
            Operator::ShiftRight => 5,
            //
            Operator::LessThan => 6,
            Operator::LessEqual => 6,
            Operator::GreaterThan => 6,
            Operator::GreaterEqual => 6,
            //
            Operator::Eequal => 7,
            Operator::NotEqual => 7,

            Operator::BitwiseAnd => 8,

            Operator::BitwiseXor => 9,

            Operator::BitwiseOr => 10,

            Operator::And => 11,

            Operator::Or => 12,

            Operator::Conditional => 13,

            Operator::AddAssign => 14,
            Operator::SubAssign => 14,
            Operator::MulAssign => 14,
            Operator::DivAssign => 14,
            Operator::ModAssign => 14,
            Operator::OrAssign => 14,
            Operator::AndAssign => 14,
            Operator::XorAssign => 14,
            Operator::ShiftLeftAssign => 14,
            Operator::ShiftRightAssign => 14,
        }
    }

    // pub fn associativity(&self) -> Associativity {
    //     match self {
    //         Operator::Defined => Associativity::LeftToRight,

    //         Operator::Increment => Associativity::LeftToRight,
    //         Operator::Decrement => Associativity::LeftToRight,
    //         // ()
    //         // []
    //         // . ->
    //         // (type){elt}

    //         // prefix increment / decrement
    //         Operator::Plus => Associativity::RightToLeft,
    //         Operator::Minus => Associativity::RightToLeft,
    //         Operator::Not => Associativity::RightToLeft,
    //         Operator::BitwiseNot => Associativity::RightToLeft,
    //         // (cast)
    //         // * & sizeof _alignof
    //         Operator::Mul => Associativity::LeftToRight,
    //         Operator::Div => Associativity::LeftToRight,
    //         Operator::Mod => Associativity::LeftToRight,

    //         Operator::Add => Associativity::LeftToRight,
    //         Operator::Sub => Associativity::LeftToRight,

    //         Operator::ShiftLeft => Associativity::LeftToRight,
    //         Operator::ShiftRight => Associativity::LeftToRight,

    //         Operator::LessThan => Associativity::LeftToRight,
    //         Operator::LessEqual => Associativity::LeftToRight,
    //         Operator::GreaterThan => Associativity::LeftToRight,
    //         Operator::GreaterEqual => Associativity::LeftToRight,

    //         Operator::Eequal => Associativity::LeftToRight,
    //         Operator::NotEqual => Associativity::LeftToRight,

    //         Operator::BitwiseAnd => Associativity::LeftToRight,

    //         Operator::BitwiseXor => Associativity::LeftToRight,

    //         Operator::BitwiseOr => Associativity::LeftToRight,

    //         Operator::And => Associativity::LeftToRight,

    //         Operator::Or => Associativity::LeftToRight,

    //         Operator::Conditional => Associativity::RightToLeft,

    //         Operator::AddAssign => Associativity::RightToLeft,
    //         Operator::SubAssign => Associativity::RightToLeft,
    //         Operator::MulAssign => Associativity::RightToLeft,
    //         Operator::DivAssign => Associativity::RightToLeft,
    //         Operator::ModAssign => Associativity::RightToLeft,
    //         Operator::OrAssign => Associativity::RightToLeft,
    //         Operator::AndAssign => Associativity::RightToLeft,
    //         Operator::XorAssign => Associativity::RightToLeft,
    //         Operator::ShiftLeftAssign => Associativity::RightToLeft,
    //         Operator::ShiftRightAssign => Associativity::RightToLeft,
    //     }
    // }
}

// #[derive(Debug)]
// pub enum Token {
//     Keyword(Keyword),
//     Operator(Operator),
//     Identifier,
//     Constant,
//     String,
//     SpecialSymbol,
// }

#[derive(Debug, Clone, PartialEq)]
pub enum Bracing {
    LeftParenthesis,
    RightParenthesis,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NonOpSymbol {
    Interrogation,
    Colon,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PreprocessorToken {
    Operator(Operator),
    Bracing(Bracing),
    LiteralString(String),
    LiteralNumber(f32),
    Macro(String),
    NonOpSymbol(NonOpSymbol),
}

fn is_not_operator(c: char) -> bool {
    const OPERATORS: [char; 26] = [
        ' ', '!', '+', '-', '*', '/', '%', '&', '|', '^', '<', '>', '(', ')', '{', '}', '[', ']',
        '=', ',', ';', ':', '?', '~', '#', '\\',
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
            "?" => PreprocessorToken::NonOpSymbol(NonOpSymbol::Interrogation),
            ":" => PreprocessorToken::NonOpSymbol(NonOpSymbol::Colon),
            "+" => PreprocessorToken::Operator(Operator::Add),
            "-" => PreprocessorToken::Operator(Operator::Sub),
            "*" => PreprocessorToken::Operator(Operator::Mul),
            "~" => PreprocessorToken::Operator(Operator::BitwiseNot),
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
            ">>=" => PreprocessorToken::Operator(Operator::ShiftRightAssign),
            "<<=" => PreprocessorToken::Operator(Operator::ShiftLeftAssign),
            "(" => PreprocessorToken::Bracing(Bracing::LeftParenthesis),
            ")" => PreprocessorToken::Bracing(Bracing::RightParenthesis),
            "[" => PreprocessorToken::Bracing(Bracing::LeftBracket),
            "]" => PreprocessorToken::Bracing(Bracing::RightBracket),
            "{" => PreprocessorToken::Bracing(Bracing::LeftBrace),
            "}" => PreprocessorToken::Bracing(Bracing::RightBrace),

            "defined" => PreprocessorToken::Operator(Operator::Defined),
            _ => {
                if (token_str.starts_with('\"')
                    && token_str
                        .char_indices()
                        .skip(1)
                        .all(|(i, c)| c != '\"' || i == (token_str.len() - 1)))
                    || (token_str.starts_with('\'')
                        && token_str
                            .char_indices()
                            .skip(1)
                            .all(|(i, c)| c != '\'' || i == (token_str.len() - 1)))
                {
                    PreprocessorToken::LiteralString(
                        token_str
                            .get(1..token_str.len() - 1)
                            .expect("Catastrophic failure")
                            .to_string(),
                    )
                } else if no_operator {
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
        // println!("Current = {current_token:?} and new = {new_token:?}");
        if let Some(_) = token_from_str(&new_token) {
            current_token = new_token;
            // println!("Chose new");
        } else if let Some(token) = token_from_str(&current_token) {
            tokens.push(token);
            current_token.clear();
            current_token.push(c);
            // println!("Chose current");
        } else {
            current_token.clear();
            current_token.push(c);
            // println!("Chose none");
        }
    });
    if let Some(token) = token_from_str(&current_token) {
        tokens.push(token);
    }
    tokens
}
