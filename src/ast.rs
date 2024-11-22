use crate::tokenizer::TokenType;

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
    Grouping {
        expression: Box<Expression>
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Operator {
    Sub,
    Add,
    Multiply,
    Divide
}

impl From<&TokenType> for Operator {
    fn from(value: &TokenType) -> Self {
        match value {
            TokenType::Plus => Self::Add,
            TokenType::Minus => Self::Sub,
            TokenType::Star => Self::Multiply,
            TokenType::Slash => Self::Divide,
            _ => panic!("Unknown operator for TokenType: {}", value),
        }
    }
}