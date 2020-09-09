use crate::error::ParseError;
use std::borrow::Cow;

pub(crate) mod instruction;
pub(crate) mod line;
pub(crate) mod metadata;
pub(crate) mod numeric_expr;

use line::{lines, Line};

pub(crate) fn parse(i: &str) -> Result<Vec<Line>, ParseError> {
    let (_, ls) = lines(i).map_err(|e| match e {
        nom::Err::Incomplete(_) => ParseError::Incomplete,
        nom::Err::Error((_, ek)) | nom::Err::Failure((_, ek)) => ParseError::Parse(ek),
    })?;

    Ok(ls)
}

pub(crate) fn replace_definitions(s: &str) -> Result<Cow<str>, ParseError> {
    let mut val = Cow::from(s);
    let (_, ls) = lines(s).map_err(|_| ParseError::Replace)?;

    for line in ls {
        if let Line::Definition {
            label,
            definition,
            full_definition,
        } = line
        {
            val = Cow::from(
                val.to_mut()
                    .replace(full_definition, "")
                    .replace(label, definition.trim()),
            );
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
        assert_eq!(
            replaced,
            warrior
                .replace("step    EQU      4                 ", "")
                .replace("step", "4")
        );
        lines(&replaced).unwrap();
    }

    #[test]
    fn test_bad_dwarf() {
        let warrior = include_str!("../../warriors/bad_dwarf.red");
        lines(&warrior).unwrap_err();
    }

    #[test]
    fn test_one_line_dwarf() {
        let warrior = include_str!("../../warriors/one_line_dwarf.red");
        lines(&warrior).unwrap_err();
    }
}
