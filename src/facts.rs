macro_rules! from_usize {
    ($t: ident) => {
        impl From<usize> for $t {
            fn from(index: usize) -> $t {
                $t { index }
            }
        }

        impl Into<usize> for $t {
            fn into(self) -> usize {
                self.index
            }
        }
    };
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Abomonation, Debug, Hash)]
pub(crate) struct Region {
    index: usize,
}
from_usize!(Region);

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Abomonation, Debug, Hash)]
pub(crate) struct Borrow {
    index: usize,
}
from_usize!(Borrow);

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Abomonation, Debug, Hash)]
pub(crate) struct Point {
    index: usize,
}
from_usize!(Point);

