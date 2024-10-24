pub type Program = Module;

#[derive(Clone, PartialEq, Debug)]
pub struct Module {
    pub statements: Vec<Statement>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Statement {
    Expression(Expression),
    VariableDeclaration { name: String, value: Expression },
}

#[derive(Clone, PartialEq, Debug)]
pub enum Expression {
    Constant {
        value: i64,
    },
    UnaryOp {
        operator: Operator,
        operand: Box<Expression>,
    },
    BinaryOp {
        left: Box<Expression>,
        operator: Operator,
        right: Box<Expression>,
    },
    Call {
        name: String,
        args: Vec<Expression>,
    },
    VariableAccess {
        name: String,
    },
}

#[derive(Clone, PartialEq, Debug)]
pub enum Operator {
    Sub,
    Add,
}

impl Expression {
    pub fn is_leaf(&self) -> bool {
        match self {
            Expression::Constant { .. } => true,
            Expression::Call { .. } => true,
            _ => false,
        }
    }
}
