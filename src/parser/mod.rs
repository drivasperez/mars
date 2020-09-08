use crate::error::ParseError;
use std::borrow::Cow;

pub mod metadata;
pub mod numeric_expr;
pub mod operation;

use operation::{lines, Line};

pub(crate) fn parse(i: &str) -> Result<Vec<Line>, ParseError> {
    let (_, ls) = lines(i).map_err(|e| match e {
        nom::Err::Incomplete(_) => ParseError::Incomplete,
        nom::Err::Error((_, ek)) | nom::Err::Failure((_, ek)) => ParseError::Parse(ek),
    })?;

    Ok(ls)
}

pub(crate) fn replace_definitions<'a>(s: &'a str) -> Result<Cow<str>, ParseError> {
    let mut val = Cow::from(s);
    let (_, ls) = lines(s).map_err(|_| ParseError::Replace)?;

    for line in ls {
        if let Line::Definition(label, def) = line {
            val = Cow::Owned(val.to_mut().replace(label, def.trim()));
        }
    }

    Ok(val)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_replace_definitions() {
        let warrior = include_str!("../../warriors/dwarf.red");
        let replaced = replace_definitions(warrior).unwrap();
        assert_eq!(replaced, warrior.replace("step", "4"));
        assert!(lines(&replaced).is_ok());
    }
}
