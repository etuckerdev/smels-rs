use smels::parsers::{
    java::JavaParser, js::JsParser, python::PythonParser, rust::RustParser, GenericParser,
};
use smels::rules::common::CommonRule;
use smels::templates::respond;
use smels::Analyzer;
use std::io::{self, Read};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: smels analyze [--ai|--no-ai] [--ai-debug] [--json] [--web] [-f file] [--lang language] or smels analyze < input");
        eprintln!("       smels --help");
        return;
    }

    let command = &args[1];
    if command == "--help" || command == "-h" {
        println!("SMELS - Smart Error Log Extraction and Summarization");
        println!("");
        println!("Usage:");
        println!("  smels analyze [--ai|--no-ai] [--ai-debug] [--json] [--web] [-f file] [--lang language]");
        println!("  smels analyze < input");
        println!("");
        println!("Commands:");
        println!("  analyze    Analyze error logs from stdin or file");
        println!("");
        println!("Options:");
        println!("  --ai       Enable AI-powered analysis (default: disabled)");
        println!("  --no-ai    Disable AI-powered analysis");
        println!("  --ai-debug Enable AI debug output");
        println!("  --json     Output results in JSON format");
        println!("  --web      Start web server to view results");
        println!("  -f file    Read input from file instead of stdin");
        println!("  --lang     Specify programming language (auto-detected if not provided)");
        println!("  --help     Show this help message");
        return;
    }

    if command != "analyze" {
        eprintln!("Unknown command: {}", command);
        eprintln!("Run 'smels --help' for usage information");
        return;
    }

    let mut lang = None;
    let mut file = None;
    let mut use_ai = false;
    let mut json_output = false;
    let mut i = 2;
    let mut ai_debug = false;
    let mut web_mode = false;
    while i < args.len() {
        match args[i].as_str() {
            "--lang" => {
                if i + 1 < args.len() {
                    lang = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Missing value for --lang");
                    return;
                }
            }
            "-f" => {
                if i + 1 < args.len() {
                    file = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Missing file after -f");
                    return;
                }
            }
            "--ai" => {
                use_ai = true;
                i += 1;
            }
            "--no-ai" => {
                use_ai = false;
                i += 1;
            }
            "--json" => {
                json_output = true;
                i += 1;
            }
            "--ai-debug" => {
                ai_debug = true;
                i += 1;
            }
            "--web" => {
                web_mode = true;
                i += 1;
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                return;
            }
        }
    }

    if ai_debug {
        unsafe {
            std::env::set_var("SMELS_AI_DEBUG", "1");
        }
    }

    let input = if web_mode {
        // For web mode, we don't need input from stdin/file
        String::new()
    } else {
        // Get input from file or stdin for analysis
        if let Some(filename) = file {
            std::fs::read_to_string(&filename).unwrap_or_else(|_| {
                eprintln!("Failed to read file: {}", filename);
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
                    if let Err(_) = stdin.read_to_string(&mut input) {
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

    if web_mode {
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

        if json_output {
            println!("{}", respond::format_result_json(&result));
        } else {
            println!(
                "{}",
                respond::format_result(&result, respond::OutputFormat::Default)
            );
        }
    }
}
