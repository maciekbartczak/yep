use core::panic;
use std::{collections::HashMap, vec};

use crate::ast::{Expression, Program, Statement};

type Instruction = String;

pub struct X86AssemblyCodegen {
    program: Program,
    environment: Environment,
}

#[derive(Default)]
struct Environment {
    // variable name to stack offset map
    allocated_variables: HashMap<String, u32>,
    stack_offset: u32,
}

impl Environment {
    fn allocate_variable(&mut self, name: String) {
        // TODO: support different size of variables
        // TODO: error handling
        self.stack_offset = self.stack_offset + 4;
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
            environment: Environment::default(),
        }
    }

    pub fn generate(&mut self) -> Vec<Instruction> {
        let prelude = self.emit_prelude();
        let stack_space_allocation = self.emit_stack_space_allocation();
        let program_instructions = self
            .program
            .statements
            .clone() // TODO: how to get rid of this clone?
            .iter()
            .flat_map(|statement| self.emit_statement(&statement))
            .collect();
        let epilogue = self.emit_epilogue();

        [
            prelude,
            stack_space_allocation,
            program_instructions,
            epilogue,
        ]
        .concat()
    }

    fn emit_prelude(&self) -> Vec<Instruction> {
        vec![
            "global main".to_string(),
            "extern print_int".to_string(),
            "section .text".to_string(),
            "main:".to_string(),
            "push rbp".to_string(),
            "mov rbp, rsp".to_string(),
        ]
    }

    fn emit_epilogue(&self) -> Vec<Instruction> {
        vec![
            "mov rsp, rbp".to_string(),
            "pop rbp".to_string(),
            "xor rax, rax".to_string(),
            "ret".to_string(),
        ]
    }

    fn emit_stack_space_allocation(&self) -> Vec<Instruction> {
        // this function assumes we are operating on 32-bit integers for now
        let bytes_needed: u32 = self
            .program
            .statements
            .iter()
            .filter_map(|s| match s {
                Statement::Expression(_) => None,
                Statement::VariableDeclaration { .. } => Some(4),
            })
            .sum();


        if bytes_needed > 0 {
            // add 15 to get above the next multiple of 16
            // then clear the last 4 bits to round down to multiple of 16
            let aligned_space = (bytes_needed + 15) & !15;
            vec![format!("sub rsp, {}", aligned_space).to_string()]
        } else {
            vec![]
        }
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

        let instruction = format!("mov dword [rbp - {}], {}", stack_offset, value);

        vec![instruction]
    }

    fn emit_expression(&mut self, expression: &Expression) -> Vec<Instruction> {
        match expression {
            Expression::Call { name, args } => self.emit_function_call(name, args),
            _ => todo!(),
        }
    }

    fn emit_function_call(&mut self, name: &String, args: &Vec<Expression>) -> Vec<Instruction> {
        assert_eq!(args.len(), 1, "Function calls support exactly 1 argument");

        let mut instructions = vec![];

        let source = match args.get(0).unwrap() {
            Expression::Constant { value } => format!("{}", value),
            Expression::VariableAccess { name } => {
                let stack_offset = self.environment.get_variable_stack_offset(name);
                instructions.push(format!("mov dword rax, [rbp - {}]", stack_offset));

                "rax".to_string()
            }
            _ => panic!("Tried to pass a function argument using a non atomic expression"),
        };

        instructions.push(format!("mov dword rdi, {}", source));
        instructions.push(format!("call {}", name));

        instructions
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
                "global main",
                "extern print_int",
                "section .text",
                "main:",
                "push rbp",
                "mov rbp, rsp",
                "sub rsp, 16",
                "mov dword [rbp - 4], 4",
                "mov dword [rbp - 8], 42",
                "mov dword [rbp - 12], 127",
                "mov rsp, rbp",
                "pop rbp",
                "xor rax, rax",
                "ret"
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
        assert_eq!(
            vec![
                "global main",
                "extern print_int",
                "section .text",
                "main:",
                "push rbp",
                "mov rbp, rsp",
                "mov dword rdi, 4",
                "call print_int",
                "mov rsp, rbp",
                "pop rbp",
                "xor rax, rax",
                "ret"
            ],
            result
        )
    }
}
