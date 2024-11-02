use std::fs::File;

use parser::Parser;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
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

    let source_path = Path::new(&args[1]);
    if !source_path.is_file() {
        panic!("Only files can be compiled");
    }

    let source = fs::read_to_string(source_path).unwrap();
    println!("Compiling {}", source_path.display());

    let tokens = Tokenizer::new(source).tokenize();
    let program = Parser::new(tokens).parse();

    let mut codegen = codegen::X86AssemblyCodegen::new(program);
    let instructions = codegen.generate();

    let asm_path = source_path.with_extension("asm");

    let mut file = File::create(&asm_path).unwrap();
    for instruction in instructions {
        writeln!(file, "{}", instruction).unwrap();
    }

    let object_path = source_path.with_extension("o");
    Command::new("nasm")
        .args(vec![
            "-f",
            "elf64",
            &asm_path.display().to_string(),
            "-o",
            &object_path.display().to_string(),
        ])
        .output()
        .expect("failed to compile");

    let program_path = source_path.with_extension("");
    Command::new("gcc")
        .args(vec![
            &object_path.display().to_string(),
            "runtime.o",
            "-o",
            &program_path.display().to_string(),
        ])
        .output()
        .expect("failed to compile");
}
