pub struct Input {
    pub blocks: Vec<Block>,
}

pub struct Block {
    pub name: String,
    pub statements: Vec<Statement>,
    pub goto: Vec<String>,
}

pub struct Statement {
    pub effects: Vec<Effect>,
}

pub enum Effect {
    /// A borrow `borrow` occured in this statement; the resulting
    /// reference had the region `region` (these are often given the
    /// same name). This is typically accompanied by a `post` outlives
    /// requirement for the variable where the reference is stored.
    Borrow { borrow: String, region: String },

    /// Indicates that a region is live on entry to this statement.
    LiveOnEntry { region: String },
    Kill { borrow: String },

    /// Creates an outlives requirement indicating data froms from `a`
    /// into `b`; this is positioned at the start of the statement,
    /// and hence it indicates that anything that region `a` points
    /// to, region `b` may now point to. Used for assignments.
    Outlives { time: OutlivesTime, a: String, b: String },
}

pub enum OutlivesTime {
    Pre,
    Post,
}
