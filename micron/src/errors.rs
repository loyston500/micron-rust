use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Code {
    ErrCode(usize),
}

impl Code {
    pub fn as_isize(&self) -> isize {
        use Code::*;
        *match self {
            ErrCode(code) => code,
        } as isize
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Code::*;
        match *self {
            ErrCode(code) => write!(f, "E{}", code),
        }
    }
}
