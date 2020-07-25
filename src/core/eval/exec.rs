use std::collections::HashMap;
use std::fmt;

use crate::core::eval::error::*;
use crate::core::eval::{ast, ast::ExpressionKind};

use std::convert::TryFrom;

use std::ops::{Add, Div, Mul, Sub};

use ExecutionExpr::*;

#[derive(Clone, Copy, PartialEq)]
enum ExecutionExpr {
    Integer(i128),
    Float(f64),
    Bool(bool),
}

impl ExecutionExpr {
    fn display_type(&self) -> &'static str {
        match self {
            ExecutionExpr::Integer(_) => "`integer`",
            ExecutionExpr::Float(_) => "`float`",
            ExecutionExpr::Bool(_) => "`boolean`",
        }
    }
}

impl fmt::Display for ExecutionExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ExecutionExpr::Integer(val) => val.to_string(),
                ExecutionExpr::Float(val) => val.to_string(),
                ExecutionExpr::Bool(val) => val.to_string(),
            }
        )
    }
}

#[derive(Clone, Copy)]
pub struct EE {
    value: ExecutionExpr,
    pos: Pos,
}

macro_rules! from_expr {
    ($expr: expr, $pos: expr) => {
        Ok(EE {
            value: $expr,
            pos: $pos,
        })
    };
}

impl EE {
    fn new(expr: ExecutionExpr, pos: Pos) -> Self {
        EE { value: expr, pos }
    }

    fn calc_pos(&self, other: &Self) -> Pos {
        Pos::new(self.pos.start, other.pos.end)
    }

    fn gen_type_err(&self, other: &Self, operation: &'static str) -> Error {
        Error::new(
            format!(
                "cannot apply operator `{}` on types {} and {}",
                operation,
                self.value.display_type(),
                other.value.display_type()
            ),
            ErrorType::TypeError,
            self.calc_pos(other),
        )
    }

    fn add(&self, other: &Self) -> Result<Self, Error> {
        from_expr!(
            match (&self.value, &other.value) {
                (Integer(left), Integer(right)) => Integer(match left.checked_add(*right) {
                    Some(val) => val,
                    None => {
                        return Err(Error::new(
                            format!("failed to add `{}` and `{}`: value overflowed", left, right),
                            ErrorType::RuntimeError,
                            self.calc_pos(other),
                        ));
                    }
                }),
                (Float(left), Float(right)) => Float(left.add(*right)),
                _ => return Err(self.gen_type_err(other, "add")),
            },
            self.calc_pos(other)
        )
    }

    fn sub(&self, other: &Self) -> Result<Self, Error> {
        from_expr!(
            match (&self.value, &other.value) {
                (Integer(left), Integer(right)) => Integer(match left.checked_sub(*right) {
                    Some(val) => val,
                    None => {
                        return Err(Error::new(
                            format!(
                                "failed to subtract `{}` from `{}`: value overflowed",
                                right, left
                            ),
                            ErrorType::RuntimeError,
                            self.calc_pos(other),
                        ));
                    }
                }),
                (Float(left), Float(right)) => Float(left.sub(*right)),
                _ => return Err(self.gen_type_err(other, "subtract")),
            },
            self.calc_pos(other)
        )
    }

    fn mul(&self, other: &Self) -> Result<Self, Error> {
        from_expr!(
            match (&self.value, &other.value) {
                (Integer(left), Integer(right)) => Integer(match left.checked_mul(*right) {
                    Some(val) => val,
                    None => {
                        return Err(Error::new(
                            format!("failed to multiply `{}` by `{}`: value overflowed", right, left),
                            ErrorType::RuntimeError,
                            self.calc_pos(other),
                        ));
                    }
                }),
                (Float(left), Float(right)) => Float(left.mul(*right)),
                _ => return Err(self.gen_type_err(other, "multiply")),
            },
            self.calc_pos(other)
        )
    }

