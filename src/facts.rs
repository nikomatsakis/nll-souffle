macro_rules! from_usize {
    ($t: ident) => {
        impl From<usize> for $t {
            fn from(index: usize) -> $t {
                $t { index: index as u32 }
            }
        }

        impl Into<usize> for $t {
            fn into(self) -> usize {
                self.index as usize
            }
        }
    };
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Abomonation, Debug, Hash)]
pub(crate) struct Region {
    index: u32,
}
from_usize!(Region);

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Abomonation, Debug, Hash)]
pub(crate) struct Borrow {
    index: u32,
}
from_usize!(Borrow);

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Abomonation, Debug, Hash)]
pub(crate) struct Point {
    index: u32,
}
from_usize!(Point);

crate struct AllFacts {
    crate borrow_region: Vec<(Region, Borrow, Point)>,
    crate next_statement: Vec<(Point, Point)>,
    crate goto: Vec<(Point, Point)>,
    crate region_live_on_entry: Vec<(Region, Point)>,
    crate killed: Vec<(Borrow, Point)>,
    crate outlives: Vec<(Point, Region, Region, Point)>,
}

