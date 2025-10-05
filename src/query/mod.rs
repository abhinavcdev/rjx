//! Query module for GQ
//!
//! This module handles the execution of parsed queries against JSON data

use crate::parser::{Expression, ParseError};
use serde_json::{Value, Map};
use thiserror::Error;

/// Error type for query execution failures
#[derive(Error, Debug)]
pub enum QueryError {
    #[error("path error: {0}")]
    Path(String),
    
    #[error("type error: {0}")]
    Type(String),
    
    #[error("index error: {0}")]
    Index(String),
    
    #[error("parse error: {0}")]
    Parse(#[from] ParseError),
    
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Result type for query operations
pub type QueryResult = Result<Vec<Value>, QueryError>;

/// Executes a query expression against JSON data
pub struct QueryEngine;

impl QueryEngine {
    /// Create a new query engine
    pub fn new() -> Self {
        QueryEngine
    }
    
    /// Execute a query expression against JSON data
    pub fn execute(&self, expr: &Expression, data: &Value) -> QueryResult {
        match expr {
            Expression::Identity => {
                // Identity expression (.) just returns the input data
                Ok(vec![data.clone()])
            },
            
            Expression::RecursiveDescent => {
                // Recursive descent (..) returns all nested values
                let mut results = Vec::new();
                self.collect_recursive(data, &mut results);
                Ok(results)
            },
            
            Expression::Property(name) => {
                // Property access (.name or ."name")
                match data {
                    Value::Object(obj) => {
                        if let Some(value) = obj.get(name) {
                            Ok(vec![value.clone()])
                        } else {
                            Ok(vec![Value::Null])
                        }
                    },
                    _ => Err(QueryError::Type(format!("cannot access property '{}' on non-object value", name))),
                }
            },
            
            Expression::Index(index) => {
                // Array index access (.[0])
                match data {
                    Value::Array(arr) => {
                        let idx = if *index < 0 {
                            arr.len().checked_sub(index.unsigned_abs() as usize)
                        } else {
                            Some(*index as usize)
                        };
                        
                        if let Some(idx) = idx {
                            if idx < arr.len() {
                                Ok(vec![arr[idx].clone()])
                            } else {
                                Ok(vec![Value::Null])
                            }
                        } else {
                            Ok(vec![Value::Null])
                        }
                    },
                    _ => Err(QueryError::Type("cannot index non-array value".to_string())),
                }
            },
            
            Expression::Slice(start, end) => {
                // Array slice access (.[1:3])
                match data {
                    Value::Array(arr) => {
                        let start_idx = match start {
                            Some(s) => {
                                if *s < 0 {
                                    arr.len().checked_sub(s.unsigned_abs() as usize).unwrap_or(0)
                                } else {
                                    *s as usize
                                }
                            },
                            None => 0,
                        };
                        
                        let end_idx = match end {
                            Some(e) => {
                                if *e < 0 {
                                    arr.len().checked_sub(e.unsigned_abs() as usize).unwrap_or(arr.len())
                                } else {
                                    (*e as usize).min(arr.len())
                                }
                            },
                            None => arr.len(),
                        };
                        
                        if start_idx <= end_idx && start_idx < arr.len() {
                            let slice = arr[start_idx..end_idx.min(arr.len())].to_vec();
                            Ok(vec![Value::Array(slice)])
                        } else {
                            Ok(vec![Value::Array(vec![])])
                        }
                    },
                    _ => Err(QueryError::Type("cannot slice non-array value".to_string())),
                }
            },
            
            Expression::Array(elements) => {
                // Array constructor ([expr1, expr2, ...])
                let mut result = Vec::new();
                
                for element in elements {
                    let values = self.execute(element, data)?;
                    result.extend(values);
                }
                
                Ok(vec![Value::Array(result)])
            },
            
            Expression::Object(properties) => {
                // Object constructor ({key1: expr1, key2: expr2, ...})
                let mut obj = Map::new();
                
                for (key, expr) in properties {
                    let values = self.execute(expr, data)?;
                    if let Some(value) = values.first() {
                        obj.insert(key.clone(), value.clone());
                    }
                }
                
                Ok(vec![Value::Object(obj)])
            },
            
            Expression::Pipe(left, right) => {
                // Pipe operator (expr1 | expr2)
                let mut results = Vec::new();
                
                // Execute the left expression
                let left_results = self.execute(left, data)?;
                
                // Execute the right expression on each result from the left
                for value in left_results {
                    let right_results = self.execute(right, &value)?;
                    results.extend(right_results);
                }
                
                Ok(results)
            },
            
            Expression::ArrayIteration => {
                // Array iteration (.[]) returns all elements of an array
                match data {
                    Value::Array(arr) => {
                        Ok(arr.clone())
                    },
                    Value::Object(obj) => {
                        // For objects, return all values
                        let values: Vec<Value> = obj.values().cloned().collect();
                        Ok(values)
                    },
                    _ => Err(QueryError::Type("array iteration can only be applied to arrays or objects".to_string())),
                }
            },
            
            Expression::Filter(expr) => {
                // Filter expression
                match data {
                    Value::Array(arr) => {
                        let mut results = Vec::new();
                        
                        for item in arr {
                            let filter_results = self.execute(expr, item)?;
                            
                            // If filter returns any truthy value, include the item
                            if filter_results.iter().any(|v| is_truthy(v)) {
                                results.push(item.clone());
                            }
                        }
                        
                        Ok(vec![Value::Array(results)])
                    },
                    _ => Err(QueryError::Type("filter can only be applied to arrays".to_string())),
                }
            },
            
            Expression::Select(expr, op, value_expr) => {
                // Select expression (select(.field == "value"))
                match data {
                    Value::Array(arr) => {
                        let mut results = Vec::new();
                        
                        for item in arr {
                            let left_results = self.execute(expr, item)?;
                            let right_results = self.execute(value_expr, item)?;
                            
                            if left_results.len() == 1 && right_results.len() == 1 {
                                let left = &left_results[0];
                                let right = &right_results[0];
                                
                                let include = match op.as_str() {
                                    "==" => left == right,
                                    "!=" => left != right,
                                    ">" => compare_values(left, right) == Some(std::cmp::Ordering::Greater),
                                    "<" => compare_values(left, right) == Some(std::cmp::Ordering::Less),
                                    ">=" => {
                                        let cmp = compare_values(left, right);
                                        cmp == Some(std::cmp::Ordering::Greater) || cmp == Some(std::cmp::Ordering::Equal)
                                    },
                                    "<=" => {
                                        let cmp = compare_values(left, right);
                                        cmp == Some(std::cmp::Ordering::Less) || cmp == Some(std::cmp::Ordering::Equal)
                                    },
                                    _ => false,
                                };
                                
                                if include {
                                    results.push(item.clone());
                                }
                            }
                        }
                        
                        Ok(vec![Value::Array(results)])
                    },
                    Value::Object(_) => {
                        let left_results = self.execute(expr, data)?;
                        let right_results = self.execute(value_expr, data)?;
                        
                        if left_results.len() == 1 && right_results.len() == 1 {
                            let left = &left_results[0];
                            let right = &right_results[0];
                            
                            let result = match op.as_str() {
                                "==" => left == right,
                                "!=" => left != right,
                                ">" => compare_values(left, right) == Some(std::cmp::Ordering::Greater),
                                "<" => compare_values(left, right) == Some(std::cmp::Ordering::Less),
                                ">=" => {
                                    let cmp = compare_values(left, right);
                                    cmp == Some(std::cmp::Ordering::Greater) || cmp == Some(std::cmp::Ordering::Equal)
                                },
                                "<=" => {
                                    let cmp = compare_values(left, right);
                                    cmp == Some(std::cmp::Ordering::Less) || cmp == Some(std::cmp::Ordering::Equal)
                                },
                                _ => false,
                            };
                            
                            if result {
                                Ok(vec![data.clone()])
                            } else {
                                Ok(vec![])
                            }
                        } else {
                            Ok(vec![])
                        }
                    },
                    _ => Ok(vec![]),
                }
            },
            
            Expression::Map(expr) => {
                // Map operation (map(expr))
                match data {
                    Value::Array(arr) => {
                        let mut results = Vec::new();
                        
                        for item in arr {
                            let mapped_results = self.execute(expr, item)?;
                            results.extend(mapped_results);
                        }
                        
                        Ok(vec![Value::Array(results)])
                    },
                    _ => Err(QueryError::Type("map can only be applied to arrays".to_string())),
                }
            },
            
            Expression::Keys => {
                // Keys operation (keys)
                match data {
                    Value::Object(obj) => {
                        let keys: Vec<Value> = obj.keys()
                            .map(|k| Value::String(k.clone()))
                            .collect();
                        Ok(vec![Value::Array(keys)])
                    },
                    Value::Array(arr) => {
                        let keys: Vec<Value> = (0..arr.len())
                            .map(|i| Value::Number(serde_json::Number::from(i)))
                            .collect();
                        Ok(vec![Value::Array(keys)])
                    },
                    _ => Err(QueryError::Type("keys can only be applied to objects or arrays".to_string())),
                }
            },
            
            Expression::Length => {
                // Length operation (length)
                match data {
                    Value::Array(arr) => {
                        Ok(vec![Value::Number(serde_json::Number::from(arr.len()))])
                    },
                    Value::Object(obj) => {
                        Ok(vec![Value::Number(serde_json::Number::from(obj.len()))])
                    },
                    Value::String(s) => {
                        Ok(vec![Value::Number(serde_json::Number::from(s.len()))])
                    },
                    _ => Err(QueryError::Type("length can only be applied to arrays, objects, or strings".to_string())),
                }
            },
        }
    }
    
    /// Recursively collect all values in a JSON structure
    fn collect_recursive(&self, value: &Value, results: &mut Vec<Value>) {
        results.push(value.clone());
        
        match value {
            Value::Object(obj) => {
                for (_, v) in obj {
                    self.collect_recursive(v, results);
                }
            },
            Value::Array(arr) => {
                for v in arr {
                    self.collect_recursive(v, results);
                }
            },
            _ => {},
        }
    }
}

/// Check if a JSON value is truthy
fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => !n.is_f64() || n.as_f64().unwrap() != 0.0,
        Value::String(s) => !s.is_empty(),
        Value::Array(arr) => !arr.is_empty(),
        Value::Object(obj) => !obj.is_empty(),
    }
}

