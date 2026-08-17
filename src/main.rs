use std::env;
use std::fmt;
use std::fs;
use std::iter::Peekable;
use std::str::Chars;

struct Token {
    token_type: TokenType,
    lexeme: String,
    line: usize,
}
#[derive(PartialEq)]
enum TokenType {
    Identifier,
    Number,

    Equals,
    Plus,
    Minus,
    Asterisk,
    Slash,

    LeftBracket,
    RightBracket,

    LeftSquiggly,
    RightSquiggly,

    LetTok,
    IfTok,
    ElseTok,
    ForTok,
    WhileTok,

    Semicolon,

    EndOfFile,
}
impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenType::Identifier => write!(f, "Identifier"),
            TokenType::Number => write!(f, "Number"),

            TokenType::Equals => write!(f, "Equals"),
            TokenType::Plus => write!(f, "Plus"),
            TokenType::Minus => write!(f, "Minus"),
            TokenType::Asterisk => write!(f, "Asterisk"),
            TokenType::Slash => write!(f, "Slash"),

            TokenType::LeftBracket => write!(f, "LeftBracket"),
            TokenType::RightBracket => write!(f, "RightBracket"),

            TokenType::LeftSquiggly => write!(f, "LeftSquiggly"),
            TokenType::RightSquiggly => write!(f, "RightSquiggly"),

            TokenType::LetTok => write!(f, "LetTok"),
            TokenType::IfTok => write!(f, "IfTok"),
            TokenType::ElseTok => write!(f, "ElseTok"),
            TokenType::ForTok => write!(f, "ForTok"),
            TokenType::WhileTok => write!(f, "WhileTok"),

            TokenType::Semicolon => write!(f, "Semicolon"),

            TokenType::EndOfFile => write!(f, "EndOfFile"),
        }
    }
}
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Please provide 1 file to tokenise");
        return;
    }
    let mut all_tokens: Vec<Token> = Vec::new();
    let file_content = fs::read_to_string(&args[1]).expect("failed to read file");
    let content_lines = file_content.lines();
    let total_lines = content_lines.clone().count();
    for (i, line) in content_lines.enumerate() {
        all_tokens.extend(parse_line(line, i + 1));
    }
    all_tokens.push(Token {
        token_type: TokenType::EndOfFile,
        lexeme: "".to_owned(),
        line: total_lines,
    });
    parse_keywords(&mut all_tokens);

    print_tokens(&all_tokens);
}

fn parse_line(line: &str, line_number: usize) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut chars = line.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\n' => {
                chars.next();
            }
            'a'..='z' | 'A'..='Z' => {
                let identifier = parse_identifier(&mut chars);
                tokens.push(Token {
                    token_type: TokenType::Identifier,
                    lexeme: identifier,
                    line: line_number,
                });
            }
            '0'..='9' => {
                let number = parse_number(&mut chars);
                tokens.push(Token {
                    token_type: TokenType::Number,
                    lexeme: number,
                    line: line_number,
                });
            }
            '=' => {
                chars.next();
                tokens.push(Token {
                    token_type: TokenType::Equals,
                    lexeme: String::from("="),
                    line: line_number,
                });
            }
            '+' => {
                chars.next();
                tokens.push(Token {
                    token_type: TokenType::Plus,
                    lexeme: String::from("+"),
                    line: line_number,
                });
            }
            '-' => {
                chars.next();
                tokens.push(Token {
                    token_type: TokenType::Minus,
                    lexeme: String::from("-"),
                    line: line_number,
                });
            }
            '*' => {
                chars.next();
                tokens.push(Token {
                    token_type: TokenType::Asterisk,
                    lexeme: String::from("*"),
                    line: line_number,
                });
            }
            '/' => {
                chars.next();
                tokens.push(Token {
                    token_type: TokenType::Slash,
                    lexeme: String::from("/"),
                    line: line_number,
                });
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
            _ => {
                // Skip unrecognized characters
                println!("Unrecognized character: '{}'", chars.next().unwrap());
            }
        }
    }
    tokens
}

fn parse_identifier(chars: &mut Peekable<Chars>) -> String {
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

fn parse_number(chars: &mut Peekable<Chars>) -> String {
    let mut number: String = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_digit(10) {
            number.push(c);
            chars.next();
        } else {
            break;
        }
    }
    number
}

fn parse_keywords(tokens: &mut Vec<Token>) {
    for token in tokens {
        if token.token_type == TokenType::Identifier {
            token.token_type = match token.lexeme.as_str() {
                "let" => TokenType::LetTok,
                "if" => TokenType::IfTok,
                "else" => TokenType::ElseTok,
                "for" => TokenType::ForTok,
                "while" => TokenType::WhileTok,
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
