use anyhow::{Result, anyhow, Context};
use log::{error, warn, LevelFilter, Log, Metadata, Record, Level, SetLoggerError};
use std::process;
use std::path::{Path, PathBuf};
use std::io::Write;
use std::env;
use std::fs::File;
use std::io::BufReader;

use crate::app_config::{Config, TranslationProvider};
use app_controller::Controller;

mod app_config;
mod translation_service;
mod subtitle_processor;
mod file_utils;
mod app_controller;
mod language_utils;
mod providers;

// @struct: CLI options
struct CommandLineOptions {
    input_path: PathBuf,
    force_overwrite: bool,
    provider: Option<TranslationProvider>,
    model: Option<String>,
    source_language: Option<String>,
    target_language: Option<String>,
    config_path: String,
    log_level: Option<app_config::LogLevel>,
}

// @struct: Custom logger implementation
struct CustomLogger {
    level: LevelFilter,
}

impl CustomLogger {
    // @creates: New logger with specified level
    fn new(level: LevelFilter) -> Self {
        CustomLogger { level }
    }

    // @initializes: Global logger
    fn init(level: LevelFilter) -> Result<(), SetLoggerError> {
        let logger = Box::new(CustomLogger::new(level));
        log::set_boxed_logger(logger)?;
        log::set_max_level(level);
        Ok(())
    }
    
    // @returns: Emoji for log level
    fn get_emoji_for_level(level: Level) -> &'static str {
        match level {
            Level::Error => "❌ ",
            Level::Warn => "🚧 ",
            Level::Info => " ",
            Level::Debug => "🔍 ",
            Level::Trace => "📋 ",
        }
    }
}

impl Log for CustomLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let now = chrono::Local::now().format("%H:%M:%S.%3f");
            
            let mut stderr = std::io::stderr();
            let _ = match record.level() {
                Level::Error => {
                    let emoji = Self::get_emoji_for_level(record.level());
                    writeln!(
                        stderr, 
                        "\x1B[1;31m{} {} {}\x1B[0m", 
                        now, emoji, record.args()
                    )
                },
                Level::Warn => {
                    let emoji = Self::get_emoji_for_level(record.level());
                    writeln!(
                        stderr, 
                        "\x1B[1;33m{} {} {}\x1B[0m", 
                        now, emoji, record.args()
                    )
                },
                Level::Info => {
                    let emoji = Self::get_emoji_for_level(record.level());
                    writeln!(
                        stderr, 
                        "\x1B[1;32m{} {} {}\x1B[0m", 
                        now, emoji, record.args()
                    )
                },
                Level::Debug => {
                    let emoji = Self::get_emoji_for_level(record.level());
                    writeln!(
                        stderr, 
                        "\x1B[1;36m{} {} {}\x1B[0m", 
                        now, emoji, record.args()
                    )
                },
                Level::Trace => {
                    let emoji = Self::get_emoji_for_level(record.level());
                    writeln!(
                        stderr, 
                        "\x1B[1;35m{} {} {}\x1B[0m", 
                        now, emoji, record.args()
                    )
                },
            };
        }
    }

    fn flush(&self) {
        let _ = std::io::stderr().flush();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the logger once with info level by default
    // We'll update the level after loading the config if needed
    CustomLogger::init(LevelFilter::Info)?;
    
    // Parse command line arguments
    let options = parse_command_line_args()?;
    
    // If log level is set via command line, apply it immediately
    if let Some(cmd_log_level) = &options.log_level {
        let log_level = match cmd_log_level {
            app_config::LogLevel::Error => LevelFilter::Error,
            app_config::LogLevel::Warn => LevelFilter::Warn,
            app_config::LogLevel::Info => LevelFilter::Info,
            app_config::LogLevel::Debug => LevelFilter::Debug,
            app_config::LogLevel::Trace => LevelFilter::Trace,
        };
        log::set_max_level(log_level);
    }
    
    // Load or create configuration
    let config_path = &options.config_path;
    let config = if Path::new(config_path).exists() {
        // Load existing configuration
        let file = File::open(config_path)
            .context(format!("Failed to open config file: {}", config_path))?;
        
        let reader = BufReader::new(file);
        let mut config: Config = serde_json::from_reader(reader)
            .context(format!("Failed to parse config file: {}", config_path))?;
        
        // Override config with CLI options if provided
        if let Some(provider) = &options.provider {
            config.translation.provider = provider.clone();
        }
        
        if let Some(model) = &options.model {
            // Find the provider config and update the model
            let provider_str = config.translation.provider.to_string();
            if let Some(provider_config) = config.translation.available_providers.iter_mut()
                .find(|p| p.provider_type == provider_str) {
                provider_config.model = model.clone();
            }
        }
        
        if let Some(source_lang) = &options.source_language {
            config.source_language = source_lang.clone();
        }
        
        if let Some(target_lang) = &options.target_language {
            config.target_language = target_lang.clone();
        }
        
        // Update log level in config if specified via command line
        if let Some(log_level) = &options.log_level {
            config.log_level = log_level.clone();
        }
        
        config
    } else {
        // Create default configuration if not exists
        warn!("Config file not found at '{}', creating default config.", config_path);
        
        let mut config = Config::default_config();
        
        // Apply command line log level to default config if specified
        if let Some(log_level) = &options.log_level {
            config.log_level = log_level.clone();
        }
        
        // Save default config
        let config_json = serde_json::to_string_pretty(&config)
            .context("Failed to serialize default config to JSON")?;
        
        std::fs::write(config_path, config_json)
            .context(format!("Failed to write default config to file: {}", config_path))?;
        
        config
    };
    
    // If log level was not set via command line, update it from config now
    if options.log_level.is_none() {
        let log_level = match config.log_level {
            app_config::LogLevel::Error => LevelFilter::Error,
            app_config::LogLevel::Warn => LevelFilter::Warn,
            app_config::LogLevel::Info => LevelFilter::Info,
            app_config::LogLevel::Debug => LevelFilter::Debug,
            app_config::LogLevel::Trace => LevelFilter::Trace,
        };
        
        // Just update the max level without reinitializing the logger
        log::set_max_level(log_level);
    }
    
    // Create controller
    let controller = Controller::with_config(config.clone())?;
    
    // Run the controller with the input file(s) and output directory
    if options.input_path.is_file() {
        // Process a single file
        controller.run(
            options.input_path.clone(), 
            options.input_path.parent().unwrap_or(Path::new(".")).to_path_buf(),
            options.force_overwrite
        ).await?;
    } else if options.input_path.is_dir() {
        // Process a directory
        controller.run_folder(
            options.input_path.clone(),
            options.force_overwrite
        ).await?;
    } else {
        return Err(anyhow!("Input path does not exist: {:?}", options.input_path));
    }
    
    Ok(())
}