/// Compare two JSON values for ordering
fn compare_values(left: &Value, right: &Value) -> Option<std::cmp::Ordering> {
    match (left, right) {
        (Value::Number(l), Value::Number(r)) => {
            if let (Some(lf), Some(rf)) = (l.as_f64(), r.as_f64()) {
                lf.partial_cmp(&rf)
            } else if let (Some(li), Some(ri)) = (l.as_i64(), r.as_i64()) {
                Some(li.cmp(&ri))
            } else if let (Some(lu), Some(ru)) = (l.as_u64(), r.as_u64()) {
                Some(lu.cmp(&ru))
            } else {
                None
            }
        },
        (Value::String(l), Value::String(r)) => Some(l.cmp(r)),
        (Value::Bool(l), Value::Bool(r)) => Some(l.cmp(r)),
        (Value::Array(l), Value::Array(r)) => {
            if l.len() != r.len() {
                return Some(l.len().cmp(&r.len()));
            }
            
            for (lv, rv) in l.iter().zip(r.iter()) {
                if let Some(ord) = compare_values(lv, rv) {
                    if ord != std::cmp::Ordering::Equal {
                        return Some(ord);
                    }
                } else {
                    return None;
                }
            }
            
            Some(std::cmp::Ordering::Equal)
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_identity() {
        let engine = QueryEngine::new();
        let data = json!({"name": "John", "age": 30});
        let expr = Expression::Identity;
        
        let result = engine.execute(&expr, &data).unwrap();
        assert_eq!(result, vec![data]);
    }
    
    #[test]
    fn test_property_access() {
        let engine = QueryEngine::new();
        let data = json!({"name": "John", "age": 30});
        let expr = Expression::Property("name".to_string());
        
        let result = engine.execute(&expr, &data).unwrap();
        assert_eq!(result, vec![json!("John")]);
    }
    
    #[test]
    fn test_array_index() {
        let engine = QueryEngine::new();
        let data = json!([1, 2, 3, 4, 5]);
        let expr = Expression::Index(2);
        
        let result = engine.execute(&expr, &data).unwrap();
        assert_eq!(result, vec![json!(3)]);
    }
    
    #[test]
    fn test_array_slice() {
        let engine = QueryEngine::new();
        let data = json!([1, 2, 3, 4, 5]);
        let expr = Expression::Slice(Some(1), Some(4));
        
        let result = engine.execute(&expr, &data).unwrap();
        assert_eq!(result, vec![json!([2, 3, 4])]);
    }
    
    #[test]
    fn test_pipe() {
        let engine = QueryEngine::new();
        let data = json!({"user": {"name": "John", "age": 30}});
        
        let expr = Expression::Pipe(
            Box::new(Expression::Property("user".to_string())),
            Box::new(Expression::Property("name".to_string()))
        );
        
        let result = engine.execute(&expr, &data).unwrap();
        assert_eq!(result, vec![json!("John")]);
    }
}
