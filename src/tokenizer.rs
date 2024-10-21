use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    r#type: TokenType,
    location: (usize, usize),
    length: usize,
    literal_value: String,
}

impl Token {
    pub fn get_type(&self) -> &TokenType {
        &self.r#type
    }

    pub fn get_literal_value(&self) -> &str {
        &self.literal_value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // 1 characer tokens
    ParenthesesLeft,
    ParenthesesRight,
    BraceLeft,
    BraceRight,
    Comma,
    Dot,
    Plus,
    Minus,
    Star,
    Slash,
    Semicolon,
    Not,

    // 2 character tokens
    Equals,
    EqualsEquals,
    NotEquals,
    GreaterThan,
    GreaterThanEquals,
    LessThan,
    LessThanEquals,
    ColonEquals,
    DoubleColon,
    ArrowRight,

    // multi character tokens
    Identifier,
    Number,
    String,
    Keyword(Keyword),

    Eof,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Let,
    Const,
}

impl Keyword {
    pub fn try_match_from_raw_value(raw: &str) -> Option<Keyword> {
        match raw {
            "let" => Some(Keyword::Let),
            "const" => Some(Keyword::Const),
            _ => None,
        }
    }
}

pub struct Tokenizer {
    source: String,
    cursor: usize,
    current_line: usize,
    current_column: usize,
    current_token_start: usize,
}

impl Tokenizer {
    pub fn new(source: String) -> Self {
        Self {
            source,
            cursor: 0,
            current_line: 1,
            current_column: 0,
            current_token_start: 0,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = vec![];

        while !self.is_at_end() {
            self.current_token_start = self.cursor;
            if let Some(token) = self.consume_token() {
                tokens.push(token);
            }
        }
        tokens.push(self.make_token(TokenType::Eof));

        tokens
    }

    fn consume_token(&mut self) -> Option<Token> {
        let mut c = self.consume_char();
        while c.is_whitespace() || c == '\n' || c == '\r' {
            if self.is_at_end() {
                return None;
            }

            if c == '\n' {
                self.current_line += 1;
                self.current_column = 1;
            }

            c = self.consume_char();
            self.current_token_start = self.cursor - 1;
        }

        let token = match c {
            '(' => self.make_token(TokenType::ParenthesesLeft),
            ')' => self.make_token(TokenType::ParenthesesRight),
            '{' => self.make_token(TokenType::BraceLeft),
            '}' => self.make_token(TokenType::BraceRight),
            ',' => self.make_token(TokenType::Comma),
            '.' => self.make_token(TokenType::Dot),
            '+' => self.make_token(TokenType::Plus),
            '*' => self.make_token(TokenType::Star),
            '/' => self.make_token(TokenType::Slash),
            ';' => self.make_token(TokenType::Semicolon),
            '-' => {
                if self.match_next_char('>') {
                    self.make_token(TokenType::ArrowRight)
                } else {
                    self.make_token(TokenType::Minus)
                }
            }
            '!' => {
                if self.match_next_char('=') {
                    self.make_token(TokenType::NotEquals)
                } else {
                    self.make_token(TokenType::Not)
                }
            }
            '=' => {
                if self.match_next_char('=') {
                    self.make_token(TokenType::EqualsEquals)
                } else {
                    self.make_token(TokenType::Equals)
                }
            }
            '>' => {
                if self.match_next_char('=') {
                    self.make_token(TokenType::GreaterThanEquals)
                } else {
                    self.make_token(TokenType::GreaterThan)
                }
            }
            '<' => {
                if self.match_next_char('=') {
                    self.make_token(TokenType::LessThanEquals)
                } else {
                    self.make_token(TokenType::LessThan)
                }
            }
            ':' => {
                if self.match_next_char('=') {
                    self.make_token(TokenType::ColonEquals)
                } else if self.match_next_char(':') {
                    self.make_token(TokenType::DoubleColon)
                } else {
                    panic!(
                        "Unexpected character {} at {}:{}",
                        c, self.current_line, self.current_column
                    );
                }
            }
            '"' => self.consume_string(),
            '0'..='9' => self.consume_number(),
            'a'..='z' | 'A'..='Z' | '_' => self.consume_identifier_or_keyword(),
            _ => {
                panic!(
                    "Unexpected character {} at {}:{}",
                    c, self.current_line, self.current_column
                );
            }
        };

        Some(token)
    }

    fn consume_char(&mut self) -> char {
        let c = self.source.as_bytes()[self.cursor] as char;

        self.cursor += 1;
        self.current_column += 1;

        c
    }

    fn consume_string(&mut self) -> Token {
        let mut is_terminated = false;

        while !self.is_at_end() {
            let c = self.consume_char();
            if c == '"' {
                is_terminated = true;
                break;
            };
        }

        if !is_terminated {
            panic!(
                "Unterminated string encountered, begins at {}:{}",
                self.current_line, self.current_token_start
            );
        }

        // start  + 1, because token start points at the opening quote
        // cursor - 1, because cursor points at the closing quote
        let raw_value = &self.source[self.current_token_start + 1..self.cursor - 1];
        Token {
            r#type: TokenType::String,
            location: (self.current_line, self.current_token_start),
            length: self.cursor - 1 - self.current_token_start,
            literal_value: raw_value.to_string(),
        }
    }

    // TODO: support floating point numbers
    // TODO: support alternative number formats: hex, binary, etc.
    fn consume_number(&mut self) -> Token {
        while !self.is_at_end() {
            let c = self.peek_next_char();
            if !c.is_digit(10) {
                break;
            }
            self.consume_char();
        }

        let length = self.cursor - self.current_token_start;
        let raw_value = &self.source[self.current_token_start..self.cursor];

        Token {
            r#type: TokenType::Number,
            location: (self.current_line, self.current_token_start),
            length,
            literal_value: raw_value.to_string(),
        }
    }

    fn consume_identifier_or_keyword(&mut self) -> Token {
        while !self.is_at_end() {
            let c = self.peek_next_char();

            let is_valid = match c {
                'a'..='z' | 'A'..='Z' | '_' => true,
                _ => false,
            };

            if !is_valid {
                break;
            }
            self.consume_char();
        }

        let length = self.cursor - self.current_token_start;
        let token_start = self.current_token_start;
        let raw_value = &self.source[token_start..self.cursor];

        let token_type = if let Some(keyword) = Keyword::try_match_from_raw_value(raw_value) {
            TokenType::Keyword(keyword)
        } else {
            TokenType::Identifier
        };

        Token {
            r#type: token_type,
            location: (self.current_line, self.current_token_start),
            length,
            literal_value: raw_value.to_string(),
        }
    }

    fn match_next_char(&mut self, wanted: char) -> bool {
        if self.peek_next_char() == wanted {
            self.consume_char();
            return true;
        }

        return false;
    }

    fn peek_next_char(&self) -> char {
        if self.is_at_end() {
            panic!();
        }

        self.source.as_bytes()[self.cursor] as char
    }

    fn is_at_end(&self) -> bool {
        self.cursor >= self.source.len()
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        let length = self.cursor - self.current_token_start;
        let literal_value = if token_type == TokenType::Eof {
            String::new()
        } else {
            self.source[self.current_token_start..self.cursor].to_string()
        };

        Token {
            r#type: token_type,
            location: (self.current_line, self.current_token_start),
            length,
            literal_value,
        }
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn tokenize_correctly() {
        // given
        let source = r#"
        !    != 


        ();


"#
        .to_string();

        let mut tokenizer = Tokenizer::new(source);

        // when
        let tokens = tokenizer.tokenize();

        // then
        let token_types: Vec<TokenType> = tokens.into_iter().map(|t| t.r#type).collect();
        assert_eq!(
            vec![
                TokenType::Not,
                TokenType::NotEquals,
                TokenType::ParenthesesLeft,
                TokenType::ParenthesesRight,
                TokenType::Semicolon,
                TokenType::Eof
            ],
            token_types
        );
    }

    #[test]
    fn tokenize_string() {
        // given
        let source = r#"
        ! "thisisastring()!@#$:: =>"

        "another string"
        "#
        .to_string();

        let mut tokenizer = Tokenizer::new(source);

        // when
        let tokens = tokenizer.tokenize();

        // then
        let token_types: Vec<&TokenType> = tokens.iter().map(|t| &t.r#type).collect();
        assert_eq!(
            vec![
                &TokenType::Not,
                &TokenType::String,
                &TokenType::String,
                &TokenType::Eof
            ],
            token_types
        );

        let raw_strings: Vec<&String> = tokens
            .iter()
            .filter(|t| t.r#type == TokenType::String)
            .map(|t| &t.literal_value)
            .collect();
        assert_eq!(
            vec!["thisisastring()!@#$:: =>", "another string"],
            raw_strings
        )
    }

    #[test]
    #[should_panic]
    fn report_error_on_unterminated_string() {
        // given
        let source = r#"! "valid string" "unterminated string !!!
            "#
        .to_string();

        let mut tokenizer = Tokenizer::new(source);

        // when & then
        tokenizer.tokenize();
    }

    #[test]
    fn tokenize_numbers() {
        // given
        let source = r#"=
 1234 5437"#
            .to_string();

        let mut tokenizer = Tokenizer::new(source);

        // when
        let tokens = tokenizer.tokenize();

        // then
        let token_types: Vec<&TokenType> = tokens.iter().map(|t| &t.r#type).collect();
        assert_eq!(
            vec![
                &TokenType::Equals,
                &TokenType::Number,
                &TokenType::Number,
                &TokenType::Eof
            ],
            token_types
        );

        let raw_numbers: Vec<&String> = tokens
            .iter()
            .filter(|t| t.r#type == TokenType::Number)
            .map(|t| &t.literal_value)
            .collect();
        assert_eq!(vec!["1234", "5437"], raw_numbers)
    }

    #[test]
    fn tokenize_identifier_and_keyword() {
        // given
        let source = r#"let number=1234;"#.to_string();

        let mut tokenizer = Tokenizer::new(source);

        // when
        let tokens = tokenizer.tokenize();

        // then
        let token_types: Vec<&TokenType> = tokens.iter().map(|t| &t.r#type).collect();
        assert_eq!(
            vec![
                &TokenType::Keyword(Keyword::Let),
                &TokenType::Identifier,
                &TokenType::Equals,
                &TokenType::Number,
                &TokenType::Semicolon,
                &TokenType::Eof
            ],
            token_types
        );

        let raw_values: Vec<&String> = tokens.iter().map(|t| &t.literal_value).collect();
        assert_eq!(vec!["let", "number", "=", "1234", ";", ""], raw_values)
    }
}
