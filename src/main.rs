use std::fs::File;

use crate::partial_evaluator::PartialEvaluator;
use crate::remove_complex_operands::RemoveComplexOperandsPass;
use parser::Parser;
use std::env;
use std::env::Args;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tokenizer::Tokenizer;

mod ast;
mod codegen;
mod parser;
mod partial_evaluator;
mod remove_complex_operands;
mod tokenizer;

struct CompileOptions {
    source_path: PathBuf,
    output_path: PathBuf,
    compile_runtime: bool,
}

impl From<Args> for CompileOptions {
    fn from(value: Args) -> Self {
        let args: Vec<String> = value.collect();

        if args.len() < 2 {
            eprintln!("Usage: yep <filename> [-o <output_filename>]");
            panic!();
        };

        let mut args = args.into_iter().skip(1).peekable();
        let source_path = PathBuf::from(args.next().unwrap().as_str());
        if !source_path.is_file() {
            panic!("Only files can be compiled");
        };

        let output_path = if let Some(flag) = args.next() {
            if flag == "-o" {
                if args.peek().is_none() {
                    eprintln!("-o flag provided with no value");
                    panic!();
                }

                PathBuf::from(args.next().unwrap().as_str())
            } else {
                eprintln!("Unknown flag provided: {}", flag);
                panic!();
            }
        } else {
            source_path.with_extension("")
        };

        Self {
            source_path,
            output_path,
            compile_runtime: true,
        }
    }
}

fn main() {
    let compile_options = CompileOptions::from(env::args());

    let source = fs::read_to_string(&compile_options.source_path).unwrap();

    println!("Compiling {}", compile_options.source_path.display());
    let tokens = Tokenizer::new(source).tokenize();
    let program = Parser::new(tokens).parse();
    let program = PartialEvaluator::new(program).evaluate();
    let program = RemoveComplexOperandsPass::new(program).run();

    let mut codegen = codegen::X86AssemblyCodegen::new(program);
    let instructions = codegen.generate();

    let asm_path = compile_options.output_path.with_extension("asm");

    let mut file = File::create(&asm_path).unwrap();
    for instruction in instructions {
        writeln!(file, "{}", instruction).unwrap();
    }

    if compile_options.compile_runtime {
        Command::new("gcc")
            .args(["-c", "runtime.c", "-o", "runtime.o"])
            .spawn()
            .expect("failed to compile runtime");
    }

    let object_path = compile_options.output_path.with_extension("o");
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

    let program_path = compile_options.output_path;
    let gcc_output = Command::new("gcc")
        .args(vec![
            &object_path.display().to_string(),
            "runtime.o",
            "-o",
            &program_path.display().to_string(),
        ])
        .output()
        .expect("failed to compile");

    let _stdout = String::from_utf8_lossy(&gcc_output.stdout);
    let _stderr = String::from_utf8_lossy(&gcc_output.stderr);
}
