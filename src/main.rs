use std::fs::File;

use ast::{Expression, Program, Statement};
use parser::Parser;
use std::env;
use std::fs;
use std::io::Write;
use tokenizer::Tokenizer;

mod ast;
mod codegen;
mod parser;
mod partial_evaluator;
mod remove_complex_operands;
mod tokenizer;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: yep <filename>");
        return;
    }

    let source_path = &args[1];
    let source = fs::read_to_string(source_path).unwrap();

    let tokens = Tokenizer::new(source).tokenize();
    let program = Parser::new(tokens).parse();
    dbg!(program);

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
