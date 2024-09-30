use ast::{Node, Operator};

mod ast;

fn main() {
    let not_eight = Node::UnaryOp {
        operator: Operator::Sub,
        operand: Box::new(Node::Constant { value: 8 }),
    };

    println!("{}", not_eight.is_leaf());
}
