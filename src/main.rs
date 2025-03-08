use anyhow::{Result, Context};
use log::{info, error, warn, LevelFilter, Log, Metadata, Record, Level, SetLoggerError};
use std::process;
use std::path::{Path, PathBuf};
use std::env;
use std::io::Write;
use std::fs;

mod app_config;
mod translation_service;
mod subtitle_processor;
mod file_utils;
mod app_controller;
mod language_utils;

use app_config::{Config, LogLevel};
use app_controller::Controller;
use crate::file_utils::FileManager;

/// A simple custom logger implementation
struct CustomLogger {
    level: LevelFilter,
}

impl CustomLogger {
    fn new(level: LevelFilter) -> Self {
        CustomLogger { level }
    }

    fn init(level: LevelFilter) -> Result<(), SetLoggerError> {
        let logger = Box::new(CustomLogger::new(level));
        log::set_boxed_logger(logger)?;
        log::set_max_level(level);
        Ok(())
    }
    
    fn get_emoji_for_level(level: Level) -> &'static str {
        match level {
            Level::Error => "❌ ",
            Level::Warn => "🚧 ",
            Level::Info => "ℹ️ ",
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
    // Load configuration first, to properly set up the logger with the right level
    let config_path = "conf.json";
    let example_config_path = "conf.example.json";
    
    // Load or create configuration
    let config = if FileManager::file_exists(config_path) {
        println!("Loading configuration from {}", config_path);
        Config::from_file(config_path).with_context(|| "Failed to load configuration")?
    } else {
        if FileManager::file_exists(example_config_path) {
            println!("Configuration file not found, but example configuration exists.");
            println!("You can copy it using: cp {} {}", example_config_path, config_path);
            println!("Creating default configuration at {}", config_path);
        } else {
            println!("Configuration file not found, creating default at {}", config_path);
        }
        app_config::create_default_config_file(config_path)?
    };
    
    // Convert LogLevel enum to LevelFilter
    let log_level = match config.log_level {
        LogLevel::Error => LevelFilter::Error,
        LogLevel::Warn => LevelFilter::Warn,
        LogLevel::Info => LevelFilter::Info,
        LogLevel::Debug => LevelFilter::Debug,
        LogLevel::Trace => LevelFilter::Trace,
    };
    
    // Parse CLI arguments
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        error!(" Missing required input path argument");
        print_usage(&args[0]);
        process::exit(1);
    }
    
    if args.len() > 3 {
        warn!("⚠️ Too many arguments provided. Only the first 1-2 arguments will be used.");
    }
    
    // Initialize logging with the appropriate level
    if let Err(e) = CustomLogger::init(log_level) {
        error!(" Init failed: {}", e);
        process::exit(1);
    }
    
    // Start the application with a welcome message
    info!("YASTwAI started");
    info!("Log level: {}", log_level);
    
    // Get input path from arguments
    let input_path_str = &args[1];
    
    // Sanitize the input path - basic security check
    if input_path_str.contains("..") || input_path_str.contains("|") || input_path_str.contains(";") {
        error!(" Input path contains potentially unsafe characters");
        process::exit(1);
    }
    
    // Canonicalize the input path to resolve symlinks and relative paths
    let input_path = Path::new(input_path_str);
    let canonicalized_input_path = match fs::canonicalize(input_path) {
        Ok(path) => path,
        Err(e) => {
            error!(" Invalid input path: {} - {}", input_path_str, e);
            process::exit(1);
        }
    };
    
    // Check if the input path exists
    if !canonicalized_input_path.exists() {
        error!(" Input path not found: {:?}", canonicalized_input_path);
        process::exit(1);
    }
    
    // Determine output directory
    let output_dir = if args.len() >= 3 {
        let output_path_str = &args[2];
        
        // Sanitize the output path - basic security check
        if output_path_str.contains("..") || output_path_str.contains("|") || output_path_str.contains(";") {
            error!(" Output path contains potentially unsafe characters");
            process::exit(1);
        }
        
        let dir = PathBuf::from(output_path_str);
        
        // Create the output directory if it doesn't exist
        if !dir.exists() {
            match fs::create_dir_all(&dir) {
                Ok(_) => info!("Created output directory: {:?}", dir),
                Err(e) => {
                    error!(" Failed to create output directory: {}", e);
                    process::exit(1);
                }
            }
        } else if !dir.is_dir() {
            error!(" Specified output path exists but is not a directory: {:?}", dir);
            process::exit(1);
        }
        
        // Return the output directory
        match fs::canonicalize(&dir) {
            Ok(path) => path,
            Err(e) => {
                error!(" Invalid output path: {} - {}", output_path_str, e);
                process::exit(1);
            }
        }
    } else {
        // If no output directory is provided, use the parent directory of the input file
        // or the current directory for folder inputs
        if canonicalized_input_path.is_file() {
            match canonicalized_input_path.parent() {
                Some(parent) => parent.to_path_buf(),
                None => {
                    error!(" Cannot determine parent directory for input file");
                    process::exit(1);
                }
            }
        } else {
            canonicalized_input_path.clone()
        }
    };
    
    // Create and initialize the controller
    let controller = match app_controller::Controller::with_config(config) {
        Ok(c) => c,
        Err(e) => {
            error!(" Failed to initialize controller: {}", e);
            process::exit(1);
        }
    };
    
    // Automatically determine mode based on whether the input is a file or directory
    if canonicalized_input_path.is_dir() {
        info!("Processing directory: {:?}", canonicalized_input_path);
        
        // Run the controller in folder mode
        if let Err(e) = controller.run_folder(canonicalized_input_path, output_dir).await {
            error!(" Error: {}", e);
            process::exit(1);
        }
    } else if canonicalized_input_path.is_file() {
        let file_name = canonicalized_input_path.file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| String::from("Unknown file"));
        info!("Processing file: {}", file_name);
        
        // Run the controller in file mode
        if let Err(e) = controller.run(canonicalized_input_path, output_dir).await {
            error!(" Error: {}", e);
            process::exit(1);
        }
    } else {
        error!(" Input path exists but is neither a file nor a directory: {:?}", canonicalized_input_path);
        process::exit(1);
    }
    
    Ok(())
}

/// Print usage instructions for the application
fn print_usage(program_name: &str) {
    error!(" Usage:");
    error!("   {} <input_path> [output_directory]", program_name);
    error!("");
    error!("Examples:");
    error!("   {} video.mkv", program_name);
    error!("   {} video.mkv /path/to/output", program_name);
    error!("   {} /path/to/videos", program_name);
    error!("   {} /path/to/videos /path/to/output", program_name);
    error!("");
    error!("Description:");
    error!("   - If input is a file, translates subtitles from that video");
    error!("   - If input is a folder, processes all videos in the folder");
} 