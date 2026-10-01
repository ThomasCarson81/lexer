use std::env;
use std::fs;
use std::iter::Peekable;
use std::str::Chars;
use syntax::tokens::{Token, TokenType};

#[derive(Debug)]
enum LexError {
    InvalidCharacter(char, usize),
    InvalidNumberFormat(String, usize),
    UnterminatedString(usize),
    CouldNotReadFile(String),
    TokenStreamEnded(usize),
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 && args.len() != 3 {
        println!(
            "Please provide a file to tokenise, and optionally an output file to write the tokens to."
        );
        std::process::exit(1);
    }
    let tokens = match lex_file(&args[1]) {
        Ok(tokens) => tokens,
        Err(e) => {
            match e {
                LexError::InvalidCharacter(c, line) => {
                    println!("Line {line}: Invalid character: '{c}'");
                }
                LexError::InvalidNumberFormat(num, line) => {
                    println!("Line {line}: Invalid number format: {num}");
                }
                LexError::UnterminatedString(line) => {
                    println!("Line {line}: unteriminated string literal.")
                }
                LexError::CouldNotReadFile(msg) => {
                    println!("Lexer could not read file: {msg}");
                }
                LexError::TokenStreamEnded(line) => {
                    println!("Token stream ended unexpectedly on line {}. (lexing)", line)
                }
            }
            std::process::exit(1);
        }
    };
    if args.len() == 3 {
        let json_result = serde_json::to_string_pretty(&tokens);
        match json_result {
            Ok(json) => {
                output_tokens(&json, &args[2]);
            }
            Err(e) => {
                println!("Error serialising AST JSON: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        print_tokens(&tokens);
    }
}

fn lex_file(file_path: &str) -> Result<Vec<Token>, LexError> {
    let mut all_tokens: Vec<Token> = Vec::new();
    let file_content =
        fs::read_to_string(&file_path).map_err(|e| LexError::CouldNotReadFile(e.to_string()))?;
    let content_lines = file_content.lines();
    let total_lines = content_lines.clone().count();
    for (i, line) in content_lines.enumerate() {
        all_tokens.extend(lex_line(line, i + 1)?);
    }
    all_tokens.push(Token {
        token_type: TokenType::EndOfFile,
        lexeme: "".to_owned(),
        line: total_lines,
    });
    lex_keywords(&mut all_tokens);
    Ok(all_tokens)
}

fn lex_line(line: &str, line_number: usize) -> Result<Vec<Token>, LexError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut chars = line.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\n' | '\t' => {
                chars.next();
            }
            'a'..='z' | 'A'..='Z' => {
                let identifier = lex_identifier(&mut chars);
                tokens.push(Token {
                    token_type: TokenType::Identifier,
                    lexeme: identifier,
                    line: line_number,
                });
            }
            '0'..='9' => {
                let number = lex_number(&mut chars, line_number)?;
                tokens.push(Token {
                    token_type: TokenType::Number,
                    lexeme: number,
                    line: line_number,
                });
            }
            '=' => {
                chars.next();
                if *chars
                    .peek()
                    .ok_or(LexError::TokenStreamEnded(line_number))?
                    == '='
                {
                    chars.next();
                    tokens.push(Token {
                        token_type: TokenType::EqualTo,
                        lexeme: String::from("=="),
                        line: line_number,
                    });
                } else {
                    tokens.push(Token {
                        token_type: TokenType::Equals,
                        lexeme: String::from("="),
                        line: line_number,
                    });
                }
            }
            '+' => {
                chars.next();
                let next = chars
                    .peek()
                    .ok_or(LexError::TokenStreamEnded(line_number))?;
                if *next == '+' {
                    chars.next();
                    tokens.push(Token {
                        token_type: TokenType::Increment,
                        lexeme: String::from("++"),
                        line: line_number,
                    });
                } else if *next == '=' {
                    chars.next();
                    tokens.push(Token {
                        token_type: TokenType::PlusEquals,
                        lexeme: String::from("+="),
                        line: line_number,
                    });
                } else {
                    tokens.push(Token {
                        token_type: TokenType::Plus,
                        lexeme: String::from("+"),
                        line: line_number,
                    });
                }
            }
            '-' => {
                chars.next();
                let next = chars
                    .peek()
                    .ok_or(LexError::TokenStreamEnded(line_number))?;
                if *next == '-' {
                    chars.next();
                    tokens.push(Token {
                        token_type: TokenType::Decrement,
                        lexeme: String::from("--"),
                        line: line_number,
                    });
                } else if *next == '=' {
                    chars.next();
                    tokens.push(Token {
                        token_type: TokenType::MinusEquals,
                        lexeme: String::from("-="),
                        line: line_number,
                    });
                } else {
                    tokens.push(Token {
                        token_type: TokenType::Minus,
                        lexeme: String::from("-"),
                        line: line_number,
                    });
                }
            }
            '*' => {
                chars.next();
                if *chars
                    .peek()
                    .ok_or(LexError::TokenStreamEnded(line_number))?
                    == '='
                {
                    chars.next();
                    tokens.push(Token {
                        token_type: TokenType::AsteriskEquals,
                        lexeme: String::from("*="),
                        line: line_number,
                    })
                } else {
                    tokens.push(Token {
                        token_type: TokenType::Asterisk,
                        lexeme: String::from("*"),
                        line: line_number,
                    });
                }
            }
            '/' => {
                chars.next();
                if *chars
                    .peek()
                    .ok_or(LexError::TokenStreamEnded(line_number))?
                    == '='
                {
                    chars.next();
                    tokens.push(Token {
                        token_type: TokenType::SlashEquals,
                        lexeme: String::from("/="),
                        line: line_number,
                    })
                } else if *chars
                    .peek()
                    .ok_or(LexError::TokenStreamEnded(line_number))?
                    == '/'
                {
                    return Ok(tokens); // skip the rest of the line
                } else {
                    tokens.push(Token {
                        token_type: TokenType::Slash,
                        lexeme: String::from("/"),
                        line: line_number,
                    });
                }
            }
            '%' => {
                chars.next();
                if *chars
                    .peek()
                    .ok_or(LexError::TokenStreamEnded(line_number))?
                    == '='
                {
                    chars.next();
                    tokens.push(Token {
                        token_type: TokenType::ModuloEquals,
                        lexeme: String::from("%="),
                        line: line_number,
                    })
                } else {
                    tokens.push(Token {
                        token_type: TokenType::Modulo,
                        lexeme: String::from("%"),
                        line: line_number,
                    });
                }
            }
            '>' => {
                chars.next();
                if *chars
                    .peek()
                    .ok_or(LexError::TokenStreamEnded(line_number))?
                    == '='
                {
                    chars.next();
                    tokens.push(Token {
                        token_type: TokenType::GreaterThanOrEqualTo,
                        lexeme: String::from(">="),
                        line: line_number,
                    })
                } else {
                    tokens.push(Token {
                        token_type: TokenType::GreaterThan,
                        lexeme: String::from(">"),
                        line: line_number,
                    });
                }
            }
            '<' => {
                chars.next();
                if *chars
                    .peek()
                    .ok_or(LexError::TokenStreamEnded(line_number))?
                    == '='
                {
                    chars.next();
                    tokens.push(Token {
                        token_type: TokenType::LessThanOrEqualTo,
                        lexeme: String::from("<="),
                        line: line_number,
                    })
                } else {
                    tokens.push(Token {
                        token_type: TokenType::LessThan,
                        lexeme: String::from("<"),
                        line: line_number,
                    });
                }
            }
            '!' => {
                chars.next();
                if *chars
                    .peek()
                    .ok_or(LexError::TokenStreamEnded(line_number))?
                    == '='
                {
                    chars.next();
                    tokens.push(Token {
                        token_type: TokenType::NotEqualTo,
                        lexeme: String::from("!="),
                        line: line_number,
                    })
                } else {
                    tokens.push(Token {
                        token_type: TokenType::Not,
                        lexeme: String::from("!"),
                        line: line_number,
                    })
                }
            }
            '&' => {
                chars.next();
                if *chars
                    .peek()
                    .ok_or(LexError::TokenStreamEnded(line_number))?
                    == '&'
                {
                    chars.next();
                    tokens.push(Token {
                        token_type: TokenType::LogicalAnd,
                        lexeme: String::from("&&"),
                        line: line_number,
                    })
                }
            }
            '|' => {
                chars.next();
                if *chars
                    .peek()
                    .ok_or(LexError::TokenStreamEnded(line_number))?
                    == '|'
                {
                    chars.next();
                    tokens.push(Token {
                        token_type: TokenType::LogicalOr,
                        lexeme: String::from("||"),
                        line: line_number,
                    })
                }
            }
            '(' => {
                chars.next();
                tokens.push(Token {
                    token_type: TokenType::LeftBracket,
                    lexeme: String::from("("),
                    line: line_number,
                });
            }
            ')' => {
                chars.next();
                tokens.push(Token {
                    token_type: TokenType::RightBracket,
                    lexeme: String::from(")"),
                    line: line_number,
                });
            }
            '{' => {
                chars.next();
                tokens.push(Token {
                    token_type: TokenType::LeftSquiggly,
                    lexeme: String::from("{"),
                    line: line_number,
                });
            }
            '}' => {
                chars.next();
                tokens.push(Token {
                    token_type: TokenType::RightSquiggly,
                    lexeme: String::from("}"),
                    line: line_number,
                });
            }
            ';' => {
                chars.next();
                tokens.push(Token {
                    token_type: TokenType::Semicolon,
                    lexeme: String::from(";"),
                    line: line_number,
                });
            }
            ':' => {
                chars.next();
                if *chars
                    .peek()
                    .ok_or(LexError::TokenStreamEnded(line_number))?
                    == '='
                {
                    chars.next();
                    tokens.push(Token {
                        token_type: TokenType::Walrus,
                        lexeme: String::from(":="),
                        line: line_number,
                    })
                } else {
                    tokens.push(Token {
                        token_type: TokenType::Colon,
                        lexeme: String::from(":"),
                        line: line_number,
                    });
                }
            }
            ',' => {
                chars.next();
                tokens.push(Token {
                    token_type: TokenType::Comma,
                    lexeme: String::from(","),
                    line: line_number,
                });
            }
            '.' => {
                chars.next();
                tokens.push(Token {
                    token_type: TokenType::Dot,
                    lexeme: String::from("."),
                    line: line_number,
                });
            }
            '"' => {
                let string_literal = lex_string(&mut chars, line_number)?;
                tokens.push(Token {
                    token_type: TokenType::String,
                    lexeme: string_literal,
                    line: line_number,
                });
            }
            _ => {
                let unrecognized_char = chars.next().unwrap_or('?');
                return Err(LexError::InvalidCharacter(unrecognized_char, line_number));
            }
        }
    }
    Ok(tokens)
}