    fn modulo(&self, other: &Self) -> Result<Self, Error> {
        from_expr!(
            match (&self.value, &other.value) {
                (Integer(left), Integer(right)) => Integer(*left % *right),
                (Float(left), Float(right)) => Float(*left % *right),
                _ => return Err(self.gen_type_err(other, "modulo")),
            },
            self.calc_pos(other)
        )
    }

    fn div(&self, other: &Self) -> Result<Self, Error> {
        from_expr!(
            match (&self.value, &other.value) {
                (Integer(left), Integer(right)) => Float((*left as f64).div(*right as f64)),
                (Float(left), Float(right)) => Float(left.div(*right)),
                _ => return Err(self.gen_type_err(other, "divide")),
            },
            self.calc_pos(other)
        )
    }

    fn int_div(&self, other: &Self) -> Result<Self, Error> {
        from_expr!(
            match (&self.value, &other.value) {
                (Integer(left), Integer(right)) => Integer(match left.checked_div(*right) {
                    Some(val) => val,
                    None => {
                        return Err(Error::new(
                            format!(
                                "failed to integer divide `{}` by `{}`: value overflowed",
                                right, left
                            ),
                            ErrorType::RuntimeError,
                            self.calc_pos(other),
                        ));
                    }
                }),
                (Float(left), Float(right)) => {
                    Integer(match (*left as i128).checked_div(*right as i128) {
                        Some(val) => val,
                        None => {
                            return Err(Error::new(
                                format!(
                                    "failed to integer divide `{}` by {}: value overflowed",
                                    right, left
                                ),
                                ErrorType::RuntimeError,
                                self.calc_pos(other),
                            ))
                        }
                    })
                }
                _ => return Err(self.gen_type_err(other, "integer divide")),
            },
            self.calc_pos(other)
        )
    }

    fn pow(&self, other: &Self) -> Result<Self, Error> {
        from_expr!(
            match (&self.value, &other.value) {
                (Integer(left), Integer(right)) => Integer(
                    match left.checked_pow(match u32::try_from(*right) {
                        Ok(val) => val,
                        Err(why) => {
                            return Err(Error::new(
                                format!(
                                    "failed to raise `{}` to the power of `{}`: {}",
                                    left, right, why
                                ),
                                ErrorType::RuntimeError,
                                self.calc_pos(other),
                            ));
                        }
                    }) {
                        Some(val) => val,
                        None => {
                            return Err(Error::new(
                                format!(
                                    "failed to raise `{}` to the power of `{}`: value overflowed",
                                    right, left
                                ),
                                ErrorType::RuntimeError,
                                self.calc_pos(other),
                            ));
                        }
                    },
                ),
                (Float(left), Float(right)) => Float(left.powf(*right)),
                _ => return Err(self.gen_type_err(other, "power")),
            },
            self.calc_pos(other)
        )
    }

    fn lnot(&self) -> Result<Self, Error> {
        from_expr!(
            match &self.value {
                Bool(val) => Bool(!val),
                _ =>
                    return Err(Error::new(
                        format!("cannot logically negate `{}`", self.value.display_type()),
                        ErrorType::RuntimeError,
                        self.pos
                    )),
            },
            self.pos
        )
    }

    fn land(&self, other: &Self) -> Result<Self, Error> {
        from_expr!(
            match (&self.value, &other.value) {
                (Bool(left), Bool(right)) => Bool(*left && *right),
                _ => return Err(self.gen_type_err(other, "logical and")),
            },
            self.pos
        )
    }

    fn lor(&self, other: &Self) -> Result<Self, Error> {
        from_expr!(
            match (&self.value, &other.value) {
                (Bool(left), Bool(right)) => Bool(*left || *right),
                _ => return Err(self.gen_type_err(other, "logical or")),
            },
            self.pos
        )
    }

    fn band(&self, other: &Self) -> Result<Self, Error> {
        from_expr!(
            match (&self.value, &other.value) {
                (Integer(left), Integer(right)) => Integer(*left & *right),
                _ => return Err(self.gen_type_err(other, "bitwise and")),
            },
            self.pos
        )
    }

