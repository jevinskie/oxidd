//! Espresso PLA parser
//!
//! # https://people.eecs.berkeley.edu/~alanmi/research/espresso/espresso_5.html

// spell-checker:ignore multispace

use nom::error::{context, ContextError, FromExternalError, ParseError};
use nom::IResult;

use crate::{
    Circuit, GateKind, Literal, ParseOptions, Problem, ProblemDetails, Tree, Var, VarSet, Vec2d,
};

/// Parse a PLA file
pub fn parse<'a, E>(options: &ParseOptions) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Problem, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]> + FromExternalError<&'a [u8], String>,
{
    move |input| {
        Ok((
            input,
            Problem {
                circuit: Circuit::new(VarSet::new(0)),
                details: crate::ProblemDetails::Root(Literal::UNDEF),
            },
        ))
    }
}
