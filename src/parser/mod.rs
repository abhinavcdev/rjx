//! Parser module for GQ query language
//! 
//! This module handles parsing of query expressions similar to jq syntax
//! but with a focus on performance and simplicity.

use thiserror::Error;
use std::fmt;

/// Error type for query parsing failures
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("syntax error: {0}")]
    Syntax(String),
    
    #[error("unexpected token: {0}")]
    UnexpectedToken(String),
    
    #[error("unexpected end of input")]
    UnexpectedEof,
    
    #[error("invalid filter: {0}")]
    InvalidFilter(String),
}

/// Token types for the query language lexer
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Dot,               // .
    DotDot,            // ..
    Pipe,              // |
    Comma,             // ,
    LeftBracket,       // [
    RightBracket,      // ]
    LeftBrace,         // {
    RightBrace,        // }
    Colon,             // :
    Question,          // ?
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(f64),
    BoolLiteral(bool),
    Null,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Dot => write!(f, "."),
            Token::DotDot => write!(f, ".."),
            Token::Pipe => write!(f, "|"),
            Token::Comma => write!(f, ","),
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),
            Token::LeftBrace => write!(f, "{{"),
            Token::RightBrace => write!(f, "}}"),
            Token::Colon => write!(f, ":"),
            Token::Question => write!(f, "?"),
            Token::Identifier(s) => write!(f, "{}", s),
            Token::StringLiteral(s) => write!(f, "\"{}\"", s),
            Token::NumberLiteral(n) => write!(f, "{}", n),
            Token::BoolLiteral(b) => write!(f, "{}", b),
            Token::Null => write!(f, "null"),
        }
    }
}

