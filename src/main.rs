use std::fs::File;

use ast::{Expression, Program, Statement};
use std::io::Write;

mod ast;
mod codegen;
mod partial_evaluator;
mod remove_complex_operands;
mod tokenizer;

fn main() {
    let program = Program {
        statements: vec![
            Statement::VariableDeclaration {
                name: "foo".to_string(),
                value: Expression::Constant { value: 123 },
            },
            Statement::Expression(Expression::Call {
                name: "print_int".to_string(),
                args: vec![Expression::VariableAccess {
                    name: "foo".to_string(),
                }],
            }),
        ],
    };

    let mut codegen = codegen::X86AssemblyCodegen::new(program);
    let instructions = codegen.generate();

    let mut file = File::create("./program.asm").unwrap();
    for instruction in instructions {
        writeln!(file, "{}", instruction).unwrap();
    }
}
