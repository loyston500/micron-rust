use crate::errors::Code;
use crate::parser::{Expr, Fun, Instr, InstrInfo, Value};
use std::collections::HashMap;
use std::fmt;
use std::process::exit;

#[derive(Debug, Clone, PartialEq)]
pub struct InterpreterError {
    pub error_info: ErrorInfo,
    pub instr_info: InstrInfo,
}

impl InterpreterError {
    fn new(error_info: ErrorInfo, instr_info: InstrInfo) -> Self {
        Self {
            error_info,
            instr_info,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    TypeError { expected: Value, got: Value },
    LabelError(String),
    ValueError(Value),
    NoSlotError,
    Error(String),
}

impl Error {
    pub fn error_code(&self) -> Code {
        Code::ErrCode(match self {
            Error::TypeError { .. } => 401,
            Error::LabelError(..) => 402,
            Error::ValueError(..) => 403,
            Error::NoSlotError => 404,
            Error::Error(..) => 400,
        })
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::TypeError { .. } => write!(f, "TypeError"),
            Error::LabelError(..) => write!(f, "LabelError"),
            Error::ValueError(..) => write!(f, "ValueError"),
            Error::NoSlotError => write!(f, "NoSlotError"),
            Error::Error(..) => write!(f, "Error"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorInfo {
    pub error: Error,
    pub fun: Fun,
    pub note: Option<String>,
}

impl ErrorInfo {
    fn new(error: Error, fun: Fun, note: Option<String>) -> Self {
        Self { error, fun, note }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Signal {
    Error(ErrorInfo),
    InterpreterError(InterpreterError),
    Jump(usize),
    Return(Value),
}

type LabelType = HashMap<String, usize>;
type SlotType = HashMap<isize, Value>;
type StdOutType = Result<(), ()>;
type StdInType = Result<String, ()>;

pub fn interpret(
    labels: LabelType,
    instr_infos: Vec<InstrInfo>,
    stdout: &mut dyn FnMut(String) -> StdOutType,
    stdin: &mut dyn FnMut() -> StdInType,
) -> Result<(), InterpreterError> {
    let mut slots: SlotType = HashMap::new();

    match interpret_instrs(&instr_infos, &labels, &mut slots, 0, stdout, stdin) {
        Ok(_) => {}
        Err(signal) => match signal {
            Signal::InterpreterError(interpreter_error) => {
                return Err(interpreter_error);
            }

            _ => panic!("got {:#?}", signal),
        },
    }

    Ok(())
}

pub fn interpret_instrs(
    instr_infos: &Vec<InstrInfo>,
    labels: &LabelType,
    slots: &mut SlotType,
    mut i: usize,
    stdout: &mut dyn FnMut(String) -> StdOutType,
    stdin: &mut dyn FnMut() -> StdInType,
) -> Result<Value, Signal> {
    let mut instrs = Vec::new();

    for instr_info in instr_infos.iter() {
        instrs.push(instr_info.instr.clone());
    }

    while i < instrs.len() {
        match &instrs[i] {
            Instr::SetLabel(..) => {} // This instruction is not used.

            Instr::LabelPlaceHolder(..) => {}

            Instr::FunCall(fun) => {
                match interpret_fun_call(fun.clone(), labels, slots, instr_infos, stdout, stdin) {
                    Ok(_) => {}
                    Err(signal) => match signal {
                        Signal::InterpreterError(interpreter_error) => {
                            return Err(Signal::InterpreterError(interpreter_error))
                        }

                        Signal::Error(error_info) => {
                            return Err(Signal::InterpreterError(InterpreterError::new(
                                error_info,
                                instr_infos[i].clone(),
                            )))
                        }

                        Signal::Return(value) => return Ok(value),

                        Signal::Jump(int) => {
                            i = int;
                        }
                    },
                }
            }
        }
        i += 1;
    }

    Ok(Value::None)
}

pub fn interpret_fun_call(
    fun: Box<Fun>,
    labels: &LabelType,
    slots: &mut SlotType,
    instr_infos: &Vec<InstrInfo>,
    stdout: &mut dyn FnMut(String) -> StdOutType,
    stdin: &mut dyn FnMut() -> StdInType,
) -> Result<Value, Signal> {
    let clone = *fun.clone();

    match *fun {
        Fun::Set(expr1, expr2) => {
            let value1 = match expr1 {
                Expr::Value(v) => v,
                Expr::FunCall(_fun) => {
                    interpret_fun_call(_fun, labels, slots, instr_infos, stdout, stdin)?
                }
            };

            let int = match value1 {
                Value::Int(i) => i,
                _ => {
                    return Err(Signal::Error(ErrorInfo::new(
                        Error::TypeError {
                            expected: Value::Int(0),
                            got: value1,
                        },
                        clone,
                        None,
                    )));
                }
            };

            let value2 = match expr2 {
                Expr::Value(v) => v,
                Expr::FunCall(_fun) => {
                    interpret_fun_call(_fun, labels, slots, instr_infos, stdout, stdin)?
                }
            };

            slots.insert(int, value2);
        }

        Fun::Get(expr) => {
            let value = match expr {
                Expr::Value(v) => v,
                Expr::FunCall(_fun) => {
                    interpret_fun_call(_fun, labels, slots, instr_infos, stdout, stdin)?
                }
            };

            let int = match value {
                Value::Int(i) => i,
                _ => {
                    return Err(Signal::Error(ErrorInfo::new(
                        Error::TypeError {
                            expected: Value::Int(0),
                            got: value,
                        },
                        clone,
                        None,
                    )));
                }
            };

            match slots.get(&int) {
                Some(v) => return Ok(v.clone()),
                None => return Ok(Value::None),
            }
        }

        Fun::Jump(expr) => {
            let value = match expr {
                Expr::Value(v) => v,
                Expr::FunCall(_fun) => {
                    interpret_fun_call(_fun, labels, slots, instr_infos, stdout, stdin)?
                }
            };

            let string = match value {
                Value::Str(s) => s,
                _ => {
                    return Err(Signal::Error(ErrorInfo::new(
                        Error::TypeError {
                            expected: Value::Str(String::from("")),
                            got: value,
                        },
                        clone,
                        None,
                    )));
                }
            };

            match labels.get(&string) {
                Some(i) => return Err(Signal::Jump(*i)),
                None => {
                    return Err(Signal::Error(ErrorInfo::new(
                        Error::LabelError(string.to_string()),
                        clone,
                        None,
                    )));
                }
            }
        }

        Fun::FunJump(expr) => {
            let value = match expr {
                Expr::Value(v) => v,
                Expr::FunCall(_fun) => {
                    interpret_fun_call(_fun, labels, slots, instr_infos, stdout, stdin)?
                }
            };

            let string = match value {
                Value::Str(s) => s,
                _ => {
                    return Err(Signal::Error(ErrorInfo::new(
                        Error::TypeError {
                            expected: Value::Str(String::from("")),
                            got: value,
                        },
                        clone,
                        None,
                    )));
                }
            };

            match labels.get(&string) {
                Some(i) => {
                    let result = interpret_instrs(instr_infos, labels, slots, *i, stdout, stdin)?;
                    return Ok(result);
                }
                None => {
                    return Err(Signal::Error(ErrorInfo::new(
                        Error::LabelError(string.to_string()),
                        clone,
                        None,
                    )));
                }
            }
        }

        Fun::Add(expr1, expr2) => {
            let value1 = match expr1 {
                Expr::Value(v) => v,
                Expr::FunCall(_fun) => {
                    interpret_fun_call(_fun, labels, slots, instr_infos, stdout, stdin)?
                }
            };

            let value2 = match expr2 {
                Expr::Value(v) => v,
                Expr::FunCall(_fun) => {
                    interpret_fun_call(_fun, labels, slots, instr_infos, stdout, stdin)?
                }
            };

            match (&value1, &value2) {
                (Value::Int(int1), Value::Int(int2)) => return Ok(Value::Int(int1 + int2)),
                (Value::Str(str1), Value::Str(str2)) => {
                    return Ok(Value::Str(str1.to_owned() + str2))
                }
                _ => {
                    return Err(Signal::Error(ErrorInfo::new(
                        Error::Error(format!(
                            "You are trying to add {} and {} which is invalid",
                            &value1, &value2
                        )),
                        clone,
                        None,
                    )))
                }
            }
        }

        Fun::CatchError(expr1, expr2) => {
            let value1 = match expr1 {
                Expr::Value(v) => v,
                Expr::FunCall(_fun) => {
                    interpret_fun_call(_fun, labels, slots, instr_infos, stdout, stdin)?
                }
            };

            let string = match value1 {
                Value::Str(s) => s,
                _ => {
                    return Err(Signal::Error(ErrorInfo::new(
                        Error::TypeError {
                            expected: Value::Str(String::from("")),
                            got: value1,
                        },
                        clone,
                        None,
                    )));
                }
            };

            let i = *match labels.get(&string) {
                Some(i) => i,
                None => {
                    return Err(Signal::Error(ErrorInfo::new(
                        Error::LabelError(string.to_string()),
                        clone,
                        None,
                    )));
                }
            };

            let value2 = match expr2 {
                Expr::Value(v) => v,
                Expr::FunCall(_fun) => {
                    match interpret_fun_call(_fun, labels, slots, instr_infos, stdout, stdin) {
                        Ok(v) => v,
                        Err(signal) => match signal {
                            Signal::Error(error_info) => {
                                slots.insert(
                                    -1,
                                    Value::Int(error_info.error.error_code().as_isize()),
                                );
                                return Err(Signal::Jump(i));
                            }

                            Signal::InterpreterError(interpreter_error) => {
                                slots.insert(
                                    -1,
                                    Value::Int(
                                        interpreter_error.error_info.error.error_code().as_isize(),
                                    ),
                                );
                                return Err(Signal::Jump(i));
                            }

                            _ => return Err(signal),
                        },
                    }
                }
            };

            return Ok(value2);
        }

        Fun::ThrowError(expr) => {
            let value = match expr {
                Expr::Value(v) => v,
                Expr::FunCall(_fun) => {
                    interpret_fun_call(_fun, labels, slots, instr_infos, stdout, stdin)?
                }
            };

            let string = match value {
                Value::Str(s) => s.to_string(),
                Value::Int(int) => int.to_string(),
                Value::None => "".to_string(),
            };

            return Err(Signal::Error(ErrorInfo::new(
                Error::Error(string),
                clone.clone(),
                Some(format!("This is an error raise by function `{}`", &clone)),
            )));
        }

        Fun::Print(expr) => {
            let value = match expr {
                Expr::Value(v) => v,
                Expr::FunCall(_fun) => {
                    interpret_fun_call(_fun, labels, slots, instr_infos, stdout, stdin)?
                }
            };

            let _ = match value {
                Value::Str(s) => stdout(format!("{}\n", s)),
                Value::Int(int) => stdout(format!("{}\n", int)),
                Value::None => stdout(format!("None\n")),
            };
        }

        Fun::Write(expr) => {
            let value = match expr {
                Expr::Value(v) => v,
                Expr::FunCall(_fun) => {
                    interpret_fun_call(_fun, labels, slots, instr_infos, stdout, stdin)?
                }
            };

            let _ = match value {
                Value::Str(s) => stdout(format!("{}", s)),
                Value::Int(int) => stdout(format!("{}", int)),
                Value::None => stdout(format!("None")),
            };
        }

        Fun::If(expr1, expr2) => {
            let value1 = match expr1 {
                Expr::Value(v) => v,
                Expr::FunCall(_fun) => {
                    interpret_fun_call(_fun, labels, slots, instr_infos, stdout, stdin)?
                }
            };

            let condit = match value1 {
                Value::Str(s) => s != "",
                Value::Int(int) => int != 0,
                Value::None => false,
            };

            if condit {
                let value = match expr2 {
                    Expr::Value(v) => v,
                    Expr::FunCall(_fun) => {
                        interpret_fun_call(_fun, labels, slots, instr_infos, stdout, stdin)?
                    }
                };

                return Ok(value);
            }
        }

        Fun::Equal(expr1, expr2) => {
            let value1 = match expr1 {
                Expr::Value(v) => v,
                Expr::FunCall(_fun) => {
                    interpret_fun_call(_fun, labels, slots, instr_infos, stdout, stdin)?
                }
            };

            let value2 = match expr2 {
                Expr::Value(v) => v,
                Expr::FunCall(_fun) => {
                    interpret_fun_call(_fun, labels, slots, instr_infos, stdout, stdin)?
                }
            };

            match (&value1, &value2) {
                (Value::Int(int1), Value::Int(int2)) => {
                    if int1 == int2 {
                        return Ok(Value::Int(1));
                    } else {
                        return Ok(Value::Int(0));
                    }
                }
                (Value::Str(str1), Value::Str(str2)) => {
                    if str1 == str2 {
                        return Ok(Value::Int(1));
                    } else {
                        return Ok(Value::Int(0));
                    }
                }
                _ => {
                    return Err(Signal::Error(ErrorInfo::new(
                        Error::Error(format!(
                            "You are trying to compare {} and {} which is invalid",
                            &value1, &value2
                        )),
                        clone,
                        None,
                    )));
                }
            }
        }

        Fun::Extract(expr1, expr2) => {
            let value1 = match expr1 {
                Expr::Value(v) => v,
                Expr::FunCall(_fun) => {
                    interpret_fun_call(_fun, labels, slots, instr_infos, stdout, stdin)?
                }
            };

            let value2 = match expr2 {
                Expr::Value(v) => v,
                Expr::FunCall(_fun) => {
                    interpret_fun_call(_fun, labels, slots, instr_infos, stdout, stdin)?
                }
            };

            match (&value1, &value2) {
                (Value::Str(s), Value::Int(int)) => {
                    let chars = s.chars().collect::<Vec<char>>();
                    match chars.get(*int as usize) {
                        Some(c) => return Ok(Value::Str(c.to_string())),
                        None => return Ok(Value::Str("".to_string())),
                    }
                }
                _ => {
                    return Err(Signal::Error(ErrorInfo::new(
                        Error::Error(format!(
                            "You are trying to extract from {} using index value {} which is invalid",
                            &value1, &value2
                        )),
                        clone,
                        None,
                    )));
                }
            }
        }

        Fun::Text(expr) => {
            let value = match expr {
                Expr::Value(v) => v,
                Expr::FunCall(_fun) => {
                    interpret_fun_call(_fun, labels, slots, instr_infos, stdout, stdin)?
                }
            };

            match &value {
                Value::Int(int) => return Ok(Value::Str(int.to_string())),
                _ => {
                    return Err(Signal::Error(ErrorInfo::new(
                        Error::TypeError {
                            expected: Value::Int(0),
                            got: value.clone(),
                        },
                        clone,
                        None,
                    )))
                }
            }
        }

        Fun::Number(expr) => {
            let value = match expr {
                Expr::Value(v) => v,
                Expr::FunCall(_fun) => {
                    interpret_fun_call(_fun, labels, slots, instr_infos, stdout, stdin)?
                }
            };

            match &value {
                Value::Str(s) => match s.parse::<isize>() {
                    Ok(int) => return Ok(Value::Int(int)),
                    Err(_) => {
                        return Err(Signal::Error(ErrorInfo::new(
                            Error::ValueError(value.clone()),
                            clone,
                            Some(format!("Cannot convert {} to an Int", value)),
                        )))
                    }
                },

                _ => {
                    return Err(Signal::Error(ErrorInfo::new(
                        Error::TypeError {
                            expected: Value::Int(0),
                            got: value.clone(),
                        },
                        clone,
                        None,
                    )))
                }
            }
        }

        Fun::Convert(expr) => {
            let value = match expr {
                Expr::Value(v) => v,
                Expr::FunCall(_fun) => {
                    interpret_fun_call(_fun, labels, slots, instr_infos, stdout, stdin)?
                }
            };

            match value {
                Value::Str(ref s) => {
                    if s.len() != 1 {
                        return Err(Signal::Error(ErrorInfo::new(
                            Error::ValueError(value.clone()),
                            clone,
                            Some(format!(
                                "The Str should have exactly 1 char got {}",
                                s.len()
                            )),
                        )));
                    }

                    let ch = s.chars().next().unwrap();

                    return Ok(Value::Int(ch as isize));
                }

                Value::Int(int) => match char::from_u32(int as u32) {
                    Some(ch) => return Ok(Value::Str(ch.to_string())),
                    None => {
                        return Err(Signal::Error(ErrorInfo::new(
                            Error::ValueError(value),
                            clone,
                            Some(format!("Cannot convert Int {} to a char", int)),
                        )))
                    }
                },

                Value::None => {
                    return Err(Signal::Error(ErrorInfo::new(
                        Error::ValueError(value),
                        clone,
                        Some(format!("Cannot convert None value")),
                    )))
                }
            }
        }

        Fun::Return(expr) => {
            let value = match expr {
                Expr::Value(v) => v,
                Expr::FunCall(_fun) => {
                    interpret_fun_call(_fun, labels, slots, instr_infos, stdout, stdin)?
                }
            };

            return Err(Signal::Return(value));
        }

        Fun::Input => {
            let s = match stdin() {
                Ok(s) => s,
                Err(_) => {
                    return Err(Signal::Error(ErrorInfo::new(
                        Error::Error(format!("Failed to receive an input")),
                        clone,
                        None,
                    )))
                }
            };

            let input = s.trim().to_string();

            return Ok(Value::Str(input));
        }

        Fun::KeyChar => return Ok(Value::None),

        Fun::EmptySlot => {
            for n in 0..isize::MAX {
                if !slots.contains_key(&n) {
                    return Ok(Value::Int(n));
                }
            }

            return Err(Signal::Error(ErrorInfo::new(
                Error::NoSlotError,
                clone,
                Some(format!("At this point, you better use a known number")),
            )));
        }

        Fun::Exit => {
            exit(0);
        }
    }

    Ok(Value::None)
}
