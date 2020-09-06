use nom::sequence::delimited;
use nom::IResult;

pub enum ExprValue<'a> {
    Number(i32),
    Label(&'a str),
}

pub enum NumericExpr<'a> {
    Value(ExprValue<'a>),
    Add(Box<NumericExpr<'a>>, Box<NumericExpr<'a>>),
    Subtract(Box<NumericExpr<'a>>, Box<NumericExpr<'a>>),
    Multiply(Box<NumericExpr<'a>>, Box<NumericExpr<'a>>),
    Divide(Box<NumericExpr<'a>>, Box<NumericExpr<'a>>),
    Modulo(Box<NumericExpr<'a>>, Box<NumericExpr<'a>>),
    Paren(Box<NumericExpr<'a>>),
}

pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

fn parens(i: &str) -> IResult<&str, NumericExpr> {
    todo!()
}
