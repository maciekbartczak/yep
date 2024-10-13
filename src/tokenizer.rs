struct Token {
    r#type: TokenType,
    location: (usize, usize),
    length: usize,
    literal_value: String,
}

enum TokenType {
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

    Identifier,
    Number,
    String,

    Keyword(Keyword),

    Eof,
}

enum Keyword {
    Let,
    Const,
}

struct Tokenizer {
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
            tokens.push(self.consume_token());
        }

        tokens
    }

    fn consume_token(&mut self) -> Token {
        let mut c = self.consume_char();
        while c.is_whitespace() && !self.is_at_end() {
            if c == '\n' {
                self.current_line += 1;
                self.current_column = 1;
            }

            c = self.consume_char();
            self.current_token_start = self.cursor;
        }

        match c {
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
            _ => todo!(),
        }
    }

    fn consume_char(&mut self) -> char {
        let c = self.source.as_bytes()[self.cursor] as char;

        self.cursor += 1;
        self.current_column += 1;

        c
    }

    fn is_at_end(&self) -> bool {
        self.cursor >= self.source.len()
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        Token {
            r#type: token_type,
            location: (self.current_line, self.current_token_start),
            length: self.cursor - self.current_token_start,
            literal_value: self.source[self.current_token_start..self.cursor].to_string(),
        }
    }
}