/// Lexer for tokenizing query strings
pub struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    /// Create a new lexer from a query string
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
        }
    }
    
    /// Get the current character or None if at end of input
    fn current_char(&self) -> Option<char> {
        if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        }
    }
    
    /// Advance to the next character
    fn advance(&mut self) {
        self.position += 1;
    }
    
    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char() {
            if !c.is_whitespace() {
                break;
            }
            self.advance();
        }
    }
    
    /// Tokenize the input string into a vector of tokens
    pub fn tokenize(&mut self) -> Result<Vec<Token>, ParseError> {
        let mut tokens = Vec::new();
        
        while let Some(c) = self.current_char() {
            match c {
                '.' => {
                    self.advance();
                    if self.current_char() == Some('.') {
                        self.advance();
                        tokens.push(Token::DotDot);
                    } else {
                        tokens.push(Token::Dot);
                    }
                },
                '|' => {
                    self.advance();
                    tokens.push(Token::Pipe);
                },
                ',' => {
                    self.advance();
                    tokens.push(Token::Comma);
                },
                '[' => {
                    self.advance();
                    tokens.push(Token::LeftBracket);
                },
                ']' => {
                    self.advance();
                    tokens.push(Token::RightBracket);
                },
                '{' => {
                    self.advance();
                    tokens.push(Token::LeftBrace);
                },
                '}' => {
                    self.advance();
                    tokens.push(Token::RightBrace);
                },
                ':' => {
                    self.advance();
                    tokens.push(Token::Colon);
                },
                '?' => {
                    self.advance();
                    tokens.push(Token::Question);
                },
                '"' => {
                    tokens.push(self.read_string()?);
                },
                c if c.is_ascii_digit() || c == '-' => {
                    tokens.push(self.read_number()?);
                },
                c if c.is_alphabetic() || c == '_' => {
                    tokens.push(self.read_identifier()?);
                },
                c if c.is_whitespace() => {
                    self.skip_whitespace();
                },
                _ => {
                    return Err(ParseError::Syntax(format!("unexpected character: {}", c)));
                }
            }
        }
        
        Ok(tokens)
    }
    
    /// Read a string literal
    fn read_string(&mut self) -> Result<Token, ParseError> {
        self.advance(); // Skip opening quote
        let mut value = String::new();
        
        while let Some(c) = self.current_char() {
            match c {
                '"' => {
                    self.advance(); // Skip closing quote
                    return Ok(Token::StringLiteral(value));
                },
                '\\' => {
                    self.advance();
                    match self.current_char() {
                        Some('"') => value.push('"'),
                        Some('\\') => value.push('\\'),
                        Some('n') => value.push('\n'),
                        Some('r') => value.push('\r'),
                        Some('t') => value.push('\t'),
                        Some(c) => value.push(c),
                        None => return Err(ParseError::UnexpectedEof),
                    }
                    self.advance();
                },
                _ => {
                    value.push(c);
                    self.advance();
                }
            }
        }
        
        Err(ParseError::UnexpectedEof)
    }
    
    /// Read a number literal
    fn read_number(&mut self) -> Result<Token, ParseError> {
        let mut value = String::new();
        
        // Handle negative numbers
        if self.current_char() == Some('-') {
            value.push('-');
            self.advance();
        }
        
        // Read integer part
        while let Some(c) = self.current_char() {
            if c.is_ascii_digit() {
                value.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        // Read decimal part if present
        if self.current_char() == Some('.') {
            value.push('.');
            self.advance();
            
            let mut has_decimal_digits = false;
            while let Some(c) = self.current_char() {
                if c.is_ascii_digit() {
                    value.push(c);
                    self.advance();
                    has_decimal_digits = true;
                } else {
                    break;
                }
            }
            
            if !has_decimal_digits {
                return Err(ParseError::Syntax("invalid number format".to_string()));
            }
        }
        
        // Parse the number
        match value.parse::<f64>() {
            Ok(n) => Ok(Token::NumberLiteral(n)),
            Err(_) => Err(ParseError::Syntax("invalid number format".to_string())),
        }
    }
    
    /// Read an identifier or keyword
    fn read_identifier(&mut self) -> Result<Token, ParseError> {
        let mut value = String::new();
        
        while let Some(c) = self.current_char() {
            if c.is_alphanumeric() || c == '_' {
                value.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        // Check for keywords
        match value.as_str() {
            "true" => Ok(Token::BoolLiteral(true)),
            "false" => Ok(Token::BoolLiteral(false)),
            "null" => Ok(Token::Null),
            _ => Ok(Token::Identifier(value)),
        }
    }
}

/// Represents a parsed query expression
#[derive(Debug, Clone)]
pub enum Expression {
    Identity,                          // .
    RecursiveDescent,                  // ..
    Property(String),                  // .property_name or ."property name"
    Index(i64),                        // .[0]
    Slice(Option<i64>, Option<i64>),   // .[1:3]
    Array(Vec<Expression>),            // [expr1, expr2, ...]
    Object(Vec<(String, Expression)>), // {key1: expr1, key2: expr2, ...}
    Pipe(Box<Expression>, Box<Expression>), // expr1 | expr2
    Filter(Box<Expression>),           // .[] | select(...)
    ArrayIteration,                    // .[]
    Select(Box<Expression>, String, Box<Expression>), // select(.field == "value")
    Map(Box<Expression>),              // map(expr)
    Keys,                              // keys
    Length,                            // length
}

/// Parser for query expressions
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    /// Create a new parser from a vector of tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }
    
    /// Parse the tokens into an expression
    pub fn parse(&mut self) -> Result<Expression, ParseError> {
        self.parse_expression()
    }
    
    /// Get the current token or None if at end of tokens
    fn current_token(&self) -> Option<&Token> {
        if self.position < self.tokens.len() {
            Some(&self.tokens[self.position])
        } else {
            None
        }
    }
    
    /// Advance to the next token
    fn advance(&mut self) {
        self.position += 1;
    }
    
    /// Parse an expression
    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        // Start with a simple expression
        let mut expr = self.parse_simple_expression()?;
        
        // Check for pipe operator
        while let Some(Token::Pipe) = self.current_token() {
            self.advance();
            let right = self.parse_simple_expression()?;
            expr = Expression::Pipe(Box::new(expr), Box::new(right));
        }
        
        Ok(expr)
    }
    
    /// Parse a simple expression (without pipes)
    fn parse_simple_expression(&mut self) -> Result<Expression, ParseError> {
        match self.current_token() {
            Some(Token::Dot) => {
                self.advance();
                
                // Check if it's just the identity operator
                if self.current_token().is_none() || 
                   matches!(self.current_token(), Some(Token::Pipe) | Some(Token::Comma) | Some(Token::RightBracket) | Some(Token::RightBrace)) {
                    return Ok(Expression::Identity);
                }
                
                // Check for property access
                match self.current_token() {
                    Some(Token::Identifier(name)) => {
                        let name = name.clone();
                        self.advance();
                        
                        // Check for nested property access (.address.city)
                        let mut expr = Expression::Property(name);
                        while let Some(Token::Dot) = self.current_token() {
                            self.advance();
                            match self.current_token() {
                                Some(Token::Identifier(nested_name)) => {
                                    let nested_name = nested_name.clone();
                                    self.advance();
                                    expr = Expression::Pipe(
                                        Box::new(expr),
                                        Box::new(Expression::Property(nested_name))
                                    );
                                },
                                Some(Token::StringLiteral(nested_name)) => {
                                    let nested_name = nested_name.clone();
                                    self.advance();
                                    expr = Expression::Pipe(
                                        Box::new(expr),
                                        Box::new(Expression::Property(nested_name))
                                    );
                                },
                                _ => break,
                            }
                        }
                        
                        Ok(expr)
                    },
                    Some(Token::StringLiteral(name)) => {
                        let name = name.clone();
                        self.advance();
                        
                        // Check for nested property access (."address"."city")
                        let mut expr = Expression::Property(name);
                        while let Some(Token::Dot) = self.current_token() {
                            self.advance();
                            match self.current_token() {
                                Some(Token::Identifier(nested_name)) => {
                                    let nested_name = nested_name.clone();
                                    self.advance();
                                    expr = Expression::Pipe(
                                        Box::new(expr),
                                        Box::new(Expression::Property(nested_name))
                                    );
                                },
                                Some(Token::StringLiteral(nested_name)) => {
                                    let nested_name = nested_name.clone();
                                    self.advance();
                                    expr = Expression::Pipe(
                                        Box::new(expr),
                                        Box::new(Expression::Property(nested_name))
                                    );
                                },
                                _ => break,
                            }
                        }
                        
                        Ok(expr)
                    },
                    Some(Token::LeftBracket) => {
                        self.advance();
                        
                        // Parse array index or slice
                        match self.current_token() {
                            // Handle array iteration .[]
                            Some(Token::RightBracket) => {
                                self.advance();
                                Ok(Expression::ArrayIteration)
                            },
                            Some(Token::NumberLiteral(n)) => {
                                let index = *n as i64;
                                self.advance();
                                
                                if let Some(Token::Colon) = self.current_token() {
                                    self.advance();
                                    
                                    // Parse end of slice
                                    let end = match self.current_token() {
                                        Some(Token::NumberLiteral(n)) => {
                                            let end = *n as i64;
                                            self.advance();
                                            Some(end)
                                        },
                                        _ => None,
                                    };
                                    
                                    self.expect_token(&Token::RightBracket)?;
                                    
                                    // Check for nested property access (.[0].name)
                                    let mut expr = Expression::Slice(Some(index), end);
                                    if let Some(Token::Dot) = self.current_token() {
                                        self.advance();
                                        if let Some(Token::Identifier(nested_name)) = self.current_token() {
                                            let nested_name = nested_name.clone();
                                            self.advance();
                                            expr = Expression::Pipe(
                                                Box::new(expr),
                                                Box::new(Expression::Property(nested_name))
                                            );
                                        }
                                    }
                                    
                                    Ok(expr)
                                } else {
                                    self.expect_token(&Token::RightBracket)?;
                                    
                                    // Check for nested property access (.[0].name)
                                    let mut expr = Expression::Index(index);
                                    while let Some(Token::Dot) = self.current_token() {
                                        self.advance();
                                        match self.current_token() {
                                            Some(Token::Identifier(nested_name)) => {
                                                let nested_name = nested_name.clone();
                                                self.advance();
                                                expr = Expression::Pipe(
                                                    Box::new(expr),
                                                    Box::new(Expression::Property(nested_name))
                                                );
                                            },
                                            Some(Token::StringLiteral(nested_name)) => {
                                                let nested_name = nested_name.clone();
                                                self.advance();
                                                expr = Expression::Pipe(
                                                    Box::new(expr),
                                                    Box::new(Expression::Property(nested_name))
                                                );
                                            },
                                            _ => break,
                                        }
                                    }
                                    
                                    Ok(expr)
                                }
                            },
                            Some(Token::Colon) => {
                                self.advance();
                                
                                // Parse end of slice
                                let end = match self.current_token() {
                                    Some(Token::NumberLiteral(n)) => {
                                        let end = *n as i64;
                                        self.advance();
                                        Some(end)
                                    },
                                    _ => None,
                                };
                                
                                self.expect_token(&Token::RightBracket)?;
                                Ok(Expression::Slice(None, end))
                            },
                            _ => {
                                Err(ParseError::Syntax("expected number, colon, or closing bracket in array access".to_string()))
                            }
                        }
                    },
                    _ => {
                        Err(ParseError::Syntax("expected property name or array access after dot".to_string()))
                    }
                }
            },
            Some(Token::DotDot) => {
                self.advance();
                Ok(Expression::RecursiveDescent)
            },
            Some(Token::LeftBracket) => {
                self.advance();
                let mut elements = Vec::new();
                
                // Parse array elements
                if let Some(Token::RightBracket) = self.current_token() {
                    self.advance();
                    return Ok(Expression::Array(elements));
                }
                
                loop {
                    let element = self.parse_expression()?;
                    elements.push(element);
                    
                    match self.current_token() {
                        Some(Token::Comma) => {
                            self.advance();
                        },
                        Some(Token::RightBracket) => {
                            self.advance();
                            break;
                        },
                        _ => {
                            return Err(ParseError::Syntax("expected comma or closing bracket in array".to_string()));
                        }
                    }
                }
                
                Ok(Expression::Array(elements))
            },
            Some(Token::LeftBrace) => {
                self.advance();
                let mut properties = Vec::new();
                
                // Parse object properties
                if let Some(Token::RightBrace) = self.current_token() {
                    self.advance();
                    return Ok(Expression::Object(properties));
                }
                
                loop {
                    // Parse property key
                    let key = match self.current_token() {
                        Some(Token::Identifier(name)) => {
                            let name = name.clone();
                            self.advance();
                            name
                        },
                        Some(Token::StringLiteral(name)) => {
                            let name = name.clone();
                            self.advance();
                            name
                        },
                        _ => {
                            return Err(ParseError::Syntax("expected property name in object".to_string()));
                        }
                    };
                    
                    // Expect colon
                    self.expect_token(&Token::Colon)?;
                    
                    // Parse property value
                    let value = self.parse_expression()?;
                    properties.push((key, value));
                    
                    match self.current_token() {
                        Some(Token::Comma) => {
                            self.advance();
                        },
                        Some(Token::RightBrace) => {
                            self.advance();
                            break;
                        },
                        _ => {
                            return Err(ParseError::Syntax("expected comma or closing brace in object".to_string()));
                        }
                    }
                }
                
                Ok(Expression::Object(properties))
            },
            _ => {
                Err(ParseError::Syntax("unexpected token".to_string()))
            }
        }
    }
    
    /// Expect a specific token and advance if found
    fn expect_token(&mut self, expected: &Token) -> Result<(), ParseError> {
        match self.current_token() {
            Some(token) if token == expected => {
                self.advance();
                Ok(())
            },
            Some(token) => {
                Err(ParseError::UnexpectedToken(format!("expected {:?}, got {:?}", expected, token)))
            },
            None => {
                Err(ParseError::UnexpectedEof)
            }
        }
    }
}

/// Find the position of the matching closing parenthesis
fn find_matching_paren(s: &str) -> Option<usize> {
    let chars: Vec<char> = s.chars().collect();
    let mut depth = 0;
    
    for (i, &c) in chars.iter().enumerate() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            },
            _ => {}
        }
    }
    
    None
}

