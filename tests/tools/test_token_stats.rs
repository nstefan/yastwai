use std::time::Duration;
use yastwai::translation::core::TokenUsageStats;

fn main() {
    // Case 1: Stats with both tokens and API duration
    let mut stats1 = TokenUsageStats::new();
    stats1.prompt_tokens = 500;
    stats1.completion_tokens = 300;
    stats1.total_tokens = 800;
    stats1.provider = "OpenAI".to_string();
    stats1.model = "gpt-3.5-turbo".to_string();
    stats1.api_duration = Duration::from_secs(10); // 10 seconds
    
    println!("Case 1 (with API duration): {}", stats1.summary());
    
    // Case 2: Stats with only tokens, no API duration (zero duration)
    let mut stats2 = TokenUsageStats::new();
    stats2.prompt_tokens = 600;
    stats2.completion_tokens = 400;
    stats2.total_tokens = 1000;
    stats2.provider = "Anthropic".to_string();
    stats2.model = "claude-3-haiku".to_string();
    stats2.api_duration = Duration::from_secs(0); // No API duration
    
    println!("Case 2 (without API duration): {}", stats2.summary());
    
    // Case 3: Stats with very short API duration for high tokens/min
    let mut stats3 = TokenUsageStats::new();
    stats3.prompt_tokens = 1000;
    stats3.completion_tokens = 1000;
    stats3.total_tokens = 2000;
    stats3.provider = "Ollama".to_string();
    stats3.model = "llama2".to_string();
    stats3.api_duration = Duration::from_millis(500); // 0.5 seconds
    
    println!("Case 3 (high tokens/min): {}", stats3.summary());

    // Case 4: Stats with very long API duration for low tokens/min
    let mut stats4 = TokenUsageStats::new();
    stats4.prompt_tokens = 100;
    stats4.completion_tokens = 100;
    stats4.total_tokens = 200;
    stats4.provider = "Anthropic".to_string();
    stats4.model = "claude-3-sonnet".to_string();
    stats4.api_duration = Duration::from_secs(60); // 60 seconds
    
    println!("Case 4 (low tokens/min): {}", stats4.summary());
} 