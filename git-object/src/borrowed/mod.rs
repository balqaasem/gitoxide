use crate::Time;
use bstr::BStr;
use quick_error::quick_error;
use std::str;

pub(crate) mod commit;
pub(crate) mod tag;
pub(crate) mod tree;
pub(crate) mod util;

pub use commit::Commit;
use nom::error::ParseError;
pub use tag::Tag;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        ParseIntegerError(msg: &'static str, number: bstr::BString, err: btoi::ParseIntegerError) {
            display("{}: {:?}", msg, number)
            cause(err)
        }
        Nom(err_msg: String) {
            display("{}", err_msg)
        }
        NomDetail(input: bstr::BString, msg: &'static str) {
            display("{}: '{}' could not be parsed", msg, input)
        }
        ParseKindError(err: crate::types::Error) {
            display("{}", err)
            cause(err)
        }
        ObjectKind(err: crate::Error) {
            from()
            cause(err)
        }
    }
}

impl Error {
    fn set_parse_context(mut self, ctx: &'static str) -> Self {
        match self {
            Error::NomDetail(_, ref mut message) => *message = ctx,
            _ => {}
        };
        self
    }

    fn context(msg: &'static str) -> impl Fn(nom::Err<Self>) -> nom::Err<Self> {
        move |e: nom::Err<Self>| e.map(|e| e.set_parse_context(msg))
    }
}

impl ParseError<&[u8]> for Error {
    fn from_error_kind(input: &[u8], _kind: nom::error::ErrorKind) -> Self {
        Error::NomDetail(input.into(), "parse error")
    }

    fn append(_: &[u8], _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl From<nom::Err<Error>> for Error {
    fn from(e: nom::Err<Error>) -> Self {
        match e {
            nom::Err::Error(err) | nom::Err::Failure(err) => Error::Nom(err.to_string()),
            nom::Err::Incomplete(_) => unreachable!("we do not implement streaming parsers"),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Hash)]
pub enum Object<'data> {
    Tag(Tag<'data>),
    Commit(Commit<'data>),
}

mod convert {
    use crate::borrowed::{Commit, Object, Tag};
    use std::convert::TryFrom;

    impl<'data> Object<'data> {
        pub fn kind(&self) -> crate::Kind {
            match self {
                Object::Tag(_) => crate::Kind::Tag,
                Object::Commit(_) => crate::Kind::Commit,
            }
        }
    }

    impl<'data> From<Tag<'data>> for Object<'data> {
        fn from(v: Tag<'data>) -> Self {
            Object::Tag(v)
        }
    }

    impl<'data> From<Commit<'data>> for Object<'data> {
        fn from(v: Commit<'data>) -> Self {
            Object::Commit(v)
        }
    }

    impl<'data> TryFrom<Object<'data>> for Tag<'data> {
        type Error = Object<'data>;

        fn try_from(value: Object<'data>) -> Result<Self, Self::Error> {
            Ok(match value {
                Object::Tag(v) => v,
                _ => return Err(value),
            })
        }
    }

    impl<'data> TryFrom<Object<'data>> for Commit<'data> {
        type Error = Object<'data>;

        fn try_from(value: Object<'data>) -> Result<Self, Self::Error> {
            Ok(match value {
                Object::Commit(v) => v,
                _ => return Err(value),
            })
        }
    }
}

#[derive(PartialEq, Eq, Debug, Hash)]
pub struct Signature<'data> {
    pub name: &'data BStr,
    pub email: &'data BStr,
    pub time: Time,
}
