// Module-specific lints configuration
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::too_many_arguments)]
// Add other lints specific to this module that you want to allow but not auto-fix

use anyhow::{Result, anyhow, Context};
use log::{error, warn, info, debug, LevelFilter, Log, Metadata, Record, Level, SetLoggerError};
use std::path::{Path, PathBuf};
use std::io::Write;
use std::fs::File;
use std::io::BufReader;
use clap::{Parser, ValueEnum, CommandFactory, Subcommand};
use clap_complete::{generate, Shell};

use crate::app_config::{Config, TranslationProvider};
use app_controller::Controller;

mod app_config;
mod translation;
mod subtitle_processor;
mod file_utils;
mod app_controller;
mod language_utils;
mod providers;
mod errors;

/// CLI Wrapper for TranslationProvider to implement ValueEnum
#[derive(Debug, Clone, ValueEnum)]
enum CliTranslationProvider {
    Ollama,
    OpenAI,
    Anthropic,
    LMStudio,
}

impl From<CliTranslationProvider> for TranslationProvider {
    fn from(cli_provider: CliTranslationProvider) -> Self {
        match cli_provider {
            CliTranslationProvider::Ollama => TranslationProvider::Ollama,
            CliTranslationProvider::OpenAI => TranslationProvider::OpenAI,
            CliTranslationProvider::Anthropic => TranslationProvider::Anthropic,
            CliTranslationProvider::LMStudio => TranslationProvider::LMStudio,
        }
    }
}

/// CLI Wrapper for LogLevel to implement ValueEnum
#[derive(Debug, Clone, ValueEnum)]
enum CliLogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<CliLogLevel> for app_config::LogLevel {
    fn from(cli_level: CliLogLevel) -> Self {
        match cli_level {
            CliLogLevel::Error => app_config::LogLevel::Error,
            CliLogLevel::Warn => app_config::LogLevel::Warn,
            CliLogLevel::Info => app_config::LogLevel::Info,
            CliLogLevel::Debug => app_config::LogLevel::Debug,
            CliLogLevel::Trace => app_config::LogLevel::Trace,
        }
    }
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Translate video subtitles using AI providers (default command)
    #[command(alias = "translate")]
    Translate(TranslateArgs),
    
    /// Generate shell completions for yastwai
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Parser, Debug)]
struct TranslateArgs {
    /// Input video file or directory to process
    #[arg(value_name = "INPUT_PATH")]
    input_path: PathBuf,

    /// Force overwrite of existing output files
    #[arg(short, long)]
    force_overwrite: bool,

    /// Translation provider to use
    #[arg(short, long, value_enum)]
    provider: Option<CliTranslationProvider>,

    /// Model name to use for translation
    #[arg(short, long)]
    model: Option<String>,

    /// Source language code (e.g., 'en', 'es', 'fr')
    #[arg(short, long)]
    source_language: Option<String>,

    /// Target language code (e.g., 'en', 'es', 'fr')  
    #[arg(short, long)]
    target_language: Option<String>,

    /// Configuration file path
    #[arg(short, long, default_value = "conf.json")]
    config_path: String,

    /// Set logging level
    #[arg(short, long, value_enum)]
    log_level: Option<CliLogLevel>,

    /// Extract subtitle without translation
    #[arg(short, long)]
    extract_only: bool,

    /// Language code for extraction (when using --extract)
    #[arg(long, requires = "extract_only")]
    extract_language: Option<String>,
}

/// YASTwAI - Yet Another Subtitle Translation with AI
/// 
/// A powerful subtitle translation tool that extracts subtitles from video files 
/// and translates them using various AI providers (Ollama, OpenAI, Anthropic).
#[derive(Parser, Debug)]
#[command(name = "yastwai")]
#[command(author = "YASTwAI Team")]
#[command(version = "0.1.0")]
#[command(about = "AI-powered subtitle translation tool")]
#[command(long_about = "YASTwAI extracts subtitles from video files and translates them using AI providers.

EXAMPLES:
    yastwai movie.mkv                           # Translate using default config
    yastwai -f movie.mkv                        # Force overwrite existing files
    yastwai -p openai -m gpt-4 movie.mkv       # Use specific provider and model
    yastwai -s en -t es movie.mkv               # Translate from English to Spanish
    yastwai -e movie.mkv                        # Extract subtitles without translation
    yastwai -e --extract-language en movie.mkv # Extract English subtitles only
    yastwai --log-level debug /movies/         # Process entire directory with debug logging
    yastwai completions bash > yastwai.bash    # Generate bash completions

