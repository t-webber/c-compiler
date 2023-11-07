use std::fmt::Debug;

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
    While
}

pub enum Operator {
    Plus,
    Minus,
    Mul,
    Div,
    Modulo,
    Not,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    Negate,
    
        
    And,
    Or,
    Xor,
    ShiftLeft,
    ShiftRight,
    Increment,
    Decrement,
    ShiftLeftAssign,
    ShiftRightAssign,
    NotEqual,
    Eequal,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModuloAssign,
    BitwiseAndAssign,
}

pub enum Token {
    Keyword(Keyword),
    Operator(Operator),
    Identifier,
    Constant,
    String,
    SpecialSymbol,
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl From<char> for Token {
    fn from(value: char) -> Self {
        Token(String::from(value))
    }
}

#[rustfmt::skip]
pub fn parse(string: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![Token::default()];
    string.chars().for_each(|c| match c {
        'A'..='z' | '0'..='9' => tokens.last_mut().unwrap().0.push(c),
        ' ' => if !tokens.last().unwrap().0.is_empty() { tokens.push(Token::default()) },
        _ if tokens.last().unwrap().0.is_empty() => { tokens.last_mut().unwrap().0.push(c); tokens.push(Token::default()) }
        _  => { tokens.push(Token::from(c)); tokens.push(Token::default()) }
    });
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
