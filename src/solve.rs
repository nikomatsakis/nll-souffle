#![allow(dead_code, unused_variables)]

use crate::ir;
use std::collections::HashMap;

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

    //let mut probe = ProbeHandle::new();
    //
    //let mut inputs = worker.dataflow::<(),_,_>(|scope| {
    //    // inputs to the computation
    //    let (input_1, borrow_region) = scope.new_collection::<(Region,Borrow,Point),isize>();
    //    let (input_2, next_statement) = scope.new_collection::<(Point,Point),isize>();
    //    let (input_3, goto) = scope.new_collection::<(Point,Point),isize>();
    //    let (input_4, rloets) = scope.new_collection::<(Region,Point),isize>();
    //    let (input_5, killed) = scope.new_collection::<(Borrow,Point),isize>();
    //    let (input_6, out_lives) = scope.new_collection::<(Region,Region,Point),isize>();
    //});
}
