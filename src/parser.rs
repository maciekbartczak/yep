use crate::{
    ast::{Expression, Program, Statement},
    tokenizer::{Keyword, Token, TokenType},
};

pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, cursor: 0 }
    }

    pub fn parse(&mut self) -> Program {
        let mut statements = vec![];

        while !self.is_at_end() {
            statements.push(self.parse_statement());
        }

        Program { statements }
    }

    fn parse_statement(&mut self) -> Statement {
        self.parse_variable_declaration()
    }

    fn parse_variable_declaration(&mut self) -> Statement {
        if let Some(_) = self.consume_if_matched(vec![TokenType::Keyword(Keyword::Let)]) {
            let identifier = self.consume_required(TokenType::Identifier);

            self.consume_required(TokenType::Equals);

            let initializer = self.parse_expression();

            self.consume_required(TokenType::Semicolon);

            return Statement::VariableDeclaration {
                name: identifier.get_literal_value().to_string(),
                value: initializer,
            };
        }

        Statement::Expression(self.parse_expression())
    }

    fn parse_expression(&mut self) -> Expression {
        self.parse_term()
    }

    fn parse_term(&mut self) -> Expression {
        let mut expression = self.parse_factor();

        while let Some(_) = self.consume_if_matched(vec![TokenType::Plus, TokenType::Minus]) {
            let operator = self.get_previous_token();
            let rhs = self.parse_factor();

            expression = Expression::BinaryOp {
                left: Box::new(expression),
                operator: operator.get_type().into(),
                right: Box::new(rhs),
            }
        }

        expression
    }

    fn parse_factor(&mut self) -> Expression {
        let mut expression = self.parse_function_call();

        while let Some(_) = self.consume_if_matched(vec![TokenType::Star, TokenType::Slash]) {
            let operator = self.get_previous_token();
            let rhs = self.parse_function_call();

            expression = Expression::BinaryOp {
                left: Box::new(expression),
                operator: operator.get_type().into(),
                right: Box::new(rhs),
            }
        }

        expression
    }

    fn parse_function_call(&mut self) -> Expression {
        let expression = self.parse_unary();

        if let Some(_) = self.consume_if_matched(vec![TokenType::ParenthesesLeft]) {
            // TODO: handle expressions
            let variable_access = self.consume_required(TokenType::Identifier);

            self.consume_required(TokenType::ParenthesesRight);
            self.consume_required(TokenType::Semicolon);

            let function_name = match expression {
                Expression::VariableAccess { name } => name,
                _ => panic!(),
            };

            return Expression::Call {
                name: function_name,
                args: vec![Expression::VariableAccess {
                    name: variable_access.get_literal_value().to_string(),
                }],
            };
        }

        expression
    }

    fn parse_unary(&mut self) -> Expression {
        if let Some(operator) = self.consume_if_matched(vec![TokenType::Minus]) {
            let rhs = self.parse_unary();

            return Expression::UnaryOp {
                operator: operator.get_type().into(),
                operand: Box::new(rhs)
            }
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Expression {
        if let Some(identifier) = self.consume_if_matched(vec![TokenType::Identifier]) {
            return Expression::VariableAccess {
                name: identifier.get_literal_value().to_string(),
            };
        }

        if let Some(number) = self.consume_if_matched(vec![TokenType::Number]) {
            return Expression::Constant {
                value: number.get_literal_value().parse::<i64>().unwrap(),
            };
        }

        if let Some(_) = self.consume_if_matched(vec![TokenType::ParenthesesLeft]) {
            let expression = self.parse_expression();
            self.consume_required(TokenType::ParenthesesRight);

            return Expression::Grouping { expression: Box::new(expression) }
        }

        dbg!(&self.tokens[self.cursor]);
        panic!("Expected expression");
    }

    fn consume_required(&mut self, required_type: TokenType) -> Token {
        let current = self.tokens[self.cursor].clone();
        let current_type = current.get_type();

        if required_type != *current_type {
            // TODO: proper error reporting
            panic!(
                "Expected the following token: {}, but got {} instead",
                required_type, current_type
            );
        }

        self.cursor += 1;

        current
    }

    fn consume_if_matched(&mut self, wanted: Vec<TokenType>) -> Option<Token> {
        let current = self.tokens[self.cursor].clone();
        let current_type = current.get_type();

        for wanted_type in wanted {
            if wanted_type == *current_type {
                self.cursor += 1;
                return Some(current);
            }
        }

        None
    }

    fn get_previous_token(&mut self) -> Token {
        assert!(self.cursor > 0);
        self.tokens[self.cursor - 1].clone()
    }

    fn is_at_end(&self) -> bool {
        self.tokens[self.cursor].get_type() == &TokenType::Eof
    }
}
