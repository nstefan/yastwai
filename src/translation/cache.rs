/*!
 * Translation caching functionality.
 * 
 * This module provides caching mechanisms for translations to avoid
 * redundant API calls and improve performance.
 */

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use log::debug;

/// Cache key combining source text, source language, and target language
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    /// Source text to translate
    source_text: String,
    
    /// Source language code
    source_language: String,
    
    /// Target language code
    target_language: String,
}

impl CacheKey {
    /// Create a new cache key
    pub fn new(source_text: &str, source_language: &str, target_language: &str) -> Self {
        Self {
            source_text: source_text.to_string(),
            source_language: source_language.to_string(),
            target_language: target_language.to_string(),
        }
    }
}

/// Translation cache for storing and retrieving translations
pub struct TranslationCache {
    /// Internal cache storage
    cache: Arc<RwLock<HashMap<CacheKey, String>>>,
    
    /// Cache hit counter
    hits: Arc<RwLock<usize>>,
    
    /// Cache miss counter
    misses: Arc<RwLock<usize>>,
    
    /// Whether caching is enabled
    enabled: bool,
}

impl TranslationCache {
    /// Create a new translation cache
    pub fn new(enabled: bool) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            hits: Arc::new(RwLock::new(0)),
            misses: Arc::new(RwLock::new(0)),
            enabled,
        }
    }
    
    /// Get a translation from the cache
    pub fn get(&self, source_text: &str, source_language: &str, target_language: &str) -> Option<String> {
        if !self.enabled {
            return None;
        }
        
        let key = CacheKey::new(source_text, source_language, target_language);
        let cache = self.cache.read();
        
        match cache.get(&key) {
            Some(translation) => {
                // Increment hit counter
                let mut hits = self.hits.write();
                *hits += 1;
                
                debug!("Cache hit for '{}' ({} -> {})", 
                       truncate_text(source_text, 30), 
                       source_language, 
                       target_language);
                
                Some(translation.clone())
            },
            None => {
                // Increment miss counter
                let mut misses = self.misses.write();
                *misses += 1;
                
                debug!("Cache miss for '{}' ({} -> {})", 
                       truncate_text(source_text, 30), 
                       source_language, 
                       target_language);
                
                None
            }
        }
    }
    
    /// Store a translation in the cache
    pub fn store(&self, source_text: &str, source_language: &str, target_language: &str, translation: &str) {
        if !self.enabled {
            return;
        }
        
        let key = CacheKey::new(source_text, source_language, target_language);
        let mut cache = self.cache.write();
        
        cache.insert(key, translation.to_string());
        
        debug!("Cached translation for '{}' ({} -> {})", 
               truncate_text(source_text, 30), 
               source_language, 
               target_language);
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> (usize, usize, f64) {
        let hits = *self.hits.read();
        let misses = *self.misses.read();
        let total = hits + misses;
        
        let hit_rate = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };
        
        (hits, misses, hit_rate)
    }
    
    /// Clear the cache
    pub fn clear(&self) {
        let mut cache = self.cache.write();
        cache.clear();
        
        let mut hits = self.hits.write();
        *hits = 0;
        
        let mut misses = self.misses.write();
        *misses = 0;
        
        debug!("Translation cache cleared");
    }
    
    /// Get the number of entries in the cache
    pub fn len(&self) -> usize {
        self.cache.read().len()
    }
    
    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.read().is_empty()
    }
    
    /// Enable or disable the cache
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Check if the cache is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for TranslationCache {
    fn default() -> Self {
        Self::new(true)
    }
}

impl Clone for TranslationCache {
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
            hits: self.hits.clone(),
            misses: self.misses.clone(),
            enabled: self.enabled,
        }
    }
}

/// Truncate text to a maximum length with ellipsis
fn truncate_text(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        text.to_string()
    } else {
        format!("{}...", &text[..max_length])
    }
} 