    fn bor(&self, other: &Self) -> Result<Self, Error> {
        from_expr!(
            match (&self.value, &other.value) {
                (Integer(left), Integer(right)) => Integer(*left | *right),
                _ => return Err(self.gen_type_err(other, "bitwise or")),
            },
            self.pos
        )
    }

    fn bxor(&self, other: &Self) -> Result<Self, Error> {
        from_expr!(
            match (&self.value, &other.value) {
                (Integer(left), Integer(right)) => Integer(*left ^ *right),
                _ => return Err(self.gen_type_err(other, "bitwise xor")),
            },
            self.pos
        )
    }

    fn bnot(&self) -> Result<Self, Error> {
        from_expr!(
            match &self.value {
                Integer(val) => Integer(!val),
                _ =>
                    return Err(Error::new(
                        format!("cannot bitwise negate `{}`", self.value.display_type()),
                        ErrorType::RuntimeError,
                        self.pos
                    )),
            },
            self.pos
        )
    }


    fn bitshift_l(&self, other: &Self) -> Result<Self, Error> {
        from_expr!(
            match (&self.value, &other.value) {
                (Integer(left), Integer(right)) => {
                    match left.checked_shl(match u32::try_from(*right) {
                        Ok(val) => val,
                        Err(why) => {
                            return Err(Error::new(
                                format!("failed to bitshift `{}` left `{}`: {}", left, right, why),
                                ErrorType::RuntimeError,
                                self.calc_pos(other),
                            ));
                        }
                    }) {
                        Some(val) => Integer(val),
                        None => {
                            return Err(Error::new(
                                format!(
                                    "failed to bitshift `{}` left by `{}`: overflowed",
                                    left, right
                                ),
                                ErrorType::RuntimeError,
                                self.calc_pos(other),
                            ))
                        }
                    }
                }
                _ => return Err(self.gen_type_err(other, "left bitshift")),
            },
            self.calc_pos(other)
        )
    }

    fn bitshift_r(&self, other: &Self) -> Result<Self, Error> {
        from_expr!(
            match (&self.value, &other.value) {
                (Integer(left), Integer(right)) => {
                    match left.checked_shr(match u32::try_from(*right) {
                        Ok(val) => val,
                        Err(why) => {
                            return Err(Error::new(
                                format!("failed to bitshift `{}` right `{}`: {}", left, right, why),
                                ErrorType::RuntimeError,
                                self.calc_pos(other),
                            ));
                        }
                    }) {
                        Some(val) => Integer(val),
                        None => {
                            return Err(Error::new(
                                format!(
                                    "failed to bitshift `{}` right by `{}`: overflowed",
                                    left, right
                                ),
                                ErrorType::RuntimeError,
                                self.calc_pos(other),
                            ))
                        }
                    }
                }
                _ => return Err(self.gen_type_err(other, "right bitshift ")),
            },
            self.calc_pos(other)
        )
    }

