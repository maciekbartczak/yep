use ast::{Expression, Operator};

mod ast;
mod partial_evaluator;

fn main() {
    let not_eight = Expression::UnaryOp {
        operator: Operator::Sub,
        operand: Box::new(Expression::Constant { value: 8 }),
    };

    println!("{}", not_eight.is_leaf());
}
