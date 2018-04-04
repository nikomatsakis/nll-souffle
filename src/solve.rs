#![allow(dead_code, unused_variables, unused_mut)]

use facts::*;
use intern::{InternTo, InternerTables};
use ir;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use timely::{self, dataflow::*};

use differential_dataflow::input::Input;
use differential_dataflow::operators::*;

trait PushInterned<E> {
    fn push_interned(&mut self, tables: &mut InternerTables, element: impl InternTo<E>);
}

impl<E> PushInterned<E> for Vec<E> {
    fn push_interned(&mut self, tables: &mut InternerTables, element: impl InternTo<E>) {
        self.push(InternTo::intern(tables, element));
    }
}

// This basically recreates what is in regions.dl
crate fn region_computation(input: &ir::Input) {
    let mut intern_tables = &mut InternerTables::new();

    macro_rules! collect_facts {
        (
            $input:expr,
            $for_each_name:ident,
            $intern_tables:expr,
            ($($arg_name:ident : $arg_ty:ty),*),
        ) => {
            {
                let mut temp: Vec<($($arg_ty),*)> = vec![];
                $input.$for_each_name(|$($arg_name : &str),*| {
                    Ok::<(), !>(temp.push_interned($intern_tables, ($($arg_name),*)))
                }).unwrap();
                temp
            }
        }
    }

    let all_facts = AllFacts {
        borrow_region: collect_facts!(
            input,
            for_each_borrow_region_fact,
            intern_tables,
            (r: Region, b: Borrow, p: Point),
        ),

        next_statement: collect_facts!(
            input,
            for_each_next_statement_fact,
            intern_tables,
            (p: Point, q: Point),
        ),

        goto: collect_facts!(
            input,
            for_each_goto_fact,
            intern_tables,
            (p: Point, q: Point),
        ),

        region_live_on_entry: collect_facts!(
            input,
            for_each_region_live_on_entry_fact,
            intern_tables,
            (p: Region, q: Point),
        ),

        killed: collect_facts!(
            input,
            for_each_killed_fact,
            intern_tables,
            (b: Borrow, p: Point),
        ),

        outlives: collect_facts!(
            input,
            for_each_outlives_fact,
            intern_tables,
            (p: Point, a: Region, b: Region, q: Point),
        ),
    };

    region_computation_from_facts(intern_tables, all_facts);
}

crate fn region_computation_from_facts(intern_tables: &mut InternerTables, all_facts: AllFacts) {
    let instant = Instant::now();
    let borrow_live_at_vec: Arc<Mutex<Vec<(Borrow, Point)>>> = Arc::new(Mutex::new(Vec::new()));
    push_timely_facts(all_facts, borrow_live_at_vec.clone());
    let duration = instant.elapsed();
    println!("duration: {}.{:09}s", duration.as_secs(), duration.subsec_nanos());

    println!("vvv borrowLiveAt vvv");
    let mut vector = borrow_live_at_vec.lock().unwrap().clone();
    vector.sort();
    for (borrow, point) in vector {
        println!(
            "borrow {} live at {}",
            intern_tables.borrows.untern(borrow),
            intern_tables.points.untern(point),
        );
    }
    println!("^^^ borrowLiveAt ^^^");
}

