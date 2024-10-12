use core::panic;
use std::{collections::HashMap, vec};

use crate::ast::{Expression, Program, Statement};

type Instruction = String;

pub struct X86AssemblyCodegen {
    program: Program,
    environment: Enviroment,
}

#[derive(Default)]
struct Enviroment {
    // variable name to stack offset map
    allocated_variables: HashMap<String, u32>,
    stack_offset: u32,
}

impl Enviroment {
    fn allocate_variable(&mut self, name: String) {
        // TODO: support different size of variables
        // TODO: error handling
        self.stack_offset = self.stack_offset + 8;
        self.allocated_variables.insert(name, self.stack_offset);
    }

    fn get_variable_stack_offset(&self, name: &String) -> u32 {
        // TODO: error handling
        *self.allocated_variables.get(name).unwrap()
    }
}

impl X86AssemblyCodegen {
    pub fn new(program: Program) -> Self {
        Self {
            program,
            environment: Enviroment::default(),
        }
    }

    pub fn generate(&mut self) -> Vec<Instruction> {
        let prelude = self.emit_prelude();
        let program_instructions = self
            .program
            .statements
            .clone() // TODO: how to get rid of this clone?
            .iter()
            .flat_map(|statement| self.emit_statement(&statement))
            .collect();
        let cleanup = self.emit_cleanup();

        [prelude, program_instructions, cleanup].concat()
    }

    fn emit_prelude(&self) -> Vec<Instruction> {
        vec![
            "global main".to_string(),
            "extern print_int".to_string(),
            "section .text".to_string(),
            "main:".to_string(),
        ]
    }

    fn emit_cleanup(&self) -> Vec<Instruction> {
        vec!["xor rax, rax".to_string(), "ret".to_string()]
    }

    fn emit_statement(&mut self, statement: &Statement) -> Vec<Instruction> {
        match statement {
            Statement::Expression(expression) => self.emit_expression(expression),
            Statement::VariableDeclaration { name, value } => {
                self.emit_variable_declaration(name, value)
            }
        }
    }

    fn emit_variable_declaration(
        &mut self,
        name: &String,
        initializer: &Expression,
    ) -> Vec<Instruction> {
        self.environment.allocate_variable(name.clone());
        let stack_offset = self.environment.get_variable_stack_offset(name);

        let value = match initializer {
            Expression::Constant { value } => value,
            Expression::VariableAccess { .. } => todo!(),
            _ => panic!("Tried to initialize variable using a non atomic expression"),
        };

        let instruction = format!("mov qword [rbp - {}], {}", stack_offset, value);

        vec![instruction]
    }

    fn emit_expression(&mut self, expression: &Expression) -> Vec<Instruction> {
        match expression {
            Expression::Call { name, args } => self.emit_function_call(name, args),
            _ => todo!(),
        }
    }

    fn emit_function_call(&mut self, name: &String, args: &Vec<Expression>) -> Vec<Instruction> {
        assert!(args.len() == 1, "Function calls support exactly 1 argument");

        let mut instructions = vec![];

        let source = match args.get(0).unwrap() {
            Expression::Constant { value } => format!("{}", value),
            Expression::VariableAccess { name } => {
                let stack_offset = self.environment.get_variable_stack_offset(name);
                instructions.push(format!("mov qword rax, [rbp - {}]", stack_offset));

                "rax".to_string()
            }
            _ => panic!("Tried to pass a function argument using a non atomic expression"),
        };

        instructions.push(format!("mov qword rdi, {}", source));
        instructions.push(format!("call {}", name));

        return instructions;
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn declare_variable_with_constant_initializer() {
        // given
        let program = Program {
            statements: vec![
                Statement::VariableDeclaration {
                    name: "foo".to_string(),
                    value: Expression::Constant { value: 4 },
                },
                Statement::VariableDeclaration {
                    name: "bar".to_string(),
                    value: Expression::Constant { value: 42 },
                },
                Statement::VariableDeclaration {
                    name: "baz".to_string(),
                    value: Expression::Constant { value: 127 },
                },
            ],
        };

        let mut codegen = X86AssemblyCodegen::new(program);

        // when
        let result = codegen.generate();

        // then
        assert_eq!(
            vec![
                "mov qword [rbp - 8], 4",
                "mov qword [rbp - 16], 42",
                "mov qword [rbp - 24], 127"
            ],
            result
        )
    }

    #[test]
    fn emit_function_call_with_constant_argument() {
        // given
        let program = Program {
            statements: vec![Statement::Expression(Expression::Call {
                name: "print_int".to_string(),
                args: vec![Expression::Constant { value: 4 }],
            })],
        };

        let mut codegen = X86AssemblyCodegen::new(program);

        // when
        let result = codegen.generate();

        // then
        assert_eq!(vec!["mov qword rdi, 4", "call print_int",], result)
    }
}
