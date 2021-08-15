use crate::errors::Code;
use crate::scanner;
use scanner::Char;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Str(String), // String Literal
    Int(isize),  // Integer
    Idn(String), // Identifier
    Til,         // ~
    Col,         // :
    Smi,         // ;
    Dot,         // .
    Eol,         // \n
    Dol,         // $
    Que,         // ?
    Eql,         // =
    Not,         // !
    Hsh,         // #
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Token::*;
        match self {
            Str(s) => write!(f, "{:?}", s),
            Int(i) => write!(f, "{}", i),
            Idn(s) => write!(f, "{}", s),
            Til => write!(f, "~"),
            Col => write!(f, ":"),
            Smi => write!(f, ";"),
            Dot => write!(f, "."),
            Eol => write!(f, "EOL"),
            Dol => write!(f, "$"),
            Que => write!(f, "?"),
            Eql => write!(f, "="),
            Not => write!(f, "!"),
            Hsh => write!(f, "#"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenInfo {
    pub token: Token,
    pub start: usize,
    pub end: usize,
}

impl TokenInfo {
    pub fn new(start: usize, end: usize, token: Token) -> Self {
        Self { token, start, end }
    }

    #[allow(dead_code)]
    pub fn from_chars(chars: &Vec<Char>, token: Token) -> Self {
        let first = &chars[0];
        let last = &chars[chars.len() - 1];

        Self {
            token,
            start: first.n,
            end: last.n,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenizerError {
    SyntaxError(ErrorInfo),
}

impl fmt::Display for TokenizerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TokenizerError::*;
        match self {
            SyntaxError(..) => write!(f, "SyntaxError"),
        }
    }
}

impl TokenizerError {
    pub fn error_code(&self) -> Code {
        use TokenizerError::*;
        Code::ErrCode(match self {
            SyntaxError(..) => 201,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorInfo {
    pub start: usize,
    pub end: usize,
    pub msg: Option<String>,
}

pub fn tokenize(chars: Vec<Char>) -> Result<Vec<TokenInfo>, TokenizerError> {
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        match chars[i].char {
            '~' => tokens.push(TokenInfo::new(i, i, Token::Til)),
            ':' => tokens.push(TokenInfo::new(i, i, Token::Col)),
            '.' => tokens.push(TokenInfo::new(i, i, Token::Dot)),
            '\n' => tokens.push(TokenInfo::new(i, i, Token::Eol)),
            '$' => tokens.push(TokenInfo::new(i, i, Token::Dol)),
            ';' => tokens.push(TokenInfo::new(i, i, Token::Smi)),
            '?' => tokens.push(TokenInfo::new(i, i, Token::Que)),
            '=' => tokens.push(TokenInfo::new(i, i, Token::Eql)),
            '!' => tokens.push(TokenInfo::new(i, i, Token::Not)),
            '#' => tokens.push(TokenInfo::new(i, i, Token::Hsh)),
            ' ' => {}

            'a'..='z' | 'A'..='Z' | '_' => {
                let j = i;
                let mut temp = Vec::new();

                while i < chars.len() {
                    match chars[i].char {
                        'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => temp.push(chars[i]),
                        _ => break,
                    }
                    i += 1;
                }

                let col = Char::extract(&temp).iter().collect::<String>();
                tokens.push(TokenInfo::new(j, i, Token::Idn(col)));
                i -= 1;
            }

            '0'..='9' | '-' => {
                let j = i;
                i += 1;
                let mut temp = vec![chars[j]];

                while i < chars.len() {
                    match chars[i].char {
                        '0'..='9' => temp.push(chars[i]),
                        'a'..='z' | 'A'..='Z' => {
                            return Err(TokenizerError::SyntaxError(ErrorInfo {
                                start: j,
                                end: i,
                                msg: Some("Invalid number literal".to_string()),
                            }))
                        }
                        _ => break,
                    }
                    i += 1;
                }

                let col = Char::extract(&temp).iter().collect::<String>();

                let num = match col.parse::<isize>() {
                    Ok(ok) => ok,
                    Err(_) => {
                        return Err(TokenizerError::SyntaxError(ErrorInfo {
                            start: j,
                            end: i,
                            msg: Some("Invalid isize".to_string()),
                        }))
                    }
                };

                tokens.push(TokenInfo::new(j, i, Token::Int(num)));
                i -= 1;
            }

            '"' => {
                let j = i;
                i += 1;
                let mut temp = Vec::new();

                if !(i < chars.len()) {
                    return Err(TokenizerError::SyntaxError(ErrorInfo {
                        start: j,
                        end: j,
                        msg: Some("EOF while scanning for the string literal".to_string()),
                    }));
                }

                while chars[i].char != '"' {
                    if chars[i].char == '\\' {
                        if let Some(chr) = chars.get(i + 1) {
                            let _char = match chr.char {
                                'n' => '\n',
                                't' => '\t',
                                _ => chr.char,
                            };

                            temp.push(Char::new(_char, chr.n));
                        } else {
                            return Err(TokenizerError::SyntaxError(ErrorInfo {
                                start: i,
                                end: i,
                                msg: Some("EOF while scanning for the escape sequence".to_string()),
                            }));
                        }

                        i += 1;
                    } else {
                        temp.push(chars[i]);
                    }
                    i += 1;

                    if !(i < chars.len()) {
                        return Err(TokenizerError::SyntaxError(ErrorInfo {
                            start: j,
                            end: j,
                            msg: Some("EOF while scanning for the string literal".to_string()),
                        }));
                    }
                }

                let col = Char::extract(&temp).iter().collect::<String>();
                tokens.push(TokenInfo::new(j, i + 1, Token::Str(col)));
            }

            '[' => {
                let j = i;
                i += 1;

                if !(i < chars.len()) {
                    return Err(TokenizerError::SyntaxError(ErrorInfo {
                        start: j,
                        end: j,
                        msg: Some("EOF while scanning for the comment literal".to_string()),
                    }));
                }

                while chars[i].char != ']' {
                    i += 1;
                    if !(i < chars.len()) {
                        return Err(TokenizerError::SyntaxError(ErrorInfo {
                            start: j,
                            end: j,
                            msg: Some("EOF while scanning for the comment literal".to_string()),
                        }));
                    }
                }
            }

            _ => {
                return Err(TokenizerError::SyntaxError(ErrorInfo {
                    start: i,
                    end: i,
                    msg: Some("Invalid character".to_string()),
                }))
            }
        }

        i += 1;
    }

    Ok(tokens)
}

pub struct TokenCheck;

impl TokenCheck {
    pub fn is_iden(s: &String) -> bool {
        let mut i = 0;
        let chars = s.chars().collect::<Vec<char>>();

        while i < chars.len() {
            match chars[i] {
                'a'..='z' | 'A'..='Z' | '_' => {
                    while i < chars.len() {
                        match chars[i] {
                            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {}
                            _ => return false,
                        }
                        i += 1;
                    }
                }
                _ => return false,
            }
        }

        true
    }
}
