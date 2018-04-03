use facts::*;
use std::collections::HashMap;

crate struct Interner<TargetType: From<usize> + Copy> {
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

    crate fn untern(&self, data: TargetType) -> &str {
        let data: usize = data.into();
        &self.rev_strings[data]
    }

    crate fn intern(&mut self, data: &str) -> TargetType {
        if let Some(&interned) = self.strings.get(data) {
            return interned;
        }

        let index = TargetType::from(self.strings.len());
        self.rev_strings.push(data.to_string());
        *self.strings.entry(data.to_string()).or_insert(index)
    }
}

crate struct InternerTables {
    crate regions: Interner<Region>,
    crate borrows: Interner<Borrow>,
    crate points: Interner<Point>,
}

impl InternerTables {
    crate fn new() -> Self {
        Self {
            regions: Interner::new(),
            borrows: Interner::new(),
            points: Interner::new(),
        }
    }
}

crate trait Intern<From: ?Sized> {
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
