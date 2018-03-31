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
    Borrow { region: String },
    Live { region: String },
    PreOutlives { a: String, b: String },
    PostOutlives { a: String, b: String },
}
