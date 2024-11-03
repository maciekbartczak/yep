use crate::ast::{Expression, Operator, Program, Statement};

pub struct PartialEvaluator {
    program: Program,
}

impl PartialEvaluator {
    pub fn new(program: Program) -> Self {
        Self { program }
    }

    pub fn evaluate(self) -> Program {
        Program {
            statements: self
                .program
                .statements
                .iter()
                .map(|statement| self.evaluate_statement(statement.clone()))
                .collect(),
        }
    }

    fn evaluate_statement(&self, statment: Statement) -> Statement {
        match statment {
            Statement::Expression(expression) => {
                Statement::Expression(self.evaluate_expression(expression))
            }
            Statement::VariableDeclaration { name, value } => Statement::VariableDeclaration {
                name,
                value: self.evaluate_expression(value),
            },
        }
    }

    fn evaluate_expression(&self, expression: Expression) -> Expression {
        match &expression {
            Expression::UnaryOp { operator, operand } => {
                let operand = self.evaluate_expression(*operand.clone());

                match operand {
                    Expression::Constant { value } => match operator {
                        Operator::Sub => Expression::Constant { value: -value },
                        Operator::Add => todo!(),
                    },
                    _ => expression,
                }
            }
            Expression::BinaryOp {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate_expression(*left.clone());
                let right = self.evaluate_expression(*right.clone());

                match (left, right) {
                    (
                        Expression::Constant { value: left_value },
                        Expression::Constant { value: right_value },
                    ) => match operator {
                        Operator::Sub => Expression::Constant {
                            value: left_value - right_value,
                        },
                        Operator::Add => Expression::Constant {
                            value: left_value + right_value,
                        },
                    },
                    _ => expression,
                }
            }
            _ => expression,
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn do_nothing_with_constant_expression() {
        // given
        let program = Program {
            statements: vec![Statement::Expression(Expression::Constant { value: 5 })],
        };
        let evaluator = PartialEvaluator::new(program);

        // when
        let result = evaluator.evaluate();

        // then
        assert_eq!(
            result.statements,
            vec![Statement::Expression(Expression::Constant { value: 5 })]
        );
    }

    #[test]
    fn evaluate_unary_op_with_constant() {
        // given
        let program = Program {
            statements: vec![Statement::Expression(Expression::UnaryOp {
                operator: Operator::Sub,
                operand: Box::new(Expression::Constant { value: 5 }),
            })],
        };

        let evaluator = PartialEvaluator::new(program);

        // when
        let result = evaluator.evaluate();

        // then
        assert_eq!(
            result.statements,
            vec![Statement::Expression(Expression::Constant { value: -5 })]
        );
    }

    #[test]
    fn evaluate_binary_op_with_constants() {
        // given
        let program = Program {
            statements: vec![Statement::Expression(Expression::BinaryOp {
                left: Box::new(Expression::Constant { value: 8 }),
                operator: Operator::Sub,
                right: Box::new(Expression::Constant { value: 3 }),
            })],
        };

        let evaluator = PartialEvaluator::new(program);

        // when
        let result = evaluator.evaluate();

        // then
        assert_eq!(
            result.statements,
            vec![Statement::Expression(Expression::Constant { value: 5 })]
        );
    }

    #[test]
    fn evaluate_binary_op_with_nested_expressions() {
        // given
        // 8 - (-(3 + 1) + 2) = 10
        let program = Program {
            statements: vec![Statement::Expression(Expression::BinaryOp {
                left: Box::new(Expression::Constant { value: 8 }),
                operator: Operator::Sub,
                right: Box::new(Expression::BinaryOp {
                    left: Box::new(Expression::UnaryOp {
                        operator: Operator::Sub,
                        operand: Box::new(Expression::BinaryOp {
                            left: Box::new(Expression::Constant { value: 3 }),
                            operator: Operator::Add,
                            right: Box::new(Expression::Constant { value: 1 }),
                        }),
                    }),
                    operator: Operator::Add,
                    right: Box::new(Expression::BinaryOp {
                        left: Box::new(Expression::Constant { value: 1 }),
                        operator: Operator::Add,
                        right: Box::new(Expression::Constant { value: 1 }),
                    }),
                }),
            })],
        };

        let evaluator = PartialEvaluator::new(program);

        // when
        let result = evaluator.evaluate();

        // then
        assert_eq!(
            result.statements,
            vec![Statement::Expression(Expression::Constant { value: 10 })]
        );
    }

    #[test]
    fn evaluate_constant_assignment_to_variable() {
        // given
        let program = Program {
            statements: vec![Statement::VariableDeclaration {
                name: "foo".to_string(),
                value: Expression::BinaryOp {
                    left: Box::new(Expression::Constant { value: 8 }),
                    operator: Operator::Sub,
                    right: Box::new(Expression::Constant { value: 3 }),
                },
            }],
        };

        let evaluator = PartialEvaluator::new(program);

        // when
        let result = evaluator.evaluate();

        // then
        assert_eq!(
            result.statements,
            vec![Statement::VariableDeclaration {
                name: "foo".to_string(),
                value: Expression::Constant { value: 5 }
            }]
        );
    }

    #[test]
    fn do_not_evalute_when_there_is_runtime_expressions() {
        // given
        // x - (-(3 + 1) + get_value()) = ???
        let program = Program {
            statements: vec![Statement::Expression(Expression::BinaryOp {
                left: Box::new(Expression::VariableAccess {
                    name: "x".to_string(),
                }),
                operator: Operator::Sub,
                right: Box::new(Expression::BinaryOp {
                    left: Box::new(Expression::UnaryOp {
                        operator: Operator::Sub,
                        operand: Box::new(Expression::BinaryOp {
                            left: Box::new(Expression::Constant { value: 3 }),
                            operator: Operator::Add,
                            right: Box::new(Expression::Call {
                                name: "get_value".to_string(),
                                args: vec![],
                            }),
                        }),
                    }),
                    operator: Operator::Add,
                    right: Box::new(Expression::BinaryOp {
                        left: Box::new(Expression::Constant { value: 1 }),
                        operator: Operator::Add,
                        right: Box::new(Expression::Constant { value: 1 }),
                    }),
                }),
            })],
        };
        let original_program = program.clone();

        let evaluator = PartialEvaluator::new(program);

        // when
        let result = evaluator.evaluate();

        // then
        assert_eq!(result.statements, original_program.statements);
    }
}