fn lex_identifier(chars: &mut Peekable<Chars>) -> String {
    let mut identifier = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_alphanumeric() || c == '_' {
            identifier.push(c);
            chars.next();
        } else {
            break;
        }
    }
    identifier
}

fn lex_number(chars: &mut Peekable<Chars>, line_number: usize) -> Result<String, LexError> {
    let mut number: String = String::new();
    let mut valid = true;
    while let Some(&c) = chars.peek() {
        if c.is_digit(10) || c == '.' {
            if c == '.' && number.contains('.') {
                valid = false;
            }
            number.push(c);
            chars.next();
        } else {
            break;
        }
    }
    if !valid {
        return Err(LexError::InvalidNumberFormat(number, line_number));
    }
    Ok(number)
}

fn lex_string(chars: &mut Peekable<Chars>, line_number: usize) -> Result<String, LexError> {
    let mut str_lit: String = String::new();
    chars.next(); // consume first "
    let mut ended: bool = false;
    while let Some(&c) = chars.peek() {
        if c == '"' {
            chars.next();
            ended = true;
            break;
        }
        str_lit.push(c);
        chars.next();
    }
    if !ended {
        return Err(LexError::UnterminatedString(line_number));
    }
    Ok(str_lit)
}

fn lex_keywords(tokens: &mut Vec<Token>) {
    for token in tokens {
        if token.token_type == TokenType::Identifier {
            token.token_type = match token.lexeme.as_str() {
                "let" => TokenType::Let,
                "if" => TokenType::If,
                "else" => TokenType::Else,
                "for" => TokenType::For,
                "while" => TokenType::While,
                "fn" => TokenType::Fn,
                "return" => TokenType::Return,
                "break" => TokenType::Break,
                "continue" => TokenType::Continue,
                "true" => TokenType::True,
                "false" => TokenType::False,
                "scope" => TokenType::Scope,
                "mut" => TokenType::Mut,
                "struct" => TokenType::Struct,
                "enum" => TokenType::Enum,

                _ => TokenType::Identifier,
            }
        }
    }
}

fn print_tokens(tokens: &Vec<Token>) {
    for token in tokens {
        println!("{}: {} ({})", token.line, token.token_type, token.lexeme)
    }
}
fn output_tokens(json: &str, output_file: &str) {
    fs::write(output_file, json).expect("Unable to write to file");
}
