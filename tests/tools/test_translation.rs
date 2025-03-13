use std::env;
use std::path::Path;
use anyhow::{Result, Context};
use std::path::PathBuf;
use std::io;

use yastwai::translation::TranslationService;
use yastwai::app_config::{Config, TranslationConfig, TranslationProvider, ProviderConfig, OllamaConfig};
use yastwai::subtitle_processor::SubtitleCollection;
use log::debug;

#[tokio::main]
async fn main() -> Result<()> {
    // Set up logging with simple println
    unsafe {
        std::env::set_var("RUST_LOG", "debug");
    }
    debug!("Starting translation test...");
    
    // Get config file path from command line or use default
    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() > 1 {
        args[1].clone()
    } else if Path::new("conf.json").exists() {
        "conf.json".to_string()
    } else {
        "conf.example.json".to_string()
    };
    
    debug!("Testing translation with config: {}", config_path);
    
    // Load configuration
    let config = Config::from_file(&config_path)?;
    
    // Extract source and target languages
    let source_language = &config.source_language;
    let target_language = &config.target_language;
    
    debug!("Translation setup: {} -> {}", source_language, target_language);
    
    // Create translation service
    let translation_service = TranslationService::new(config.translation.clone())?;
    
    // Test connection
    debug!("Testing connection to the translation service...");
    translation_service.test_connection(source_language, target_language, None).await?;
    debug!("Connection test successful!");
    
    // Test a simple translation
    let text_to_translate = "Hello world! This is a test of the translation service.";
    debug!("Translating: '{}'", text_to_translate);
    
    let translated_text = translation_service.test_translation(source_language, target_language).await?;
    debug!("Translation result: '{}'", translated_text);
    
    debug!("Translation test completed successfully!");
    Ok(())
} 