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
            statements.push(self.parse_variable_declaration());
        }

        Program { statements }
    }

    fn parse_variable_declaration(&mut self) -> Statement {
        if let Some(_) = self.consume_if_matched(vec![TokenType::Keyword(Keyword::Let)]) {
            let identifier = self.consume_required(TokenType::Identifier);

            self.consume_required(TokenType::Equals);

            // TODO: handle expressions
            let number = self.consume_required(TokenType::Number);

            self.consume_required(TokenType::Semicolon);

            return Statement::VariableDeclaration {
                name: identifier.get_literal_value().to_string(),
                value: Expression::Constant {
                    value: number.get_literal_value().parse::<i64>().unwrap(),
                },
            };
        }

        self.parse_function_call()
    }

    fn parse_function_call(&mut self) -> Statement {
        if let Some(function_identifier) = self.consume_if_matched(vec![TokenType::Identifier]) {
            self.consume_required(TokenType::ParenthesesLeft);

            // TODO: handle expressions
            let variable_access = self.consume_required(TokenType::Identifier);

            self.consume_required(TokenType::ParenthesesRight);
            self.consume_required(TokenType::Semicolon);

            return Statement::Expression(Expression::Call {
                name: function_identifier.get_literal_value().to_string(),
                args: vec![Expression::VariableAccess {
                    name: variable_access.get_literal_value().to_string(),
                }],
            });
        }

        self.parse_statement()
    }

    fn parse_statement(&mut self) -> Statement {
        // TODO: This is a placeholder
        self.cursor += 1;
        Statement::Expression(Expression::Constant { value: 1 })
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

        return current;
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

        return None;
    }

    fn is_at_end(&self) -> bool {
        self.tokens[self.cursor].get_type() == &TokenType::Eof
    }
}
