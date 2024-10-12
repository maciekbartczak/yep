use core::panic;
use std::collections::HashMap;

use crate::ast::{Expression, Program, Statement};

type Instruction = String;

struct X86AssemblyCodegen {
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
    fn new(program: Program) -> Self {
        Self {
            program,
            environment: Enviroment::default(),
        }
    }

    fn generate(&mut self) -> Vec<Instruction> {
        self.program
            .statements
            .clone() // TODO: how to get rid of this clone?
            .iter()
            .flat_map(|statement| self.emit_statement(&statement))
            .collect()
    }

    fn emit_statement(&mut self, statement: &Statement) -> Vec<Instruction> {
        match statement {
            Statement::Expression(expression) => todo!(),
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
}
