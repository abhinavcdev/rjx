mod parser;
mod query;
mod output;

use anyhow::{Result, Context};
use clap::Parser;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::PathBuf;
use std::time::Instant;

use parser::parse_query;
use query::QueryEngine;
use output::{OutputFormatter, OutputOptions};
use serde_json::Value;

/// RJQ - A fast and lightweight JSON processor in Rust (jq alternative)
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    /// The query to run on the JSON input
    #[clap(short, long, value_parser)]
    query: String,

    /// Input file (reads from stdin if not provided)
    #[clap(value_parser)]
    input: Option<PathBuf>,

    /// Pretty print the output
    #[clap(short, long, action)]
    pretty: bool,

    /// Compact output (no whitespace)
    #[clap(short, long, action)]
    compact: bool,

    /// Raw output (unwrap strings)
    #[clap(short, long, action)]
    raw: bool,

    /// Colorize JSON output
    #[clap(short = 'C', long, action)]
    color: bool,
    
    /// Benchmark mode - show execution time
    #[clap(short, long, action)]
    benchmark: bool,
    
    /// Debug mode (show detailed error information)
    #[clap(long, action)]
    debug: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Read input from file or stdin
    let json_input = match cli.input {
        Some(path) => {
            let file = File::open(&path)
                .with_context(|| format!("Failed to open file: {}", path.display()))?;
            let mut reader = BufReader::new(file);
            let mut contents = String::new();
            reader.read_to_string(&mut contents)
                .with_context(|| format!("Failed to read file: {}", path.display()))?;
            contents
        }
        None => {
            let mut contents = String::new();
            io::stdin().read_to_string(&mut contents)
                .context("Failed to read from stdin")?;
            contents
        }
    };

    // Parse the JSON input
    let start_parse = Instant::now();
    let json_value: Value = serde_json::from_str(&json_input)
        .context("Failed to parse JSON input")?;
    let parse_duration = start_parse.elapsed();
    
    // Parse the query
    let start_query_parse = Instant::now();
    let query_expr = parse_query(&cli.query)
        .context("Failed to parse query")?;
    let query_parse_duration = start_query_parse.elapsed();
    
    // Execute the query
    let start_execute = Instant::now();
    let query_engine = QueryEngine::new();
    
    // Debug the query expression
    if cli.debug {
        eprintln!("Query expression: {:?}", query_expr);
    }
    
    let results = match query_engine.execute(&query_expr, &json_value) {
        Ok(results) => results,
        Err(e) => {
            eprintln!("Error executing query: {}", e);
            if cli.debug {
                eprintln!("Expression: {:?}", query_expr);
                eprintln!("Data: {}", serde_json::to_string_pretty(&json_value).unwrap_or_default());
            }
            return Err(e.into());
        }
    };
    let execute_duration = start_execute.elapsed();
    
    // Format and output the results
    let start_output = Instant::now();
    let output_options = OutputOptions {
        pretty: cli.pretty,
        compact: cli.compact,
        raw: cli.raw,
        color: cli.color,
    };
    
    let formatter = OutputFormatter::new(output_options);
    let output = formatter.format_multiple(&results)
        .context("Failed to format output")?;
    let output_duration = start_output.elapsed();
    
    // Print the results
    println!("{}", output);
    
    // Print benchmark information if requested
    if cli.benchmark {
        eprintln!("\nBenchmark:");
        eprintln!("  JSON parse time:   {:?}", parse_duration);
        eprintln!("  Query parse time:  {:?}", query_parse_duration);
        eprintln!("  Execution time:    {:?}", execute_duration);
        eprintln!("  Formatting time:   {:?}", output_duration);
        eprintln!("  Total time:        {:?}", 
            parse_duration + query_parse_duration + execute_duration + output_duration);
    }

    Ok(())
}
