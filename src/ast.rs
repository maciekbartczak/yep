pub enum Node {
    Constant {
        value: i64,
    },
    UnaryOp {
        operator: Operator,
        operand: Box<Node>,
    },
    BinaryOp {
        left: Box<Node>,
        operator: Operator,
        right: Box<Node>,
    },
    Call {
        name: String,
        args: Vec<Node>,
    },
}

pub enum Operator {
    Sub,
    Add,
}

impl Node {
    pub fn is_leaf(&self) -> bool {
        match self {
            Node::Constant { .. } => true,
            Node::UnaryOp { .. } => false,
            Node::BinaryOp { .. } => false,
            Node::Call { .. } => true,
        }
    }
}
