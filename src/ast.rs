use crate::tokenizer::TokenType;
use std::ptr::addr_of;

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

impl From<&TokenType> for Operator {
    fn from(value: &TokenType) -> Self {
        match value {
            TokenType::Plus => Self::Add,
            TokenType::Minus => Self::Sub,
            _ => panic!("Unknown operator for TokenType: {}", value),
        }
    }
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