    fn as_cast<'a>(&self, target_type: ast::Expression<'a>) -> Result<Self, Error> {
        from_expr!(
            match target_type.expr {
                ExpressionKind::Ident(tok) => match tok {
                    "float" => match self.value {
                        Integer(val) => Float(
                            match f64::try_from(match i32::try_from(val) {
                                Ok(val) => val,
                                Err(why) =>
                                    return Err(Error::new(
                                        format!(
                                            "failed to convert `{}` to `{}`: {}",
                                            self.value, tok, why
                                        ),
                                        ErrorType::RuntimeError,
                                        self.pos,
                                    )),
                            }) {
                                Ok(val) => val,
                                Err(why) =>
                                    return Err(Error::new(
                                        format!(
                                            "failed to convert `{}` to `{}`: {}",
                                            self.value, tok, why
                                        ),
                                        ErrorType::RuntimeError,
                                        self.pos,
                                    )),
                            }
                        ),
                        Float(_) => self.value,
                        Bool(val) => Float(val as i8 as f64),
                    },
                    "int" => match self.value {
                        Integer(_) => self.value,
                        Float(val) => Integer(val.round() as i128),
                        Bool(val) => Integer(val as i128),
                    },
                    _ =>
                        return Err(Error::new(
                            format!("unknown type `{}`", tok),
                            ErrorType::TypeError,
                            target_type.pos
                        )),
                },
                _ => {
                    return Err(Error::new(
                        format!("invalid type for `as` type operand `{}`", target_type.expr),
                        ErrorType::TypeError,
                        target_type.pos,
                    ));
                }
            },
            self.pos
        )
    }

    fn neg(&self) -> Result<Self, Error> {
        from_expr!(
            match &self.value {
                Integer(val) => Integer(-val),
                Float(val) => Float(-val),
                _ => {
                    return Err(Error::new(
                        format!("cannot make type {} negative", self.value.display_type()),
                        ErrorType::TypeError,
                        self.pos,
                    ));
                }
            },
            self.pos
        )
    }

    fn pos(&self) -> Result<Self, Error> {
        from_expr!(
            match &self.value {
                Integer(val) => Integer(val.abs()),
                Float(val) => Float(val.abs()),
                _ => {
                    return Err(Error::new(
                        format!("cannot make type {} positive", self.value.display_type()),
                        ErrorType::TypeError,
                        self.pos,
                    ));
                }
            },
            self.pos
        )
    }

    fn eql(&self, other: &Self) -> Result<Self, Error> {
        from_expr!(Bool(self.value == other.value), self.calc_pos(other))
    }

    fn neql(&self, other: &Self) -> Result<Self, Error> {
        from_expr!(Bool(self.value != other.value), self.calc_pos(other))
    }

    fn lt(&self, other: &Self) -> Result<Self, Error> {
        from_expr!(
            match (&self.value, &other.value) {
                (Integer(left), Integer(right)) => Bool(left < right),
                (Float(left), Float(right)) => Bool(left < right),
                _ => return Err(self.gen_type_err(other, "less than")),
            },
            self.calc_pos(other)
        )
    }

    fn gt(&self, other: &Self) -> Result<Self, Error> {
        from_expr!(
            match (&self.value, &other.value) {
                (Integer(left), Integer(right)) => Bool(left > right),
                (Float(left), Float(right)) => Bool(left > right),
                _ => return Err(self.gen_type_err(other, "greater than")),
            },
            self.calc_pos(other)
        )
    }

    fn lte(&self, other: &Self) -> Result<Self, Error> {
        from_expr!(
            match (&self.value, &other.value) {
                (Integer(left), Integer(right)) => Bool(left <= right),
                (Float(left), Float(right)) => Bool(left <= right),
                _ => return Err(self.gen_type_err(other, "less than")),
            },
            self.calc_pos(other)
        )
    }

    fn gte(&self, other: &Self) -> Result<Self, Error> {
        from_expr!(
            match (&self.value, &other.value) {
                (Integer(left), Integer(right)) => Bool(left >= right),
                (Float(left), Float(right)) => Bool(left >= right),
                _ => return Err(self.gen_type_err(other, "greater than")),
            },
            self.calc_pos(other)
        )
    }

}

impl fmt::Display for EE {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

pub struct Executer<'a> {
    symbtab: HashMap<&'a str, EE>,
}

macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key, EE::new(Float($value), Pos::new(0, 0)));
            )+
            m
        }
     };
);

use std::f64::consts;
impl<'a> Executer<'a> {
    pub(crate) fn new() -> Self {
        Executer {
            symbtab: map!(
                "pi" => consts::PI,
                "tau" => consts::PI * 2.0,
                "e" => consts::E,
                "sqrt2" => consts::SQRT_2
            ),
        }
    }

