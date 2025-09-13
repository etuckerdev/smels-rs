use clap::{Parser, Subcommand};
use smels::check::{output, CodeChecker};
use smels::parsers::{
    java::JavaParser, js::JsParser, python::PythonParser, rust::RustParser, GenericParser,
};
use smels::rules::common::CommonRule;
use smels::templates::respond;
use smels::Analyzer;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Clone, clap::ValueEnum)]
enum OutputFormat {
    Default,
    Json,
}

#[derive(Parser)]
#[command(name = "smels")]
#[command(version = "0.2.0")]
#[command(about = "Small Model Error Log Summarizer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze error logs from stdin or file
    Analyze {
        /// Input file to analyze (reads from stdin if not provided)
        #[arg(short, long, value_name = "FILE")]
        file: Option<PathBuf>,

        /// Programming language (auto-detected if not provided)
        #[arg(short, long)]
        lang: Option<String>,

        /// Enable small model analysis
        #[arg(long)]
        ai: bool,

        /// Disable small model analysis
        #[arg(long)]
        no_ai: bool,

        /// Enable AI debug output
        #[arg(long)]
        ai_debug: bool,

        /// Output results in JSON format
        #[arg(long)]
        json: bool,

        /// Start web server to view results
        #[arg(long)]
        web: bool,
    },
    /// Check code quality and detect potential issues
    Check {
        /// Path to check (defaults to current directory)
        #[arg(short, long, value_name = "PATH")]
        path: Option<PathBuf>,

        /// Strict mode - fail on any issues
        #[arg(long)]
        strict: bool,

        /// Output format
        #[arg(short, long, value_enum, default_value = "default")]
        format: OutputFormat,

        /// Check specific file types only (e.g., "rs,js,py")
        #[arg(long, value_name = "TYPES")]
        filter: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze {
            file,
            lang,
            ai,
            no_ai,
            ai_debug,
            json,
            web,
        } => {
            if ai_debug {
                std::env::set_var("SMELS_AI_DEBUG", "1");
            }

            let use_ai = if no_ai { false } else { ai };

            let input = if web {
                // For web mode, we don't need input from stdin/file
                String::new()
            } else {
                // Get input from file or stdin for analysis
                if let Some(filename) = file {
                    std::fs::read_to_string(&filename).unwrap_or_else(|_| {
                        eprintln!("Failed to read file: {}", filename.display());
                        std::process::exit(1);
                    })
                } else {
                    // Try to read from stdin
                    let mut input = String::new();
                    let mut stdin = io::stdin();
                    let mut buffer = [0; 1];

                    // Check if stdin has data available
                    match stdin.read(&mut buffer) {
                        Ok(0) => {
                            // No data available
                            eprintln!(
                                "No input provided. Use 'smels analyze < input' or 'smels analyze -f file'"
                            );
                            eprintln!("Run 'smels --help' for more information");
                            std::process::exit(1);
                        }
                        Ok(_) => {
                            // Data available, read the rest
                            input.push(buffer[0] as char);
                            if stdin.read_to_string(&mut input).is_err() {
                                eprintln!("Failed to read from stdin");
                                std::process::exit(1);
                            }
                            input
                        }
                        Err(_) => {
                            eprintln!("Failed to read from stdin");
                            std::process::exit(1);
                        }
                    }
                }
            };

            if web {
                // Start web server using the new web module
                println!("Starting web server at http://localhost:3001");
                println!("Press Ctrl+C to stop");

                #[cfg(feature = "web")]
                {
                    if let Err(e) = smels::web::start_web_server(3001).await {
                        eprintln!("Failed to start web server: {}", e);
                        std::process::exit(1);
                    }
                }

                #[cfg(not(feature = "web"))]
                {
                    eprintln!(
                        "Web feature not enabled. Compile with --features web to enable web interface."
                    );
                    std::process::exit(1);
                }
            } else {
                let mut analyzer = Analyzer::new().with_ai(use_ai);
                analyzer.add_parser(Box::new(GenericParser));
                if lang.as_deref() != Some("none") {
                    analyzer.add_parser(Box::new(RustParser));
                    analyzer.add_parser(Box::new(JsParser));
                    analyzer.add_parser(Box::new(PythonParser));
                    analyzer.add_parser(Box::new(JavaParser));
                }
                analyzer.add_rule(Box::new(CommonRule));

                let result = analyzer.analyze(&input).await;

                if json {
                    println!("{}", respond::format_result_json(&result));
                } else {
                    println!(
                        "{}",
                        respond::format_result(&result, respond::OutputFormat::Default)
                    );
                }
            }
        }
        Commands::Check {
            path,
            strict,
            format,
            filter: _filter,
        } => {
            let checker = CodeChecker::new();
            let check_path = path.unwrap_or_else(|| PathBuf::from("."));

            match checker.check_path(&check_path, _filter.as_deref()).await {
                Ok(result) => {
                    match format {
                        OutputFormat::Json => {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&result).unwrap_or_else(|_| {
                                    r#"{"error":"Failed to serialize result"}"#.to_string()
                                })
                            );
                        }
                        OutputFormat::Default => {
                            output::print_check_result(&result, strict);
                        }
                    }

                    // Exit with error code if issues found and strict mode
                    if strict && !result.passed {
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Error during check: {e:?}");
                    std::process::exit(1);
                }
            }
        }
    }
}
