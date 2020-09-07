use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, not_line_ending, space0, space1},
    combinator::map,
    sequence::preceded,
    sequence::tuple,
    IResult,
};

#[derive(PartialEq, Eq, Debug)]
pub(crate) enum MetadataValue<'a> {
    Author(&'a str),
    Date(&'a str),
    Strategy(&'a str),
    Version(&'a str),
    Name(&'a str),
}

macro_rules! metadata_comment {
    ($tagtype:literal, $variant:path) => {
        map(
            preceded(
                tuple((space0, char(';'), space0, tag($tagtype), space1)),
                not_line_ending,
            ),
            |v: &str| $variant(v.trim()),
        )
    };
}

fn metadata(i: &str) -> IResult<&str, MetadataValue> {
    alt((
        metadata_comment!("strategy", MetadataValue::Strategy),
        metadata_comment!("name", MetadataValue::Name),
        metadata_comment!("author", MetadataValue::Author),
        metadata_comment!("date", MetadataValue::Date),
        metadata_comment!("version", MetadataValue::Version),
    ))(i)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_metadata() {
        let (_, res) = metadata(";strategy Hit it until it stops moving").unwrap();
        assert_eq!(res, MetadataValue::Strategy("Hit it until it stops moving"));

        let (_, res) = metadata("  ; author    Daniel Rivas Perez").unwrap();
        assert_eq!(res, MetadataValue::Author("Daniel Rivas Perez"));

        let (_, res) = metadata(";name Hittite    ").unwrap();
        assert_eq!(res, MetadataValue::Name("Hittite"));
    }
}
