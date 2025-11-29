//! Espresso PLA parser
//!
//! # https://people.eecs.berkeley.edu/~alanmi/research/espresso/espresso_5.html

// spell-checker:ignore multispace

use std::io::Error;
use std::result::Result;
use std::sync::Arc;

use espresso_logic::{Cover, PLAReader, PLAWriter};
use nom::error::{context, ContextError, ErrorKind, FromExternalError, ParseError};
use nom::IResult;

use crate::util::{
    self, collect, context_loc, eol, fail, fail_with_contexts, line_span, usize, word_span,
    MAX_CAPACITY,
};
use crate::{
    Circuit, GateKind, Literal, ParseOptions, Problem, ProblemDetails, Tree, Var, VarSet, Vec2d,
};

/// Parse a PLA file
pub fn parse<'a, E>(options: &ParseOptions) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Problem, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]> + FromExternalError<&'a [u8], String>,
{
    move |input| {
        let pla_str = match str::from_utf8(input) {
            Ok(s) => s,
            Err(_) => return fail(input, "bad utf8"),
        };
        let cvr = match Cover::from_pla_string(pla_str) {
            Ok(c) => c,
            Err(_) => return fail(input, "bad pla"),
        };
        let ni = cvr.num_inputs();
        let no = cvr.num_outputs();
        let ilb = Vec::from_iter(cvr.input_labels().iter().map(move |l| l.as_ref()));
        let olb = Vec::from_iter(cvr.output_labels().iter().map(move |l| l.as_ref()));
        let mut vs = VarSet::new(ni);
        if ilb.len() > 0 {
            assert!(ilb.len() == ni);
            for i in 0..ni {
                vs.set_name(i, ilb[i]);
            }
        }

        let mut circt = Circuit::new(vs);

        let mut implicants: Vec<Literal> = vec![];

        for mt in cvr.cubes() {
            let prod = circt.push_gate(GateKind::And);
            implicants.push(prod);

            let mt_in_lits: Vec<Literal> = mt
                .inputs()
                .iter()
                .enumerate()
                .filter_map(|(i, ib)| match ib {
                    Some(true) => Some(Literal::from_input(false, i)),
                    Some(false) => Some(Literal::from_input(true, i)),
                    None => None,
                })
                .collect();
            circt.push_gate_inputs(mt_in_lits);

            // TODO: Multibit output
            // let mt_out_lits: Vec<Literal> = mt
            //     .outputs()
            //     .iter()
            //     .map(|ob| match ob {
            //         true => Literal::TRUE,
            //         false => Literal::FALSE,
            //     })
            //     .collect();
        }

        let sum = circt.push_gate(GateKind::Or);
        circt.push_gate_inputs(implicants);

        Ok((
            &[],
            Problem {
                circuit: circt,
                details: crate::ProblemDetails::Root(sum),
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use nom::Finish;
    use nom::Parser;
    use pretty_assertions::{assert_eq, assert_ne};

    use crate::{util::test::*, Gate};

    use super::*;

    /// PLA tweaked majority-2
    #[test]
    fn pla_example() {
        let input = b".i 4\n\
            .o 1\n\
            .ilb A B C D\n\
            .ob MAJ\n\
            .p 4\n\
            11-- 1\n\
            1-1- 1\n\
            1--1 1\n\
            -111 1\n\
            .e\n";

        let (input, problem) = parse::<()>(&OPTS_NO_ORDER).parse(input).finish().unwrap();
        assert!(input.is_empty());

        let (circuit, root) = unwrap_problem(problem);
        let inputs = circuit.inputs();
        assert_eq!(inputs.len(), 4);
        assert!(inputs.order().is_none());

        #[rustfmt::skip]
        let nodes = &[
            g(0), v(0), v(1),
            g(1), v(0), v(2),
            g(2), v(0), v(3),
            g(3), v(1), v(2), v(3),
            g(4),
        ];
        assert_eq!(root, *nodes.last().unwrap());

        for (i, &gate) in [
            Gate::and(&[nodes[1], nodes[2]]),
            Gate::and(&[nodes[4], nodes[5]]),
            Gate::and(&[nodes[7], nodes[8]]),
            Gate::and(&[nodes[10], nodes[11], nodes[12]]),
            Gate::or(&[nodes[0], nodes[3], nodes[6], nodes[9]]),
        ]
        .iter()
        .enumerate()
        {
            assert_eq!(circuit.gate(g(i)), Some(gate), "mismatch for gate {i}");
        }
    }
}
