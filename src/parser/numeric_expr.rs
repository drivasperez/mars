use super::instruction::label;
use crate::error::EvaluateError;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, one_of, space0},
    combinator::{map, opt, recognize},
    multi::many0,
    sequence::{delimited, pair, preceded},
    IResult,
};
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Display};

#[derive(Eq, PartialEq)]
pub(crate) enum ExprValue<'a> {
    Number(i32),
    Label(&'a str),
}

impl Display for ExprValue<'_> {
    fn fmt(&self, format: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Number(n) => write!(format, "{}", n),
            Self::Label(l) => write!(format, "{}", l),
        }
    }
}

impl Debug for ExprValue<'_> {
    fn fmt(&self, format: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Number(n) => write!(format, "{:?}", n),
            Self::Label(l) => write!(format, "{:?}", l),
        }
    }
}

#[derive(Eq, PartialEq)]
pub(crate) enum NumericExpr<'a> {
    Value(ExprValue<'a>),
    Add(Box<NumericExpr<'a>>, Box<NumericExpr<'a>>),
    Subtract(Box<NumericExpr<'a>>, Box<NumericExpr<'a>>),
    Multiply(Box<NumericExpr<'a>>, Box<NumericExpr<'a>>),
    Divide(Box<NumericExpr<'a>>, Box<NumericExpr<'a>>),
    Modulo(Box<NumericExpr<'a>>, Box<NumericExpr<'a>>),
    Paren(Box<NumericExpr<'a>>),
}
impl Debug for NumericExpr<'_> {
    fn fmt(&self, format: &mut fmt::Formatter) -> fmt::Result {
        use self::NumericExpr::*;

        match *self {
            Value(ref val) => write!(format, "{:?}", val),
            Add(ref left, ref right) => write!(format, "{:?} + {:?}", left, right),
            Subtract(ref left, ref right) => write!(format, "{:?} - {:?}", left, right),
            Multiply(ref left, ref right) => write!(format, "{:?} * {:?}", left, right),
            Divide(ref left, ref right) => write!(format, "{:?} / {:?}", left, right),
            Modulo(ref left, ref right) => write!(format, "{:?} % {:?}", left, right),
            Paren(ref expr) => write!(format, "[{:?}]", expr),
        }
    }
}
impl Display for NumericExpr<'_> {
    fn fmt(&self, format: &mut fmt::Formatter) -> fmt::Result {
        use self::NumericExpr::*;

        match *self {
            Value(ref val) => write!(format, "{}", val),
            Add(ref left, ref right) => write!(format, "{} + {}", left, right),
            Subtract(ref left, ref right) => write!(format, "{} - {}", left, right),
            Multiply(ref left, ref right) => write!(format, "{} * {}", left, right),
            Divide(ref left, ref right) => write!(format, "{} / {}", left, right),
            Modulo(ref left, ref right) => write!(format, "{} % {}", left, right),
            Paren(ref expr) => write!(format, "({})", expr),
        }
    }
}

impl NumericExpr<'_> {
    pub(crate) fn evaluate(&self, labels: &HashMap<&str, i32>) -> Result<i32, EvaluateError> {
        let res: i32 = match self {
            Self::Value(val) => match val {
                ExprValue::Number(n) => *n,
                ExprValue::Label(l) => *labels
                    .get(l)
                    .ok_or_else(|| EvaluateError::UndefinedLabel(String::from(*l)))?,
            },

            Self::Paren(ref val) => val.evaluate(labels)?,
            Self::Add(ref left, ref right) => left.evaluate(labels)? + right.evaluate(labels)?,
            Self::Subtract(ref left, ref right) => {
                left.evaluate(labels)? - right.evaluate(labels)?
            }
            Self::Multiply(ref left, ref right) => {
                left.evaluate(labels)? * right.evaluate(labels)?
            }
            Self::Divide(ref left, ref right) => left
                .evaluate(labels)?
                .checked_div(right.evaluate(labels)?)
                .ok_or(EvaluateError::DivideByZero)?,
            Self::Modulo(ref left, ref right) => left.evaluate(labels)? % right.evaluate(labels)?,
        };

        Ok(res)
    }

    pub(crate) fn evaluate_relative(
        &self,
        labels: &HashMap<&str, i32>,
        current_line: i32,
    ) -> Result<i32, EvaluateError> {
        let abs_value = self.evaluate(labels)?;
        Ok(abs_value - current_line)
    }
}

pub(crate) enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

fn number(i: &str) -> IResult<&str, i32> {
    map(recognize(pair(opt(one_of("+-")), digit1)), |num: &str| {
        num.parse().unwrap()
    })(i)
}

fn parens(i: &str) -> IResult<&str, NumericExpr> {
    delimited(
        space0,
        delimited(
            tag("("),
            map(expr, |e| NumericExpr::Paren(Box::new(e))),
            tag(")"),
        ),
        space0,
    )(i)
}

