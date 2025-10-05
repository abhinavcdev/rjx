//! Output module for GQ
//!
//! This module handles formatting and displaying JSON results

use colored::Colorize;
use serde_json::{Value, to_string_pretty, to_string};
use thiserror::Error;

/// Error type for output formatting failures
#[derive(Error, Debug)]
pub enum OutputError {
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Output format options
#[derive(Debug, Clone)]
pub struct OutputOptions {
    /// Pretty print the output with indentation
    pub pretty: bool,
    
    /// Compact output (no whitespace)
    pub compact: bool,
    
    /// Raw output (unwrap strings)
    pub raw: bool,
    
    /// Colorize JSON output
    pub color: bool,
}

impl Default for OutputOptions {
    fn default() -> Self {
        OutputOptions {
            pretty: false,
            compact: false,
            raw: false,
            color: false,
        }
    }
}

/// Formatter for JSON output
pub struct OutputFormatter {
    options: OutputOptions,
}

impl OutputFormatter {
    /// Create a new output formatter with the given options
    pub fn new(options: OutputOptions) -> Self {
        OutputFormatter { options }
    }
    
    /// Format a JSON value as a string
    pub fn format(&self, value: &Value) -> Result<String, OutputError> {
        // Handle raw output (unwrap strings)
        if self.options.raw {
            if let Value::String(s) = value {
                return Ok(s.clone());
            }
        }
        
        // Format the JSON value
        let json_str = if self.options.compact {
            to_string(value)?
        } else if self.options.pretty {
            to_string_pretty(value)?
        } else {
            to_string(value)?
        };
        
        // Colorize the output if requested
        if self.options.color {
            Ok(self.colorize_json(&json_str))
        } else {
            Ok(json_str)
        }
    }
    
    /// Format multiple JSON values as a string
    pub fn format_multiple(&self, values: &[Value]) -> Result<String, OutputError> {
        let mut result = String::new();
        
        for (i, value) in values.iter().enumerate() {
            if i > 0 {
                result.push('\n');
            }
            result.push_str(&self.format(value)?);
        }
        
        Ok(result)
    }
    
    /// Colorize a JSON string
    fn colorize_json(&self, json_str: &str) -> String {
        // Simple colorization for demonstration
        // A more sophisticated implementation would parse the JSON and colorize each element
        
        let mut result = String::new();
        let mut in_string = false;
        let mut escape_next = false;
        
        for c in json_str.chars() {
            match c {
                '"' if !escape_next => {
                    in_string = !in_string;
                    result.push_str(&format!("{}", c.to_string().green()));
                },
                '\\' if in_string => {
                    escape_next = true;
                    result.push(c);
                },
                '{' | '[' if !in_string => {
                    result.push_str(&format!("{}", c.to_string().yellow()));
                },
                '}' | ']' if !in_string => {
                    result.push_str(&format!("{}", c.to_string().yellow()));
                },
                ':' if !in_string => {
                    result.push_str(&format!("{}", c.to_string().cyan()));
                },
                ',' if !in_string => {
                    result.push_str(&format!("{}", c.to_string().cyan()));
                },
                '0'..='9' if !in_string => {
                    result.push_str(&format!("{}", c.to_string().blue()));
                },
                't' | 'f' | 'n' if !in_string && (json_str[json_str.find(c).unwrap()..].starts_with("true") || 
                                                 json_str[json_str.find(c).unwrap()..].starts_with("false") || 
                                                 json_str[json_str.find(c).unwrap()..].starts_with("null")) => {
                    result.push_str(&format!("{}", c.to_string().magenta()));
                },
                _ => {
                    if in_string {
                        result.push_str(&format!("{}", c.to_string().green()));
                    } else {
                        result.push(c);
                    }
                    escape_next = false;
                }
            }
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_format_compact() {
        let options = OutputOptions {
            compact: true,
            ..Default::default()
        };
        let formatter = OutputFormatter::new(options);
        let value = json!({"name": "John", "age": 30});
        
        let result = formatter.format(&value).unwrap();
        assert_eq!(result, r#"{"name":"John","age":30}"#);
    }
    
    #[test]
    fn test_format_pretty() {
        let options = OutputOptions {
            pretty: true,
            ..Default::default()
        };
        let formatter = OutputFormatter::new(options);
        let value = json!({"name": "John", "age": 30});
        
        let result = formatter.format(&value).unwrap();
        assert!(result.contains("{\n"));
        assert!(result.contains("  \"name\""));
    }
    
    #[test]
    fn test_format_raw() {
        let options = OutputOptions {
            raw: true,
            ..Default::default()
        };
        let formatter = OutputFormatter::new(options);
        let value = json!("Hello, world!");
        
        let result = formatter.format(&value).unwrap();
        assert_eq!(result, "Hello, world!");
    }
}
