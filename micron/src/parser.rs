use crate::errors::Code;
use crate::tokenizer::{Token, TokenCheck, TokenInfo};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    Int(isize),
    None,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Value::*;
        match *self {
            Str(ref s) => write!(f, "{:?} (an Str)", s),
            Int(i) => write!(f, "{} (an Int)", i),
            None => write!(f, "None"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Value(Value),
    FunCall(Box<Fun>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Fun {
    Set(Expr, Expr),
    Get(Expr),
    Write(Expr),
    Print(Expr),
    Add(Expr, Expr),
    Jump(Expr),
    Equal(Expr, Expr),
    Convert(Expr),
    Extract(Expr, Expr),
    If(Expr, Expr),
    Input,
    KeyChar,
    Text(Expr),
    Number(Expr),
    CatchError(Expr, Expr),
    ThrowError(Expr),
    Return(Expr),
    FunJump(Expr),
    EmptySlot,
    Exit,
}

impl fmt::Display for Fun {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Fun::*;
        match *self {
            Set(..) => write!(f, "s:"),
            Get(..) => write!(f, "g:"),
            Write(..) => write!(f, "w:"),
            Print(..) => write!(f, "p:"),
            Add(..) => write!(f, "a:"),
            Jump(..) => write!(f, "j:"),
            Equal(..) => write!(f, "e:"),
            Convert(..) => write!(f, "c:"),
            Extract(..) => write!(f, "x:"),
            If(..) => write!(f, "?:"),
            Input => write!(f, "i"),
            KeyChar => write!(f, "k"),
            Text(..) => write!(f, "t:"),
            Number(..) => write!(f, "n:"),
            CatchError(..) => write!(f, "#:"),
            ThrowError(..) => write!(f, "!:"),
            Return(..) => write!(f, "r:"),
            FunJump(..) => write!(f, "f:"),
            EmptySlot => write!(f, "~"),
            Exit => write!(f, "$"),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Instr {
    SetLabel(String),
    LabelPlaceHolder(String),
    FunCall(Box<Fun>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstrInfo {
    pub instr: Instr,
    pub start: usize,
    pub end: usize,
}

impl InstrInfo {
    fn new(instr: Instr, start: usize, end: usize) -> Self {
        Self { instr, start, end }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    LabelAlreadySet {
        label: String,
        line: Vec<TokenInfo>,
    },
    UnexpectedToken(TokenInfo),
    InvalidSyntax,
    NotEnoughArgument {
        token_info: TokenInfo,
        got: usize,
        expected: usize,
    },
    UnknownFunctionName(TokenInfo),
}

impl ParseError {
    pub fn error_code(&self) -> Code {
        use ParseError::*;
        Code::ErrCode(match self {
            LabelAlreadySet { .. } => 301,
            UnexpectedToken(..) => 302,
            InvalidSyntax => 303,
            NotEnoughArgument { .. } => 304,
            UnknownFunctionName(..) => 305,
        })
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ParseError::*;
        match *self {
            LabelAlreadySet { .. } => write!(f, "LabelAlreadySet"),
            UnexpectedToken(..) => write!(f, "UnexpectedToken"),
            InvalidSyntax => write!(f, "InvalidSyntax"),
            NotEnoughArgument { .. } => write!(f, "NotEnoughArgument"),
            UnknownFunctionName(..) => write!(f, "UnknownFunctionName"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseErrorInfo {
    pub error: ParseError,
    pub line: Vec<TokenInfo>,
    pub note: Option<String>,
}

impl ParseErrorInfo {
    fn new(error: ParseError, line: Vec<TokenInfo>, note: Option<String>) -> Self {
        Self { error, line, note }
    }
}

pub fn parse(
    token_infos: Vec<TokenInfo>,
) -> Result<(HashMap<String, usize>, Vec<InstrInfo>), ParseErrorInfo> {
    use Token::*;
    let mut labels = HashMap::new();
    let mut instrs = Vec::new();

    let mut token_infos_lines: Vec<Vec<TokenInfo>> = Vec::new();
    let mut i = 0;

    while i < token_infos.len() {
        let mut token_infos_line = Vec::new();

        while i < token_infos.len() {
            match token_infos[i].token {
                Token::Eol => break,
                _ => token_infos_line.push(token_infos[i].clone()),
            }
            i += 1;
        }
        i += 1;

        // filters out empty lines
        if token_infos_line.len() != 0 {
            token_infos_lines.push(token_infos_line);
        }
    }

    for (n, line) in token_infos_lines.iter().enumerate() {
        let mut token_line = Vec::new();

        for token_info in line.iter() {
            token_line.push(token_info.token.clone());
        }

        // ready to match
        match &token_line[..] {
            // ;idn
            [Smi, ..] => match &token_line[..] {
                [Smi, Idn(idn)] => {
                    match labels.get(idn) {
                        Some(line_no) => {
                            let line_no: usize = *line_no;
                            let line_at = token_infos_lines[line_no].clone();
                            return Err(ParseErrorInfo::new(
                                ParseError::LabelAlreadySet {
                                    label: idn.to_string(),
                                    line: line_at,
                                },
                                line.to_vec(),
                                None,
                            ));
                        }

                        None => {
                            labels.insert(idn.to_string(), n);
                            instrs.push(InstrInfo::new(
                                Instr::LabelPlaceHolder(idn.to_string()),
                                line[0].start,
                                line[line.len() - 1].end,
                            ));
                        }
                    };
                }

                _ => {
                    let note: Option<String> = match &token_line[1] {
                        Str(s) if TokenCheck::is_iden(&s) => {
                            Some(format!("Maybe you meant `{}{}`", Smi, s))
                        }
                        _ => None,
                    };

                    return Err(ParseErrorInfo::new(
                        ParseError::UnexpectedToken(line[1].clone()),
                        line.to_vec(),
                        note,
                    ));
                }
            },

            // idn:
            [Idn(_) | Dol | Que | Eql | Not | Hsh, ..] => {
                let (_i, expr) = parse_func_call(line.clone())?;

                match expr {
                    Expr::FunCall(fun) => instrs.push(InstrInfo::new(
                        Instr::FunCall(fun),
                        line[0].start,
                        line[line.len() - 1].end,
                    )),

                    _ => panic!("Got `{:?}`. (This error is not supposed to occur.)", expr),
                }
            }

            _ => {
                return Err(ParseErrorInfo::new(
                    ParseError::UnexpectedToken(line[0].clone()),
                    line.to_vec(),
                    None,
                ))
            }
        }
    }

    Ok((labels, instrs))
}

pub fn parse_dot_op(line: Vec<TokenInfo>) -> Result<(usize, Expr), ParseErrorInfo> {
    let mut token_line = Vec::new();

    for tok_inf in line.iter() {
        token_line.push(tok_inf.token.clone());
    }

    match &token_line[..] {
        [Token::Dot, Token::Int(int), ..] => {
            return Ok((
                1,
                Expr::FunCall(Box::new(Fun::Get(Expr::Value(Value::Int(*int))))),
            ));
        }

        _ => {
            return Err(ParseErrorInfo::new(
                ParseError::UnexpectedToken(line[1].clone()),
                line,
                None,
            ))
        }
    }
}

pub fn parse_func_call(line: Vec<TokenInfo>) -> Result<(usize, Expr), ParseErrorInfo> {
    let mut token_line = Vec::new();
    for tok_inf in line.iter() {
        token_line.push(tok_inf.token.clone());
    }

    let mut i = 0;
    let mut c = 0;
    let mut args = Vec::new();

    let count = match &token_line[0] {
        Token::Idn(s) => match s.as_str() {
            "s" => 2,
            "g" => 1,
            "w" => 1,
            "p" => 1,
            "a" => 2,
            "j" => 1,
            "c" => 1,
            "x" => 2,
            "i" => 0,
            "k" => 0,
            "n" => 1,
            "t" => 1,
            "f" => 1,
            "r" => 1,
            _ => {
                return Err(ParseErrorInfo::new(
                    ParseError::UnknownFunctionName(line[0].clone()),
                    line,
                    None,
                ))
            }
        },

        Token::Que => 2,
        Token::Eql => 2,
        Token::Hsh => 2,
        Token::Dol => 0,
        Token::Til => 0,
        Token::Not => 1,
        _ => panic!(
            "Got an unknown token `{:?}`. (This error is not supposed to occur.)",
            token_line[0]
        ),
    };

    if count == 0 {
        i += 1;
    } else {
        i += 1;
        match token_line.get(i) {
            Some(token) => match token {
                Token::Col => i += 1,
                _ => {
                    return Err(ParseErrorInfo::new(
                        ParseError::UnexpectedToken(line[i].clone()),
                        line,
                        None,
                    ))
                }
            },

            None => {
                return Err(ParseErrorInfo::new(
                    ParseError::NotEnoughArgument {
                        token_info: line[0].clone(),
                        expected: count,
                        got: 0,
                    },
                    line,
                    None,
                ))
            }
        }

        while c < count {
            match token_line.get(c + i) {
                Some(token) => match token {
                    Token::Str(s) => args.push(Expr::Value(Value::Str(s.to_string()))),

                    Token::Int(int) => args.push(Expr::Value(Value::Int(*int))),

                    Token::Idn(_) | Token::Eql | Token::Que | Token::Not | Token::Hsh => {
                        let (_i, _args) = parse_func_call(line[c + i..].to_vec())?;
                        args.push(_args);
                        i += _i;
                    }

                    Token::Til => args.push(Expr::FunCall(Box::new(Fun::EmptySlot))),

                    Token::Dol => args.push(Expr::FunCall(Box::new(Fun::Exit))),

                    Token::Dot => {
                        let (_i, _args) = parse_dot_op(line[c + i..].to_vec())?;
                        args.push(_args);
                        i += _i;
                    }

                    _ => {
                        return Err(ParseErrorInfo::new(
                            ParseError::UnexpectedToken(line[c].clone()),
                            line,
                            None,
                        ))
                    }
                },

                None => {
                    return Err(ParseErrorInfo::new(
                        ParseError::NotEnoughArgument {
                            token_info: line[0].clone(),
                            expected: count,
                            got: c,
                        },
                        line,
                        None,
                    ))
                }
            }

            c += 1;
        }
    }

    let fun = match &token_line[0] {
        Token::Idn(s) => {
            match s.as_str() {
                "s" => Fun::Set(args[0].clone(), args[1].clone()), // set
                "g" => Fun::Get(args[0].clone()),
                "w" => Fun::Write(args[0].clone()),
                "p" => Fun::Print(args[0].clone()),
                "a" => Fun::Add(args[0].clone(), args[1].clone()),
                "j" => Fun::Jump(args[0].clone()),
                "c" => Fun::Convert(args[0].clone()),
                "x" => Fun::Extract(args[0].clone(), args[1].clone()),
                "i" => Fun::Input,
                "k" => Fun::KeyChar,
                "n" => Fun::Number(args[0].clone()),
                "t" => Fun::Text(args[0].clone()),
                "f" => Fun::FunJump(args[0].clone()),
                "r" => Fun::Return(args[0].clone()),
                _ => panic!(
                    "Got an unknow function name `{}`. (This error is not supposed to occur.)",
                    s
                ),
            }
        }

        Token::Que => Fun::If(args[0].clone(), args[1].clone()),
        Token::Eql => Fun::Equal(args[0].clone(), args[1].clone()),
        Token::Hsh => Fun::CatchError(args[0].clone(), args[1].clone()),
        Token::Dol => Fun::Exit,
        Token::Til => Fun::EmptySlot,
        Token::Not => Fun::ThrowError(args[0].clone()),
        _ => panic!(
            "Got an unknown token `{}`. (This error is not supposed to occur.)",
            token_line[0]
        ),
    };

    Ok((i + c - 1, Expr::FunCall(Box::new(fun))))
}
