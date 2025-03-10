use std::fs;
use std::env;
use std::path::Path;
use anyhow::Result;
use yastwai::app_config::Config;
use serde_json::Value;
use log::debug;

fn main() -> Result<()> {
    // Get config file path from command line or use default
    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() > 1 {
        args[1].clone()
    } else {
        "conf.example.json".to_string()
    };
    
    debug!("Testing configuration reading from: {}", config_path);
    
    // Read the configuration file as JSON first
    if Path::new(&config_path).exists() {
        let config_content = fs::read_to_string(&config_path)?;
        let json_value: Value = serde_json::from_str(&config_content)?;
        
        debug!("Raw JSON configuration:");
        
        // Print translation provider
        if let Some(translation) = json_value.get("translation") {
            if let Some(provider) = translation.get("provider") {
                debug!("  Provider: {}", provider);
            }
            
            // Print available providers
            if let Some(providers) = translation.get("available_providers") {
                if let Some(providers_array) = providers.as_array() {
                    debug!("  Available providers: {}", providers_array.len());
                    
                    for (i, provider) in providers_array.iter().enumerate() {
                        debug!("  Provider #{}", i + 1);
                        
                        if let Some(provider_type) = provider.get("type") {
                            debug!("    Type: {}", provider_type);
                        }
                        
                        if let Some(model) = provider.get("model") {
                            debug!("    Model: {}", model);
                        }
                        
                        if let Some(endpoint) = provider.get("endpoint") {
                            debug!("    Endpoint: {}", endpoint);
                        }
                    }
                }
            }
        }
        
        // Now parse with the actual Config struct
        debug!("\nParsing with Config struct:");
        let config = Config::from_file(&config_path)?;
        
        debug!("Source language: {}", config.source_language);
        debug!("Target language: {}", config.target_language);
        debug!("Translation provider: {}", config.translation.provider.display_name());
        
        debug!("Available providers: {}", config.translation.available_providers.len());
        for provider in &config.translation.available_providers {
            debug!("  Provider type: {}", provider.provider_type);
            debug!("  Model: {}", provider.model);
            debug!("  Endpoint: {}", provider.endpoint);
        }
        
        // Check helper methods
        debug!("\nHelper methods:");
        debug!("get_model(): {}", config.translation.get_model());
        debug!("get_endpoint(): {}", config.translation.get_endpoint());
        
        debug!("\nConfiguration reading test completed successfully!");
    } else {
        debug!("Configuration file not found: {}", config_path);
    }
    
    Ok(())
} 