fn push_timely_facts(facts: AllFacts, borrow_live_at_vec: Arc<Mutex<Vec<(Borrow, Point)>>>) {
    timely::execute_from_args(vec![].into_iter(), {
        move |worker| {
            let probe = &mut ProbeHandle::new();

            let (
                mut input_borrow_region,
                mut input_next_statement,
                mut input_goto,
                mut input_region_live_on_entry,
                mut input_killed,
                mut input_outlives,
            ) = worker.dataflow::<(), _, _>(|scope| {
                // inputs to the computation
                let (input_1, borrow_region) =
                    scope.new_collection::<(Region, Borrow, Point), isize>();
                let (input_2, next_statement) = scope.new_collection::<(Point, Point), isize>();
                let (input_3, goto) = scope.new_collection::<(Point, Point), isize>();
                let (input_4, region_live_on_entry) =
                    scope.new_collection::<(Region, Point), isize>();
                let (input_5, killed) = scope.new_collection::<(Borrow, Point), isize>();
                let (input_6, outlives) =
                    scope.new_collection::<(Point, Region, Region, Point), isize>();

                // cfgEdge(P, Q) :- nextStatement(P, Q).
                // cfgEdge(P, Q) :- goto(P, Q).
                let cfg_edge = next_statement.concat(&goto).distinct().probe_with(probe);

                // .decl regionLiveAt( r:region, p:point )
                let region_live_at = {
                    // regionLiveAt(R, P) :- regionLiveOnEntryToStatement(R, P).
                    let region_live_at1 = region_live_on_entry.clone();

                    // regionLiveAt(R, P) :-
                    //   goto(P, Q),
                    //   regionLiveOnEntryToStatement(R, Q).
                    let region_live_at2 = {
                        let goto_invert = goto.map(|(p, q)| (q, p));
                        let region_live_on_entry_invert = region_live_on_entry.map(|(r, q)| (q, r));
                        goto_invert.join_map(&region_live_on_entry_invert, |_q, &p, &r| (r, p))
                    };

                    region_live_at1
                        .concat(&region_live_at2)
                        .distinct()
                        .probe_with(probe)
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
                    //   outlives(P, R2, R1, Q)
                    let restricts2 = restricts
                        .map(|(r2, b, p)| ((b, p), r2))
                        .antijoin(&killed)
                        .map(|((b, p), r2)| ((p, r2), b))
                        .join(&outlives.map(|(p, r2, r1, q)| ((p, r2), (r1, q))))
                        .map(|((p, r2), b, (r1, q))| (r1, b, q));

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
                        .map(|(_p, (b, r1), q)| ((r1, q), b))
                        .semijoin(&region_live_at)
                        .map(|((r1, q), b)| (r1, b, q));

                    restricts1
                        .concat(&restricts2)
                        .concat(&restricts3)
                        .distinct()
                });

                // .decl pointsTo( r:region, b:borrow, p:point )
                let points_to = borrow_region.iterate(|points_to| {
                    let borrow_region = borrow_region.enter(&points_to.scope());
                    let outlives = outlives.enter(&points_to.scope());
                    let next_statement = next_statement.enter(&points_to.scope());
                    let cfg_edge = cfg_edge.enter(&points_to.scope());
                    let region_live_at = region_live_at.enter(&points_to.scope());

                    // pointsTo(R, B, P) :- borrowRegion(R, B, P).
                    let points_to1 = borrow_region.clone();

                    // pointsTo(R1, B, Q) :-
                    //   pointsTo(R2, B, P)
                    //   outlives(P, R2, R1, Q)
                    let points_to2 = points_to
                        .map(|(r2, b, p)| ((p, r2), b))
                        .join(&outlives.map(|(p, r2, r1, q)| ((p, r2), (r1, q))))
                        .map(|((p, r2), b, (r1, q))| (r1, b, q));

                    // pointsTo(R1, B, Q) :-
                    //   pointsTo(R1, B, P)
                    //   cfgEdge(P, Q)
                    //   regionLiveAt(R1, Q)
                    let points_to3 = points_to
                        .map(|(r1, b, p)| (p, (b, r1)))
                        .join(&cfg_edge)
                        .map(|(_p, (b, r1), q)| ((r1, q), b))
                        .semijoin(&region_live_at)
                        .map(|((r1, q), b)| (r1, b, q));

                    points_to1
                        .concat(&points_to2)
                        .concat(&points_to3)
                             .distinct()
                             .inspect(|_| ())
                });

                // borrowLiveAt(B, P) :-
                //   restricts(R, B, P)
                //   regionLiveAt(R, P)
                let borrow_live_at = {
                    let borrow_live_at_vec = borrow_live_at_vec.clone();
                    restricts
                        .map(|(r, b, p)| ((r, p), b))
                        .semijoin(&region_live_at)
                        .map(|((r, p), b)| (b, p))
                        .distinct()
                        .inspect(move |&((b, p), _timestamp, _diff)| {
                            borrow_live_at_vec.lock().unwrap().push((b, p));
                        })
                        .probe_with(probe)
                };

                (input_1, input_2, input_3, input_4, input_5, input_6)
            });

            macro_rules! add_fact {
                ($input_name:ident, $facts_name:expr) => {
                    for fact in $facts_name.iter().cloned() {
                        $input_name.insert(fact);
                    }
                    $input_name.flush();
                }
            }

            add_fact!(input_borrow_region, facts.borrow_region);
            add_fact!(input_next_statement, facts.next_statement);
            add_fact!(input_goto, facts.goto);
            add_fact!(input_region_live_on_entry, facts.region_live_on_entry);
            add_fact!(input_killed, facts.killed);
            add_fact!(input_outlives, facts.outlives);
        }
    }).unwrap();
}