CONFIGURATION:
    Configuration is stored in conf.json by default. You can specify a different
    config file with --config. If the config file doesn't exist, a default one
    will be created automatically.

SUPPORTED PROVIDERS:
    ollama    - Local Ollama server (default: llama3.2:3b)
    openai    - OpenAI API (requires API key)
    anthropic - Anthropic Claude API (requires API key)
    lmstudio  - LM Studio local server (OpenAI-compatible on http://localhost:1234/v1)")]
struct CommandLineOptions {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Input video file or directory to process
    #[arg(value_name = "INPUT_PATH")]
    input_path: Option<PathBuf>,

    /// Force overwrite of existing output files
    #[arg(short, long)]
    force_overwrite: bool,

    /// Translation provider to use
    #[arg(short, long, value_enum)]
    provider: Option<CliTranslationProvider>,

    /// Model name to use for translation
    #[arg(short, long)]
    model: Option<String>,

    /// Source language code (e.g., 'en', 'es', 'fr')
    #[arg(short, long)]
    source_language: Option<String>,

    /// Target language code (e.g., 'en', 'es', 'fr')  
    #[arg(short, long)]
    target_language: Option<String>,

    /// Configuration file path
    #[arg(short, long, default_value = "conf.json")]
    config_path: String,

    /// Set logging level
    #[arg(short, long, value_enum)]
    log_level: Option<CliLogLevel>,

    /// Extract subtitle without translation
    #[arg(short, long)]
    extract_only: bool,

    /// Language code for extraction (when using --extract)
    #[arg(long, requires = "extract_only")]
    extract_language: Option<String>,
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
    
    // Parse command line arguments using clap
    let cli = CommandLineOptions::parse();
    
    // Handle subcommands
    match cli.command {
        Some(Commands::Completions { shell }) => {
            let mut cmd = CommandLineOptions::command();
            generate(shell, &mut cmd, "yastwai", &mut std::io::stdout());
            return Ok(());
        }
        Some(Commands::Translate(args)) => {
            // Use the explicit translate subcommand args
            return run_translate(args).await;
        }
        None => {
            // Default behavior - use top-level args for backwards compatibility
            let input_path = cli.input_path.ok_or_else(|| {
                anyhow!("INPUT_PATH is required when no subcommand is specified")
            })?;
            
            let translate_args = TranslateArgs {
                input_path,
                force_overwrite: cli.force_overwrite,
                provider: cli.provider,
                model: cli.model,
                source_language: cli.source_language,
                target_language: cli.target_language,
                config_path: cli.config_path,
                log_level: cli.log_level,
                extract_only: cli.extract_only,
                extract_language: cli.extract_language,
            };
            return run_translate(translate_args).await;
        }
    }
}

async fn run_translate(options: TranslateArgs) -> Result<()> {
    // If log level is set via command line, apply it immediately
    if let Some(cmd_log_level) = &options.log_level {
        let config_log_level: app_config::LogLevel = cmd_log_level.clone().into();
        let log_level = match config_log_level {
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
            config.translation.provider = provider.clone().into();
        }
        
        if let Some(model) = &options.model {
            // Find the provider config and update the model
            let provider_str = config.translation.provider.to_lowercase_string();
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
            config.log_level = log_level.clone().into();
        }
        
        config
    } else {
        // Create default configuration if not exists
        warn!("Config file not found at '{}', creating default config.", config_path);
        
        let mut config = Config::default();
        
        // Apply command line log level to default config if specified
        if let Some(log_level) = &options.log_level {
            config.log_level = log_level.clone().into();
        }
        
        // Save default config
        let config_json = serde_json::to_string_pretty(&config)
            .context("Failed to serialize default config to JSON")?;
        
        std::fs::write(config_path, config_json)
            .context(format!("Failed to write default config to file: {}", config_path))?;
        
        config
    };
    
    // Validate the configuration after loading and overriding
    config.validate()
        .context("Configuration validation failed")?;
    
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
    
    // Handle extraction-only mode if enabled
    if options.extract_only {
        // Run the extraction-only mode with the input file(s) and output directory
        if options.input_path.is_file() {
            // Process a single file
            extraction_only_mode(
                &options.input_path, 
                options.input_path.parent().unwrap_or(Path::new(".")).to_path_buf(),
                options.extract_language.as_deref(),
                options.force_overwrite
            ).await?;
        } else if options.input_path.is_dir() {
            // Process a directory
            extraction_only_mode_for_folder(
                &options.input_path,
                options.extract_language.as_deref(),
                options.force_overwrite
            ).await?;
        } else {
            return Err(anyhow!("Input path does not exist: {:?}", options.input_path));
        }
        
        return Ok(());
    }
    
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


// Helper function to implement extraction-only mode
async fn extraction_only_mode(input_file: &Path, output_dir: PathBuf, language_code: Option<&str>, force_overwrite: bool) -> Result<()> {
    use crate::subtitle_processor::SubtitleCollection;
    
    // Check if the input file exists
    if !input_file.exists() {
        return Err(anyhow!("Input file does not exist: {:?}", input_file));
    }
    
    info!("🔍 Extracting subtitles for: {:?}", input_file);
    
    // List available subtitle tracks
    let tracks = SubtitleCollection::list_subtitle_tracks(input_file)
        .await
        .map_err(|e| anyhow!("Failed to list subtitle tracks: {}", e))?;
    
    if tracks.is_empty() {
        warn!("No subtitle tracks found in file: {:?}", input_file);
        return Ok(());
    }
    
    // Display available tracks
    debug!("Found {} subtitle track(s):", tracks.len());
    for track in tracks.iter() {
        debug!("  Track {}: {} ({})", 
             track.index, 
             track.language.as_deref().unwrap_or("unknown"), 
             track.title.as_deref().unwrap_or("No title"));
    }
    
    // Determine which track to extract
    let track_id = if let Some(lang) = language_code {
        // Find track matching the requested language
        let lang = lang.to_lowercase();
        if let Some(matching_track) = tracks.iter().find(|t| 
            t.language.as_ref().is_some_and(|track_lang| language_utils::language_codes_match(track_lang, &lang))) {
            debug!("Selected track {} matching requested language: {}", 
                 matching_track.index, 
                 matching_track.language.as_deref().unwrap_or("unknown"));
            matching_track.index
        } else {
            warn!("No track found matching requested language: {}. Available languages: {}", 
                 lang, 
                 tracks.iter()
                     .filter_map(|t| t.language.clone())
                     .collect::<Vec<_>>()
                     .join(", "));
            return Ok(());
        }
    } else {
        // If no language specified, use the first track
        info!("No language specified, using first track: {} ({})", 
             tracks[0].language.as_deref().unwrap_or("unknown"), 
             tracks[0].title.as_deref().unwrap_or("No title"));
        tracks[0].index
    };
    
    // Create output filename
    let track_info = tracks.iter().find(|t| t.index == track_id)
        .expect("Track should exist");
    
    // Determine the language code to use in the output filename
    let output_lang_code = if let Some(requested_lang) = language_code {
        // Use the user's requested language code format
        requested_lang.to_lowercase()
    } else {
        // If no language specified, use the track's language code
        track_info.language.as_deref().unwrap_or("unknown").to_lowercase()
    };
    
    let output_filename = format!("{}.{}.srt", 
        input_file.file_stem().unwrap().to_string_lossy(),
        output_lang_code);
    
    let output_file = output_dir.join(output_filename);
    
    // Check if output file exists
    if output_file.exists() && !force_overwrite {
        warn!("Output file already exists: {:?}. Use -f to force overwrite.", output_file);
        return Ok(());
    }
    
    // Extract the subtitle
    let _subtitles = SubtitleCollection::extract_from_video(
        input_file, 
        track_id, 
        track_info.language.as_deref().unwrap_or("unknown"),
        &output_file,
    ).await
    .map_err(|e| anyhow!("Failed to extract subtitle: {}", e))?;
    
    info!("Success: {:?}", output_file);
    
    Ok(())
}

// Helper function to process an entire folder in extraction-only mode
async fn extraction_only_mode_for_folder(input_dir: &Path, language_code: Option<&str>, force_overwrite: bool) -> Result<()> {
    use walkdir::WalkDir;
    
    info!("Starting subtitle extraction mode for directory: {:?}", input_dir);
    
    let mut processed_count = 0;
    
    // Find all video files in the directory
    for entry in WalkDir::new(input_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        
        // Skip directories and non-video files
        if path.is_dir() || !is_video_file(path).await {
            continue;
        }
        
        info!("Processing video: {:?}", path);
        
        // Process the file
        if let Err(e) = extraction_only_mode(path, path.parent().unwrap_or(Path::new("")).to_path_buf(), language_code, force_overwrite).await {
            error!("Error processing file: {}", e);
        } else {
            processed_count += 1;
        }
    }
    
    info!("Finished processing {} files", processed_count);
    
    Ok(())
}

// Helper function to check if a file is a video file
async fn is_video_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    
    // Check if it's a video file using FileManager
    if let Ok(file_type) = file_utils::FileManager::detect_file_type(path).await {
        matches!(file_type, file_utils::FileType::Video)
    } else {
        false
    }
} 