// Parse command line arguments and return options
fn parse_command_line_args() -> Result<CommandLineOptions> {
    let args: Vec<String> = env::args().collect();
    
    // Check for help flag first
    if args.len() > 1 && (args[1] == "-h" || args[1] == "--help") {
        print_usage(&args[0]);
        process::exit(0);
    }
    
    if args.len() < 2 {
        error!("Missing required input path argument");
        print_usage(&args[0]);
        process::exit(1);
    }
    
    let mut options = CommandLineOptions {
        input_path: PathBuf::new(),
        force_overwrite: false,
        provider: None,
        model: None,
        source_language: None,
        target_language: None,
        config_path: "conf.json".to_string(),
        log_level: None,
    };
    
    // Process in two passes:
    // First, check for flags with arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-f" | "--force" => {
                options.force_overwrite = true;
                i += 1;
            },
            "-p" | "--provider" => {
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    let provider_str = &args[i + 1];
                    options.provider = match provider_str.to_lowercase().as_str() {
                        "ollama" => Some(TranslationProvider::Ollama),
                        "openai" => Some(TranslationProvider::OpenAI),
                        "anthropic" => Some(TranslationProvider::Anthropic),
                        _ => {
                            error!("Invalid provider: {}. Valid options are: ollama, openai, anthropic", provider_str);
                            print_usage(&args[0]);
                            process::exit(1);
                        }
                    };
                    i += 2;
                } else {
                    error!("Missing value for provider option");
                    print_usage(&args[0]);
                    process::exit(1);
                }
            },
            "-m" | "--model" => {
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    options.model = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    error!("Missing value for model option");
                    print_usage(&args[0]);
                    process::exit(1);
                }
            },
            "-s" | "--source" => {
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    options.source_language = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    error!("Missing value for source language option");
                    print_usage(&args[0]);
                    process::exit(1);
                }
            },
            "-t" | "--target" => {
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    options.target_language = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    error!("Missing value for target language option");
                    print_usage(&args[0]);
                    process::exit(1);
                }
            },
            "-c" | "--config" => {
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    options.config_path = args[i + 1].clone();
                    i += 2;
                } else {
                    error!("Missing value for config option");
                    print_usage(&args[0]);
                    process::exit(1);
                }
            },
            "-l" | "--log-level" => {
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    let log_level_str = &args[i + 1].to_lowercase();
                    options.log_level = match log_level_str.as_str() {
                        "error" => Some(app_config::LogLevel::Error),
                        "warn" => Some(app_config::LogLevel::Warn),
                        "info" => Some(app_config::LogLevel::Info),
                        "debug" => Some(app_config::LogLevel::Debug),
                        "trace" => Some(app_config::LogLevel::Trace),
                        _ => {
                            error!("Invalid log level: {}. Valid options are: error, warn, info, debug, trace", log_level_str);
                            print_usage(&args[0]);
                            process::exit(1);
                        }
                    };
                    i += 2;
                } else {
                    error!("Missing value for log level option");
                    print_usage(&args[0]);
                    process::exit(1);
                }
            },
            // If it's not an option and we haven't set the input path yet, treat it as the input path
            arg if !arg.starts_with('-') => {
                if options.input_path.as_os_str().is_empty() {
                    options.input_path = PathBuf::from(arg);
                }
                i += 1;
            },
            // Unknown option
            _ => {
                error!("Unknown option: {}", args[i]);
                print_usage(&args[0]);
                process::exit(1);
            }
        }
    }
    
    // Validate that we have an input path
    if options.input_path.as_os_str().is_empty() {
        error!("No input path provided");
        print_usage(&args[0]);
        process::exit(1);
    }
    
    Ok(options)
}

fn print_usage(program_name: &str) {
    println!("Usage: {} [options] <input-path>", program_name);
    println!("Options:");
    println!("  -h, --help              Show this help message");
    println!("  -f, --force             Force overwrite of existing output files");
    println!("  -p, --provider VALUE    Override the translation provider (ollama, openai, anthropic)");
    println!("  -m, --model VALUE       Override the model name");
    println!("  -s, --source VALUE      Override the source language code");
    println!("  -t, --target VALUE      Override the target language code");
    println!("  -c, --config VALUE      Specify config file path (default: conf.json)");
    println!("  -l, --log-level VALUE   Set log level (error, warn, info, debug, trace)");
    println!("");
    println!("Examples:");
    println!("  {} movie.mkv", program_name);
    println!("  {} -f movie.mkv", program_name);
    println!("  {} -p openai -m gpt-4-turbo movie.mkv", program_name);
    println!("  {} -s en -t es movie.mkv", program_name);
    println!("  {} -l debug movie.mkv", program_name);
} 