    pub(crate) fn eval(&mut self, ast: ast::Expression<'a>) -> Result<EE, Error> {
        Ok(match ast.expr {
            ExpressionKind::True => EE::new(Bool(true), ast.pos),
            ExpressionKind::False => EE::new(Bool(false), ast.pos),

            ExpressionKind::Integer(val) => EE::new(
                ExecutionExpr::Integer(match val.parse::<i128>() {
                    Ok(val) => val,
                    Err(why) => {
                        return Err(Error::new(
                            format!("error converting `{}` to integer: {}", val, why),
                            ErrorType::RuntimeError,
                            ast.pos,
                        ))
                    }
                }),
                ast.pos,
            ),

            ExpressionKind::Float(val) => EE::new(
                ExecutionExpr::Float(match val.parse::<f64>() {
                    Ok(val) => val,
                    Err(why) => {
                        return Err(Error::new(
                            format!("error converting `{}` to float: {}", val, why),
                            ErrorType::RuntimeError,
                            ast.pos,
                        ))
                    }
                }),
                ast.pos,
            ),

            // Where all the magic happens
            ExpressionKind::InfixOp(val) => match val.op {
                ast::Operator::Add => self.eval(*val.left)?.add(&self.eval(*val.right)?)?,
                ast::Operator::Sub => self.eval(*val.left)?.sub(&self.eval(*val.right)?)?,
                ast::Operator::Mul => self.eval(*val.left)?.mul(&self.eval(*val.right)?)?,
                ast::Operator::Div => self.eval(*val.left)?.div(&self.eval(*val.right)?)?,

                ast::Operator::Mod => self.eval(*val.left)?.modulo(&self.eval(*val.right)?)?,

                ast::Operator::IntDiv => self.eval(*val.left)?.int_div(&self.eval(*val.right)?)?,
                ast::Operator::Pow => self.eval(*val.left)?.pow(&self.eval(*val.right)?)?,

                ast::Operator::As => self.eval(*val.left)?.as_cast(*val.right)?,

                ast::Operator::BitShiftL => {
                    self.eval(*val.left)?.bitshift_l(&self.eval(*val.right)?)?
                }
                ast::Operator::BitShiftR => {
                    self.eval(*val.left)?.bitshift_r(&self.eval(*val.right)?)?
                }

                ast::Operator::LNot => self.eval(*val.left)?.lnot()?,
                ast::Operator::LOr => self.eval(*val.left)?.lor(&self.eval(*val.right)?)?,
                ast::Operator::LAnd => self.eval(*val.left)?.land(&self.eval(*val.right)?)?,
                
                ast::Operator::BOr => self.eval(*val.left)?.bor(&self.eval(*val.right)?)?,
                ast::Operator::BAnd => self.eval(*val.left)?.band(&self.eval(*val.right)?)?,
                ast::Operator::BXor => self.eval(*val.left)?.bxor(&self.eval(*val.right)?)?,

                ast::Operator::NEql => self.eval(*val.left)?.neql(&self.eval(*val.right)?)?,
                ast::Operator::Eql => self.eval(*val.left)?.eql(&self.eval(*val.right)?)?,

                ast::Operator::LT => self.eval(*val.left)?.lt(&self.eval(*val.right)?)?,
                ast::Operator::LE => self.eval(*val.left)?.lte(&self.eval(*val.right)?)?,
                ast::Operator::GT => self.eval(*val.left)?.gt(&self.eval(*val.right)?)?,
                ast::Operator::GE => self.eval(*val.left)?.gte(&self.eval(*val.right)?)?,

                _ => {
                    return Err(Error::new(
                        format!("infix {} not implemented yet", val.op),
                        ErrorType::TypeError,
                        ast.pos,
                    ))
                }
            },

            ExpressionKind::PrefixOp(val) => match val.op {
                ast::Operator::Sub => self.eval(*val.value)?.neg()?,
                ast::Operator::Add => self.eval(*val.value)?.pos()?,
                ast::Operator::BNot => self.eval(*val.value)?.bnot()?,
                ast::Operator::LNot => self.eval(*val.value)?.lnot()?,
                _ => {
                    return Err(Error::new(
                        format!("prefix {} not implemented yet", val.op),
                        ErrorType::TypeError,
                        ast.pos,
                    ))
                }
            },

            ExpressionKind::Ident(val) => match self.symbtab.get(val) {
                Some(val) => *val,
                None => {
                    return Err(Error::new(
                        format!("no variable `{}` found", val),
                        ErrorType::RuntimeError,
                        ast.pos,
                    ))
                }
            },
        })
    }
}
