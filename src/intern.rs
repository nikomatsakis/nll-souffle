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

crate trait InternTo<To> {
    fn intern(tables: &mut InternerTables, input: Self) -> To;
}

macro_rules! intern_impl {
    ($t: ident, $field: ident) => {
        impl InternTo<$t> for &str {
            fn intern(tables: &mut InternerTables, input: &str) -> $t {
                tables.$field.intern(input)
            }
        }
    };
}

intern_impl!(Region, regions);
intern_impl!(Borrow, borrows);
intern_impl!(Point, points);

impl<A, FromA, B, FromB> InternTo<(A, B)> for (FromA, FromB)
where
    FromA: InternTo<A>,
    FromB: InternTo<B>,
{
    fn intern(tables: &mut InternerTables, input: (FromA, FromB)) -> (A, B) {
        let (from_a, from_b) = input;
        (FromA::intern(tables, from_a), FromB::intern(tables, from_b))
    }
}

impl<A, FromA, B, FromB, C, FromC> InternTo<(A, B, C)> for (FromA, FromB, FromC)
where
    FromA: InternTo<A>,
    FromB: InternTo<B>,
    FromC: InternTo<C>,
{
    fn intern(tables: &mut InternerTables, input: (FromA, FromB, FromC)) -> (A, B, C) {
        let (from_a, from_b, from_c) = input;
        (
            FromA::intern(tables, from_a),
            FromB::intern(tables, from_b),
            FromC::intern(tables, from_c),
        )
    }
}

impl<A, FromA, B, FromB, C, FromC, D, FromD> InternTo<(A, B, C, D)> for (FromA, FromB, FromC, FromD)
where
    FromA: InternTo<A>,
    FromB: InternTo<B>,
    FromC: InternTo<C>,
    FromD: InternTo<D>,
{
    fn intern(tables: &mut InternerTables, input: (FromA, FromB, FromC, FromD)) -> (A, B, C, D) {
        let (from_a, from_b, from_c, from_d) = input;
        (
            FromA::intern(tables, from_a),
            FromB::intern(tables, from_b),
            FromC::intern(tables, from_c),
            FromD::intern(tables, from_d),
        )
    }
}
