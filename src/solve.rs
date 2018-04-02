#![allow(dead_code, unused_variables)]

use ir;
use std::collections::HashMap;

use timely::{self, dataflow::*};

use differential_dataflow::input::Input;
use differential_dataflow::operators::*;

macro_rules! from_usize {
    ($t: ident) => {
        impl From<usize> for $t {
            fn from(index: usize) -> $t {
                $t { index }
            }
        }
    };
}

// Types whose definitions I don't actually know.
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Abomonation, Debug, Hash)]
struct Region {
    index: usize,
}
from_usize!(Region);

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Abomonation, Debug, Hash)]
struct Borrow {
    index: usize,
}
from_usize!(Borrow);

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Abomonation, Debug, Hash)]
struct Point {
    index: usize,
}
from_usize!(Point);

struct Intern<TargetType: From<usize> + Copy> {
    strings: HashMap<String, TargetType>,
}

impl<TargetType> Intern<TargetType>
where
    TargetType: From<usize> + Copy,
{
    fn new() -> Self {
        Self {
            strings: HashMap::new(),
        }
    }

    fn intern(&mut self, data: &str) -> TargetType {
        if let Some(&interned) = self.strings.get(data) {
            return interned;
        }

        let index = TargetType::from(self.strings.len());
        *self.strings.entry(data.to_string()).or_insert(index)
    }
}

// This basically recreates what is in regions.dl
fn region_computation(input: &ir::Input) {
    let regions: Intern<Region> = Intern::new();
    let borrows: Intern<Borrow> = Intern::new();
    let points: Intern<Point> = Intern::new();

    timely::execute_from_args(vec![].into_iter(), move |worker| {
        let mut probe = ProbeHandle::new();

        let inputs = worker.dataflow::<(), _, _>(|scope| {
            // inputs to the computation
            let (input_1, borrow_region) = scope.new_collection::<(Region, Borrow, Point), isize>();
            let (input_2, next_statement) = scope.new_collection::<(Point, Point), isize>();
            let (input_3, goto) = scope.new_collection::<(Point, Point), isize>();
            let (input_4, region_live_on_entry_to_statement) =
                scope.new_collection::<(Region, Point), isize>();
            let (input_5, killed) = scope.new_collection::<(Borrow, Point), isize>();
            let (input_6, outlives) = scope.new_collection::<(Region, Region, Point), isize>();

            // cfgEdge(P, Q) :- nextStatement(P, Q).
            // cfgEdge(P, Q) :- goto(P, Q).
            let cfg_edge = next_statement
                .concat(&goto)
                .distinct()
                .probe_with(&mut probe);

            // .decl regionLiveAt( r:region, p:point )
            let region_live_at = {
                // regionLiveAt(R, P) :- regionLiveOnEntryToStatement(R, P).
                let region_live_at1 = region_live_on_entry_to_statement.clone();

                // regionLiveAt(R, P) :-
                //   goto(P, Q),
                //   regionLiveOnEntryToStatement(R, Q).
                let region_live_at2 = {
                    let goto_invert = goto.map(|(p, q)| (q, p));
                    let region_live_on_entry_to_statement_invert =
                        region_live_on_entry_to_statement.map(|(r, q)| (q, r));
                    goto_invert.join_map(&region_live_on_entry_to_statement_invert, |_q, &p, &r| {
                        (r, p)
                    })
                };

                region_live_at1
                    .concat(&region_live_at2)
                    .distinct()
                    .probe_with(&mut probe)
            };

            // .decl restricts( r:region, b:borrow, p:point )
            let restricts = borrow_region.iterate(|restricts| {
                let borrow_region = borrow_region.enter(&restricts.scope());
                let outlives = outlives.enter(&restricts.scope());
                let next_statement = next_statement.enter(&restricts.scope());
                let killed = killed.enter(&restricts.scope());
                let cfg_edge = cfg_edge.enter(&restricts.scope());
                let region_live_at = region_live_at.enter(&restricts.scope());

                // restricts(R, B, P) :- borrowRegion(R, B, P).
                let restricts1 = borrow_region.clone();

                // restricts(R1, B, Q) :-
                //   restricts(R2, B, P)
                //   !killed(B, P)
                //   nextStatement(P, Q)
                //   outlives(R2, R1, P)
                let restricts2 = restricts
                    .map(|(r2, b, p)| ((b, p), r2))
                    .antijoin(&killed)
                    .map(|((b, p), r2)| (p, (b, r2)))
                    .join(&next_statement)
                    .map(|(p, (b, r2), q)| ((p, r2), (b, q)))
                    .join(&outlives.map(|(r2, r1, p)| ((p, r2), r1)))
                    .map(|((p, r2), (b, q), r1)| (r1, b, q));

                // restricts(R1, B, Q) :-
                //   restricts(R1, B, P)
                //   !killed(B, P)
                //   cfgEdge(P, Q)
                //   regionLiveAt(R1, Q)
                let restricts3 = restricts
                    .map(|(r1, b, p)| ((b, p), r1))
                    .antijoin(&killed)
                    .map(|((b, p), r1)| (p, (b, r1)))
                    .join(&cfg_edge)
                    .map(|(p, (b, r1), q)| ((r1, q), (b, p)))
                    .semijoin(&region_live_at)
                    .map(|((r1, q), (b, p))| (r1, b, q));

                restricts1
                    .concat(&restricts2)
                    .concat(&restricts3)
                    .distinct()
            });

            // borrowLiveAt(B, P) :-
            //   restricts(R, B, P)
            //   regionLiveAt(R, P)
            let borrow_live_at = {
                restricts
                    .map(|(r, b, p)| ((r, p), b))
                    .semijoin(&region_live_at)
                    .map(|((r, p), b)| (b, p))
                    .probe_with(&mut probe)
            };

            (input_1, input_2, input_3, input_4, input_5, input_6)
        });
    }).unwrap();
}
