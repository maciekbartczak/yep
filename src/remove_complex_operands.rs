use crate::ast::{Expression, Program, Statement};

struct RemoveComplexOperandsPass {
    program: Program,
    temp_variable_index: u16,
}

struct TransformExpressionResult {
    expression: Expression,
    additional_statements: Vec<Statement>,
}

impl From<Expression> for TransformExpressionResult {
    fn from(value: Expression) -> Self {
        Self {
            expression: value,
            additional_statements: vec![],
        }
    }
}

// this compiler pass is tasked with transforming the ast so that only atomic operations
// (that is a constant or variable access expressions) are present in other expressions.
// for example:
// BinaryOp { left: Call {name: "get_foo" }, op: Sub, right: Constant (3) }
// should become:
// VariableDeclaratin { name: "tmp_0", expression: Call {name: "get_foo" } }
// BinaryOp { left: VariableAccess { name: "tmp_0" }, op: Sub, right: Constant (3) }
impl RemoveComplexOperandsPass {
    pub fn new(program: Program) -> Self {
        Self {
            program,
            temp_variable_index: 0,
        }
    }

    pub fn run(mut self) -> Program {
        return Program {
            statements: self
                .program
                .statements
                .clone() // TODO: How to get rid of this clone?
                .iter()
                .flat_map(|statement| self.transform_statement(statement.clone()))
                .collect(),
        };
    }

    fn transform_statement(&mut self, statement: Statement) -> Vec<Statement> {
        match statement {
            Statement::Expression(_expression) => todo!(),
            Statement::VariableDeclaration {
                name,
                value: initializer_expression,
            } => {
                let result = self.transform_expression(initializer_expression, false);

                let new_statement = Statement::VariableDeclaration {
                    name,
                    value: result.expression,
                };

                let mut new_statements = result.additional_statements;
                new_statements.push(new_statement);

                return new_statements;
            }
        }
    }

    fn transform_expression(
        &mut self,
        expression: Expression,
        should_create_temporary_variable: bool,
    ) -> TransformExpressionResult {
        match expression {
            Expression::Constant { .. } => expression.into(),
            Expression::VariableAccess { .. } => expression.into(),
            Expression::UnaryOp { operator, operand } => {
                let operand = self.transform_expression(*operand, true);
                let mut additional_statements = operand.additional_statements;

                if !should_create_temporary_variable {
                    return TransformExpressionResult {
                        expression: Expression::UnaryOp {
                            operator,
                            operand: Box::new(operand.expression),
                        },
                        additional_statements,
                    };
                }

                let (temp_variable_name, temp_variable_statement) = self
                    .declare_temporary_variable(Expression::UnaryOp {
                        operator,
                        operand: Box::new(operand.expression),
                    });

                additional_statements.push(temp_variable_statement);

                return TransformExpressionResult {
                    expression: Expression::VariableAccess {
                        name: temp_variable_name,
                    },
                    additional_statements,
                };
            }
            Expression::BinaryOp {
                left,
                operator,
                right,
            } => {
                let left = self.transform_expression(*left, true);
                let right = self.transform_expression(*right, true);

                let new_expression = Expression::BinaryOp {
                    left: Box::new(left.expression.clone()),
                    operator,
                    right: Box::new(right.expression.clone()),
                };
                let mut additional_statements: Vec<Statement> = left
                    .additional_statements
                    .into_iter()
                    .chain(right.additional_statements.into_iter())
                    .collect();

                if !should_create_temporary_variable {
                    return TransformExpressionResult {
                        expression: new_expression,
                        additional_statements,
                    };
                }

                let (temp_variable_name, temp_variable_statement) =
                    self.declare_temporary_variable(new_expression);
                additional_statements.push(temp_variable_statement);

                return TransformExpressionResult {
                    expression: Expression::VariableAccess {
                        name: temp_variable_name,
                    },
                    additional_statements,
                };
            }
            Expression::Call { name, args } => {
                let transformed_args: Vec<TransformExpressionResult> = args
                    .iter()
                    .map(|arg| self.transform_expression(arg.clone(), true))
                    .collect();

                let new_args: Vec<Expression> = transformed_args
                    .iter()
                    .map(|arg| arg.expression.clone())
                    .collect();
                let mut additional_statements: Vec<Statement> = transformed_args
                    .into_iter()
                    .flat_map(|arg| arg.additional_statements)
                    .collect();

                let (temp_variable_name, temp_variable_statement) = self
                    .declare_temporary_variable(Expression::Call {
                        name,
                        args: new_args,
                    });
                let new_expression = Expression::VariableAccess {
                    name: temp_variable_name,
                };

                additional_statements.push(temp_variable_statement);
                return TransformExpressionResult {
                    expression: new_expression,
                    additional_statements,
                };
            }
        }
    }

