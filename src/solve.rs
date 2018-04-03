#![allow(dead_code, unused_variables, unused_mut)]

use facts::*;
use ir;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use timely::{self, dataflow::*};

use differential_dataflow::input::Input;
use differential_dataflow::operators::*;

struct Interner<TargetType: From<usize> + Copy> {
    strings: HashMap<String, TargetType>,
    rev_strings: Vec<String>,
}

impl<TargetType> Interner<TargetType>
where
    TargetType: From<usize> + Into<usize> + Copy,
{
    fn new() -> Self {
        Self {
            strings: HashMap::new(),
            rev_strings: vec![],
        }
    }

    fn untern(&self, data: TargetType) -> &str {
        let data: usize = data.into();
        &self.rev_strings[data]
    }

    fn intern(&mut self, data: &str) -> TargetType {
        if let Some(&interned) = self.strings.get(data) {
            return interned;
        }

        let index = TargetType::from(self.strings.len());
        self.rev_strings.push(data.to_string());
        *self.strings.entry(data.to_string()).or_insert(index)
    }
}

struct InternerTables {
    regions: Interner<Region>,
    borrows: Interner<Borrow>,
    points: Interner<Point>,
}

impl InternerTables {
    fn new() -> Self {
        Self {
            regions: Interner::new(),
            borrows: Interner::new(),
            points: Interner::new(),
        }
    }
}

trait Intern<From: ?Sized> {
    fn intern(tables: &mut InternerTables, input: From) -> Self;
}

macro_rules! intern_impl {
    ($t: ident, $field: ident) => {
        impl Intern<&str> for $t {
            fn intern(tables: &mut InternerTables, input: &str) -> Self {
                tables.$field.intern(input)
            }
        }
    };
}

intern_impl!(Region, regions);
intern_impl!(Borrow, borrows);
intern_impl!(Point, points);

impl<A, FromA, B, FromB> Intern<(FromA, FromB)> for (A, B)
where
    A: Intern<FromA>,
    B: Intern<FromB>,
{
    fn intern(tables: &mut InternerTables, input: (FromA, FromB)) -> (A, B) {
        let (from_a, from_b) = input;
        (A::intern(tables, from_a), B::intern(tables, from_b))
    }
}

impl<A, FromA, B, FromB, C, FromC> Intern<(FromA, FromB, FromC)> for (A, B, C)
where
    A: Intern<FromA>,
    B: Intern<FromB>,
    C: Intern<FromC>,
{
    fn intern(tables: &mut InternerTables, input: (FromA, FromB, FromC)) -> (A, B, C) {
        let (from_a, from_b, from_c) = input;
        (
            A::intern(tables, from_a),
            B::intern(tables, from_b),
            C::intern(tables, from_c),
        )
    }
}

impl<A, FromA, B, FromB, C, FromC, D, FromD> Intern<(FromA, FromB, FromC, FromD)> for (A, B, C, D)
where
    A: Intern<FromA>,
    B: Intern<FromB>,
    C: Intern<FromC>,
    D: Intern<FromD>,
{
    fn intern(tables: &mut InternerTables, input: (FromA, FromB, FromC, FromD)) -> (A, B, C, D) {
        let (from_a, from_b, from_c, from_d) = input;
        (
            A::intern(tables, from_a),
            B::intern(tables, from_b),
            C::intern(tables, from_c),
            D::intern(tables, from_d),
        )
    }
}

trait PushInterned<E> {
    fn push_interned<I>(&mut self, tables: &mut InternerTables, element: I)
    where
        E: Intern<I>;
}

impl<E> PushInterned<E> for Vec<E> {
    fn push_interned<I>(&mut self, tables: &mut InternerTables, element: I)
    where
        E: Intern<I>,
    {
        self.push(E::intern(tables, element));
    }
}

struct AllFacts {
    borrow_region: Vec<(Region, Borrow, Point)>,
    next_statement: Vec<(Point, Point)>,
    goto: Vec<(Point, Point)>,
    region_live_on_entry: Vec<(Region, Point)>,
    killed: Vec<(Borrow, Point)>,
    outlives: Vec<(Point, Region, Region, Point)>,
}

// This basically recreates what is in regions.dl
crate fn region_computation(input: &ir::Input) {
    let mut intern_tables = InternerTables::new();
    let borrow_live_at_vec: Arc<Mutex<Vec<(Borrow, Point)>>> = Arc::new(Mutex::new(Vec::new()));

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
                    Ok::<(), !>(temp.push_interned(&mut $intern_tables, ($($arg_name),*)))
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

    push_timely_facts(all_facts, borrow_live_at_vec.clone());

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

                // borrowLiveAt(B, P) :-
                //   restricts(R, B, P)
                //   regionLiveAt(R, P)
                let borrow_live_at = {
                    let borrow_live_at_vec = borrow_live_at_vec.clone();
                    restricts
                        .map(|(r, b, p)| ((r, p), b))
                        .semijoin(&region_live_at)
                        .map(|((r, p), b)| (b, p))
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
