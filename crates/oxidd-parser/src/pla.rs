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
            Err(e) => return fail(input, "bad utf8"),
        };
        let cvr = match Cover::from_pla_string(pla_str) {
            Ok(c) => c,
            Err(e) => return fail(input, "bad pla"),
        };
        println!(
            "cvr: {}",
            cvr.to_pla_string(espresso_logic::CoverType::FD)
                .expect("fd to string")
        );
        let ni = cvr.num_inputs();
        let no = cvr.num_outputs();
        let ilb = Vec::from_iter(cvr.input_labels().iter().map(move |l| l.as_ref()));
        println!("ilb init: {:#?}", ilb);
        let olb = Vec::from_iter(cvr.output_labels().iter().map(move |l| l.as_ref()));
        println!("olb init: {:#?}", olb);
        let mut vs = VarSet::new(ni + no);
        if ilb.len() > 0 {
            assert!(ilb.len() == ni);
            for i in 0..ni {
                vs.set_name(i, ilb[i]);
            }
        }
        if olb.len() > 0 {
            assert!(olb.len() == no);
            for i in 0..no {
                vs.set_name(ni + i, olb[i]);
            }
        }

        let mut circt = Circuit::new(vs);
        // let root = circt.push_gate(GateKind::Or);
        let mut terms: Vec<Var> = vec![];
        println!("FALSE: {} {} {} TRUE: {} {} {}", Literal::FALSE, Literal::FALSE.0, Literal::FALSE.0 >> Literal::VAR_LSB, Literal::TRUE, Literal::TRUE.0, Literal::TRUE.0 >> Literal::VAR_LSB);

        for mt in cvr.cubes() {
            circt.push_gate(GateKind::And);
            println!("mt: {:#?}", mt);
            let ins = Vec::from_iter(mt.inputs().iter().map(|ib| match ib {
                Some(true) => Literal::TRUE,
                Some(false) => Literal::TRUE,
                None => Literal::UNDEF,
            }));
            println!("ins: {:#?}", ins);

            let mt_in_lits: Vec<Literal> = mt
                .inputs()
                .iter()
                .enumerate()
                .filter_map(|(i, ib)| match ib {
                    // Some(true) => Some(Literal::from_input(false, 1 + i)),
                    // Some(false) => Some(Literal::from_input(true, 1 + i)),
                    Some(true) => Some(Literal::FALSE),
                    Some(false) => Some(Literal::FALSE),
                    None => None,
                })
                .collect();

            let mt_out_lits: Vec<Literal> = mt
                .outputs()
                .iter()
                .map(|ob| match ob {
                    true => Literal::TRUE,
                    false => Literal::TRUE,
                })
                .collect();
            println!(
                "mt_in_lits:\n{:#?}\nmt_out_lits:\n{:#?}",
                mt_in_lits, mt_out_lits
            );
        }

        Ok((
            input,
            Problem {
                circuit: circt,
                details: crate::ProblemDetails::Root(Literal::UNDEF),
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
            1-1- 0\n\
            1--1 1\n\
            -111 1\n\
            .e\n";

        let (input, problem) = parse::<()>(&OPTS_NO_ORDER).parse(input).finish().unwrap();
        // assert!(input.is_empty());

        let (circuit, root) = unwrap_problem(problem);
        let inputs = circuit.inputs();
        assert_eq!(inputs.len(), 4);
        // assert!(inputs.order().is_none());

        let nodes = &[
            !v(2), // L -3
            !v(1), // L -2
            v(0),  // L 1
            g(0),  // A 3 2 1 0
            v(2),  // L 3
            g(1),  // O 3 2 4 3
            !v(3), // L -4
            g(2),  // A 2 6 5
            v(3),  // L 4
            g(3),  // A 2 2 8
            g(4),  // A 2 1 4
            v(1),  // L 2
            g(5),  // O 2 2 11 10
            g(6),  // A 2 12 9
            g(7),  // O 4 2 13 7
        ];
        assert_eq!(root, *nodes.last().unwrap());

        for (i, &gate) in [
            Gate::and(&[nodes[2], nodes[1], nodes[0]]), // 0: A 3 2 1 0
            Gate::or(&[nodes[4], nodes[3]]),            // 1: O 3 2 4 3
            Gate::and(&[nodes[6], nodes[5]]),           // 2: A 2 6 5
            Gate::and(&[nodes[2], nodes[8]]),           // 3: A 2 2 8
            Gate::and(&[nodes[1], nodes[4]]),           // 4: A 2 1 4
            Gate::or(&[nodes[11], nodes[10]]),          // 5: O 2 2 11 10
            Gate::and(&[nodes[12], nodes[9]]),          // 6: A 2 12 9
            Gate::or(&[nodes[13], nodes[7]]),           // 7: O 4 2 13 7
        ]
        .iter()
        .enumerate()
        {
            assert_eq!(circuit.gate(g(i)), Some(gate), "mismatch for gate {i}");
        }
    }
}