    fn declare_temporary_variable(
        &mut self,
        initialzer_expression: Expression,
    ) -> (String, Statement) {
        let temp_variable_name = format!("tmp_{}", self.temp_variable_index).to_string();
        self.temp_variable_index += 1;

        let statement = Statement::VariableDeclaration {
            name: temp_variable_name.clone(),
            value: initialzer_expression,
        };

        return (temp_variable_name, statement);
    }
}

#[cfg(test)]
mod test {
    use crate::ast::Operator;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn transform_method_call_into_variable_access() {
        // given
        let program = Program {
            statements: vec![Statement::VariableDeclaration {
                name: "test".to_string(),
                value: Expression::BinaryOp {
                    left: Box::new(Expression::BinaryOp {
                        left: Box::new(Expression::Call {
                            name: "get_number".to_string(),
                            args: vec![],
                        }),
                        operator: Operator::Add,
                        right: Box::new(Expression::Call {
                            name: "get_number_2".to_string(),
                            args: vec![Expression::Call {
                                name: "get_number_3".to_string(),
                                args: vec![],
                            }],
                        }),
                    }),
                    operator: Operator::Sub,
                    right: Box::new(Expression::Constant { value: 3 }),
                },
            }],
        };

        let pass = RemoveComplexOperandsPass::new(program);

        // when
        let result = pass.run();

        // then
        assert_eq!(
            result.statements,
            vec![
                Statement::VariableDeclaration {
                    name: "tmp_0".to_string(),
                    value: Expression::Call {
                        name: "get_number".to_string(),
                        args: vec![],
                    },
                },
                Statement::VariableDeclaration {
                    name: "tmp_1".to_string(),
                    value: Expression::Call {
                        name: "get_number_3".to_string(),
                        args: vec![],
                    },
                },
                Statement::VariableDeclaration {
                    name: "tmp_2".to_string(),
                    value: Expression::Call {
                        name: "get_number_2".to_string(),
                        args: vec![Expression::VariableAccess {
                            name: "tmp_1".to_string()
                        }],
                    },
                },
                Statement::VariableDeclaration {
                    name: "tmp_3".to_string(),
                    value: Expression::BinaryOp {
                        left: Box::new(Expression::VariableAccess {
                            name: "tmp_0".to_string()
                        }),
                        operator: Operator::Add,
                        right: Box::new(Expression::VariableAccess {
                            name: "tmp_2".to_string()
                        })
                    },
                },
                Statement::VariableDeclaration {
                    name: "test".to_string(),
                    value: Expression::BinaryOp {
                        left: Box::new(Expression::VariableAccess {
                            name: "tmp_3".to_string()
                        }),
                        operator: Operator::Sub,
                        right: Box::new(Expression::Constant { value: 3 }),
                    },
                }
            ]
        )
    }

    #[test]
    fn test() {
        // given
        let program = Program {
            statements: vec![Statement::VariableDeclaration {
                name: "test".to_string(),
                value: Expression::BinaryOp {
                    left: Box::new(Expression::BinaryOp {
                        left: Box::new(Expression::Constant { value: 3 }),
                        operator: Operator::Add,
                        right: Box::new(Expression::UnaryOp {
                            operator: Operator::Sub,
                            operand: Box::new(Expression::Constant { value: 4 }),
                        }),
                    }),
                    operator: Operator::Sub,
                    right: Box::new(Expression::UnaryOp {
                        operator: Operator::Sub,
                        operand: Box::new(Expression::Call {
                            name: "get_number".to_string(),
                            args: vec![],
                        }),
                    }),
                },
            }],
        };

        let pass = RemoveComplexOperandsPass::new(program);

        // when
        let result = pass.run();

        // then
        assert_eq!(
            result.statements,
            vec![
                Statement::VariableDeclaration {
                    name: "tmp_0".to_string(),
                    value: Expression::UnaryOp {
                        operator: Operator::Sub,
                        operand: Box::new(Expression::Constant { value: 4 }),
                    },
                },
                Statement::VariableDeclaration {
                    name: "tmp_1".to_string(),
                    value: Expression::BinaryOp {
                        left: Box::new(Expression::Constant { value: 3 }),
                        operator: Operator::Add,
                        right: Box::new(Expression::VariableAccess {
                            name: "tmp_0".to_string()
                        })
                    },
                },
                Statement::VariableDeclaration {
                    name: "tmp_2".to_string(),
                    value: Expression::Call {
                        name: "get_number".to_string(),
                        args: vec![],
                    },
                },
                Statement::VariableDeclaration {
                    name: "tmp_3".to_string(),
                    value: Expression::UnaryOp {
                        operator: Operator::Sub,
                        operand: Box::new(Expression::VariableAccess {
                            name: "tmp_2".to_string()
                        }),
                    },
                },
                Statement::VariableDeclaration {
                    name: "test".to_string(),
                    value: Expression::BinaryOp {
                        left: Box::new(Expression::VariableAccess {
                            name: "tmp_1".to_string()
                        }),
                        operator: Operator::Sub,
                        right: Box::new(Expression::VariableAccess {
                            name: "tmp_3".to_string()
                        }),
                    },
                }
            ]
        )
    }
}