/// Parse a query string into an expression
pub fn parse_query(query: &str) -> Result<Expression, ParseError> {
    // Handle string literals in quotes
    if query.starts_with('"') && query.ends_with('"') && query.len() >= 2 {
        let content = &query[1..query.len()-1];
        return Ok(Expression::Property(content.to_string()));
    }
    
    // Special case for array iteration like '.resources[]'
    if query.ends_with("[]") {
        let base_part = &query[0..query.len()-2];
        if !base_part.is_empty() {
            let base_expr = parse_query(base_part)?;
            return Ok(Expression::Pipe(
                Box::new(base_expr),
                Box::new(Expression::ArrayIteration)
            ));
        }
    }
    
    // Special case for map operation like '.resources[] | map(.type)'
    if query.contains(" | map(") && query.contains(")") {
        if let Some(pipe_pos) = query.find(" | map(") {
            let left_part = &query[0..pipe_pos];
            let map_part = &query[pipe_pos + 7..];
            
            if map_part.ends_with(')') {
                let map_expr_str = &map_part[0..map_part.len()-1];
                
                // Parse the left part of the pipe
                let outer_expr = parse_query(left_part)?;
                
                // Parse the map expression
                let map_expr = parse_query(map_expr_str)?;
                
                return Ok(Expression::Pipe(
                    Box::new(outer_expr),
                    Box::new(Expression::Map(Box::new(map_expr)))
                ));
            }
        }
    }
    
    // Special case for keys operation like '.resources | keys'
    if query.contains(" | keys") {
        if let Some(pipe_pos) = query.find(" | keys") {
            let left_part = &query[0..pipe_pos];
            
            // Parse the left part of the pipe
            let left_expr = parse_query(left_part)?;
            
            return Ok(Expression::Pipe(
                Box::new(left_expr),
                Box::new(Expression::Keys)
            ));
        }
    }
    
    // Special case for length operation like '.resources | length'
    if query.contains(" | length") {
        if let Some(pipe_pos) = query.find(" | length") {
            let left_part = &query[0..pipe_pos];
            
            // Parse the left part of the pipe
            let left_expr = parse_query(left_part)?;
            
            return Ok(Expression::Pipe(
                Box::new(left_expr),
                Box::new(Expression::Length)
            ));
        }
    }
    
    // Special case for select expressions with chained operations
    // like '.resources[] | select(.type == "aws_instance") | .instances[].attributes.id'
    if query.contains(" | select(") {
        if let Some(pipe_pos) = query.find(" | select(") {
            let left_part = &query[0..pipe_pos];
            let remaining = &query[pipe_pos + 10..];
            
            // Find the closing parenthesis for select
            if let Some(close_paren) = find_matching_paren(remaining) {
                let condition = &remaining[0..close_paren];
                
                // Check if there are more operations after select
                let has_more_ops = close_paren + 1 < remaining.len() && remaining[close_paren+1..].contains(" | ");
                
                // Handle common comparison operators: ==, !=, >, <, >=, <=
                for op in &["==", "!=", ">", "<", ">=", "<="] {
                    if condition.contains(op) {
                        if let Some(op_pos) = condition.find(op) {
                            let left_expr_str = &condition[0..op_pos].trim();
                            let right_expr_str = &condition[op_pos + op.len()..].trim();
                            
                            // Parse the left part of the pipe
                            let outer_expr = parse_query(left_part)?;
                            
                            // Parse the left and right expressions of the condition
                            let left_expr = parse_query(left_expr_str)?;
                            
                            // Handle string literals in the right expression
                            let right_expr = if right_expr_str.starts_with('"') && right_expr_str.ends_with('"') {
                                let content = &right_expr_str[1..right_expr_str.len()-1];
                                Expression::Property(content.to_string())
                            } else {
                                parse_query(right_expr_str)?
                            };
                            
                            let select_expr = Expression::Pipe(
                                Box::new(outer_expr),
                                Box::new(Expression::Select(
                                    Box::new(left_expr),
                                    op.to_string(),
                                    Box::new(right_expr)
                                ))
                            );
                            
                            // If there are more operations, parse them
                            if has_more_ops {
                                let next_pipe_pos = remaining[close_paren+1..].find(" | ").unwrap();
                                let next_ops = &remaining[close_paren+1+next_pipe_pos..];
                                let next_expr = parse_query(next_ops)?;
                                
                                return Ok(Expression::Pipe(
                                    Box::new(select_expr),
                                    Box::new(next_expr)
                                ));
                            } else {
                                return Ok(select_expr);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Special case for object construction like '.address | {city, state}'
    if query.contains(" | {") && query.contains("}") {
        if let Some(pipe_pos) = query.find(" | {") {
            let left_part = &query[0..pipe_pos];
            let right_part = &query[pipe_pos+3..];
            
            if right_part.starts_with('{') && right_part.ends_with('}') {
                let obj_content = &right_part[1..right_part.len()-1];
                let fields: Vec<&str> = obj_content.split(',').map(|s| s.trim()).collect();
                
                // Parse the left part of the pipe
                let left_expr = parse_query(left_part)?;
                
                // Create object construction with fields
                let mut properties = Vec::new();
                for field in fields {
                    properties.push((field.to_string(), Expression::Property(field.to_string())));
                }
                
                let obj_expr = Expression::Object(properties);
                
                return Ok(Expression::Pipe(
                    Box::new(left_expr),
                    Box::new(obj_expr)
                ));
            }
        }
    }
    
    // Special case for array indexing with property access
    if query.contains('[') && query.contains(']') {
        // Handle simple array indexing like .tags[1]
        if let Some(first_dot) = query.find('.') {
            if let Some(bracket_start) = query.find('[') {
                if first_dot < bracket_start {
                    let property = &query[first_dot+1..bracket_start];
                    if let Some(bracket_end) = query[bracket_start..].find(']') {
                        let bracket_end = bracket_start + bracket_end + 1;
                        
                        // Check if this is a pattern like .phones[0].number
                        if bracket_end < query.len() && query[bracket_end..].contains('.') {
                            let second_dot = bracket_end + query[bracket_end..].find('.').unwrap();
                            let index_str = &query[bracket_start+1..bracket_end-1];
                            if let Ok(index) = index_str.parse::<i64>() {
                                let nested_property = &query[second_dot+1..];
                                
                                // Create a pipe expression: .property | .[index] | .nested_property
                                let property_expr = Expression::Property(property.to_string());
                                let index_expr = Expression::Index(index);
                                let nested_expr = Expression::Property(nested_property.to_string());
                                
                                let pipe1 = Expression::Pipe(
                                    Box::new(property_expr),
                                    Box::new(index_expr)
                                );
                                
                                return Ok(Expression::Pipe(
                                    Box::new(pipe1),
                                    Box::new(nested_expr)
                                ));
                            }
                        } 
                        // Simple array indexing like .tags[1]
                        else if bracket_end == query.len() {
                            let index_str = &query[bracket_start+1..bracket_end-1];
                            if let Ok(index) = index_str.parse::<i64>() {
                                // Create a pipe expression: .property | .[index]
                                let property_expr = Expression::Property(property.to_string());
                                let index_expr = Expression::Index(index);
                                
                                return Ok(Expression::Pipe(
                                    Box::new(property_expr),
                                    Box::new(index_expr)
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Regular parsing for other queries
    let mut lexer = Lexer::new(query);
    let tokens = lexer.tokenize()?;
    
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lexer_simple_tokens() {
        let mut lexer = Lexer::new(". | .. [] {} , : ?");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens, vec![
            Token::Dot,
            Token::Pipe,
            Token::DotDot,
            Token::LeftBracket,
            Token::RightBracket,
            Token::LeftBrace,
            Token::RightBrace,
            Token::Comma,
            Token::Colon,
            Token::Question,
        ]);
    }
    
    #[test]
    fn test_lexer_literals() {
        let mut lexer = Lexer::new("\"hello\" 42 true false null");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens, vec![
            Token::StringLiteral("hello".to_string()),
            Token::NumberLiteral(42.0),
            Token::BoolLiteral(true),
            Token::BoolLiteral(false),
            Token::Null,
        ]);
    }
    
    #[test]
    fn test_parser_identity() {
        let expr = parse_query(".").unwrap();
        assert!(matches!(expr, Expression::Identity));
    }
    
    #[test]
    fn test_parser_property_access() {
        let expr = parse_query(".name").unwrap();
        match expr {
            Expression::Property(name) => assert_eq!(name, "name"),
            _ => panic!("Expected Property expression"),
        }
    }
    
    #[test]
    fn test_parser_array_index() {
        let expr = parse_query(".[0]").unwrap();
        match expr {
            Expression::Index(idx) => assert_eq!(idx, 0),
            _ => panic!("Expected Index expression"),
        }
    }
    
    #[test]
    fn test_parser_pipe() {
        let expr = parse_query(". | .name").unwrap();
        match expr {
            Expression::Pipe(left, right) => {
                assert!(matches!(*left, Expression::Identity));
                match *right {
                    Expression::Property(name) => assert_eq!(name, "name"),
                    _ => panic!("Expected Property expression on right side of pipe"),
                }
            },
            _ => panic!("Expected Pipe expression"),
        }
    }
}