fn factor(i: &str) -> IResult<&str, NumericExpr> {
    alt((
        map(delimited(space0, number, space0), |v| {
            NumericExpr::Value(ExprValue::Number(v))
        }),
        map(delimited(space0, label, space0), |v| {
            NumericExpr::Value(ExprValue::Label(v))
        }),
        parens,
    ))(i)
}

fn fold_exprs<'a>(
    initial: NumericExpr<'a>,
    remainder: Vec<(Operation, NumericExpr<'a>)>,
) -> NumericExpr<'a> {
    remainder.into_iter().fold(initial, |acc, pair| {
        let (oper, expr) = pair;
        match oper {
            Operation::Add => NumericExpr::Add(Box::new(acc), Box::new(expr)),
            Operation::Subtract => NumericExpr::Subtract(Box::new(acc), Box::new(expr)),
            Operation::Multiply => NumericExpr::Multiply(Box::new(acc), Box::new(expr)),
            Operation::Divide => NumericExpr::Divide(Box::new(acc), Box::new(expr)),
            Operation::Modulo => NumericExpr::Modulo(Box::new(acc), Box::new(expr)),
        }
    })
}

fn term(i: &str) -> IResult<&str, NumericExpr> {
    let (i, initial) = factor(i)?;
    let (i, remainder) = many0(alt((
        |i| {
            let (i, mul) = preceded(tag("*"), factor)(i)?;
            Ok((i, (Operation::Multiply, mul)))
        },
        |i| {
            let (i, div) = preceded(tag("/"), factor)(i)?;
            Ok((i, (Operation::Divide, div)))
        },
        |i| {
            let (i, modulo) = preceded(tag("%"), factor)(i)?;
            Ok((i, (Operation::Modulo, modulo)))
        },
    )))(i)?;

    Ok((i, fold_exprs(initial, remainder)))
}

pub(crate) fn expr(i: &str) -> IResult<&str, NumericExpr> {
    let (i, initial) = term(i)?;
    let (i, remainder) = many0(alt((
        |i| {
            let (i, add) = preceded(tag("+"), term)(i)?;
            Ok((i, (Operation::Add, add)))
        },
        |i| {
            let (i, sub) = preceded(tag("-"), term)(i)?;
            Ok((i, (Operation::Subtract, sub)))
        },
    )))(i)?;

    Ok((i, fold_exprs(initial, remainder)))
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn parse_number() {
        let res = number("322");
        assert_eq!(res, Ok(("", 322)));

        let res = number("+9");
        assert_eq!(res, Ok(("", 9)));

        let res = number("-545");
        assert_eq!(res, Ok(("", -545)));

        let res = number("-545abc");
        assert_eq!(res, Ok(("abc", -545)));

        let res = number("323.32");
        assert_eq!(res, Ok((".32", 323)));

        let res = number("u323");
        assert!(res.is_err());
    }

    #[test]
    fn parse_expression() {
        assert_eq!(
            format!("{}", expr("3 * 4 + 4").unwrap().1),
            String::from("3 * 4 + 4")
        );
        assert_eq!(
            format!("{}", expr("tag * 4 + 4").unwrap().1),
            String::from("tag * 4 + 4")
        );
        assert_eq!(
            format!("{}", expr("3 * tag + 4").unwrap().1),
            String::from("3 * tag + 4")
        );
        assert_eq!(
            format!("{}", expr("3 * (tag + 4)").unwrap().1),
            String::from("3 * (tag + 4)")
        );

        assert_eq!(
            format!(
                "{}",
                expr("3 *      12 /   (4   + 5 + variable) % (tag + 4)")
                    .unwrap()
                    .1
            ),
            String::from("3 * 12 / (4 + 5 + variable) % (tag + 4)")
        );
    }

    #[test]
    fn evaluate_expression() {
        let labels: HashMap<&str, i32> = vec![("hello", 33), ("world", -2)].into_iter().collect();

        assert_eq!(expr("3 + 5").unwrap().1.evaluate(&labels).unwrap(), 8);
        assert_eq!(expr("3 + -5").unwrap().1.evaluate(&labels).unwrap(), -2);
        assert_eq!(expr("3 + 5 * 2").unwrap().1.evaluate(&labels).unwrap(), 13);
        assert_eq!(
            expr("3 + hello * 2").unwrap().1.evaluate(&labels).unwrap(),
            69
        );
        assert!(expr("8 / 0").unwrap().1.evaluate(&labels).is_err())
    }

    #[test]
    fn evaluate_relative_expression() {
        let labels: HashMap<&str, i32> = vec![("hello", 33), ("world", -2)].into_iter().collect();

        assert_eq!(
            expr("3 + 5")
                .unwrap()
                .1
                .evaluate_relative(&labels, 5)
                .unwrap(),
            3
        );
    }
}
