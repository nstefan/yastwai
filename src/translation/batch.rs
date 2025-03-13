/*!
 * Batch translation processing.
 * 
 * This module contains functionality for processing translations in batches,
 * with support for concurrency, progress tracking, and error handling.
 */

use anyhow::{Result, anyhow};
use log::error;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Semaphore;
use futures::stream::{self, StreamExt};
use std::time::Instant;

use crate::subtitle_processor::SubtitleEntry;

use super::core::{TranslationService, TokenUsageStats, LogEntry};
use super::formatting::FormatPreserver;

/// Batch translator for processing subtitle entries in batches
pub struct BatchTranslator {
    /// The translation service to use
    service: TranslationService,
    
    /// Maximum number of concurrent requests
    max_concurrent_requests: usize,
    
    /// Whether to retry individual entries on batch failure
    retry_individual_entries: bool,
}

impl BatchTranslator {
    /// Create a new batch translator
    pub fn new(service: TranslationService) -> Self {
        Self {
            max_concurrent_requests: service.options.max_concurrent_requests,
            retry_individual_entries: service.options.retry_individual_entries,
            service,
        }
    }
    
    /// Translate batches of subtitle entries
    pub async fn translate_batches(
        &self,
        batches: &[Vec<SubtitleEntry>],
        source_language: &str,
        target_language: &str,
        log_capture: Arc<StdMutex<Vec<LogEntry>>>,
        progress_callback: impl Fn(usize, usize) + Clone + Send + 'static
    ) -> Result<(Vec<SubtitleEntry>, TokenUsageStats)> {
        // Initialize token usage stats
        let token_stats = TokenUsageStats::with_provider_info(
            self.service.config.provider.to_string(),
            self.service.config.get_model()
        );
        
        // Create a semaphore to limit concurrent requests
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent_requests));
        
        // Track progress
        let total_batches = batches.len();
        let processed_batches = Arc::new(AtomicUsize::new(0));
        
        // Process batches concurrently
        let results = stream::iter(batches.iter().enumerate())
            .map(|(batch_index, batch)| {
                let service = self.service.clone();
                let semaphore = semaphore.clone();
                let log_capture = log_capture.clone();
                let processed_batches = processed_batches.clone();
                let progress_callback = progress_callback.clone();
                let source_language = source_language.to_string();
                let target_language = target_language.to_string();
                let retry_individual_entries = self.retry_individual_entries;
                
                async move {
                    // Acquire a permit from the semaphore
                    let _permit = semaphore.acquire().await.unwrap();
                    
                    // Log batch processing start
                    {
                        let mut logs = log_capture.lock().unwrap();
                        logs.push(LogEntry {
                            level: "info".to_string(),
                            message: format!("Processing batch {} of {}", batch_index + 1, total_batches),
                        });
                    }
                    
                    // Process the batch
                    let start_time = Instant::now();
                    let result = service.translate_batch_with_recovery(
                        batch,
                        &source_language,
                        &target_language,
                        log_capture.clone(),
                        retry_individual_entries
                    ).await;
                    
                    // Update progress
                    let current = processed_batches.fetch_add(1, Ordering::SeqCst) + 1;
                    progress_callback(current, total_batches);
                    
                    // Log batch processing completion
                    {
                        let mut logs = log_capture.lock().unwrap();
                        let duration = start_time.elapsed();
                        match &result {
                            Ok(_) => {
                                logs.push(LogEntry {
                                    level: "info".to_string(),
                                    message: format!("Batch {} completed in {:?}", batch_index + 1, duration),
                                });
                            },
                            Err(e) => {
                                logs.push(LogEntry {
                                    level: "error".to_string(),
                                    message: format!("Batch {} failed: {}", batch_index + 1, e),
                                });
                            }
                        }
                    }
                    
                    (batch_index, result)
                }
            })
            .buffer_unordered(self.max_concurrent_requests)
            .collect::<Vec<_>>()
            .await;
        
        // Process results
        let mut all_entries = Vec::new();
        let mut errors = Vec::new();
        
        // Sort results by batch index to maintain original order
        let mut sorted_results = results;
        sorted_results.sort_by_key(|(idx, _)| *idx);
        
        for (batch_idx, result) in sorted_results {
            match result {
                Ok(entries) => {
                    all_entries.extend(entries);
                },
                Err(e) => {
                    errors.push(format!("Batch {} failed: {}", batch_idx + 1, e));
                }
            }
        }
        
        // Check if any batches failed
        if !errors.is_empty() {
            let error_message = format!("Failed to translate all batches: {}", errors.join("; "));
            error!("{}", error_message);
            return Err(anyhow!(error_message));
        }
        
        // Return all translated entries and token stats
        Ok((all_entries, token_stats))
    }
}

impl TranslationService {
    /// Translate a batch of subtitle entries with recovery options
    pub async fn translate_batch_with_recovery(
        &self,
        batch: &[SubtitleEntry],
        source_language: &str,
        target_language: &str,
        log_capture: Arc<StdMutex<Vec<LogEntry>>>,
        retry_individual_entries: bool
    ) -> Result<Vec<SubtitleEntry>> {
        // Skip empty batches
        if batch.is_empty() {
            return Ok(Vec::new());
        }
        
        // Try to translate the entire batch first
        let batch_result = self.translate_batch(batch, source_language, target_language, log_capture.clone()).await;
        
        // If batch translation succeeded or we don't want to retry individual entries, return the result
        if batch_result.is_ok() || !retry_individual_entries {
            return batch_result;
        }
        
        // If batch translation failed, try to translate each entry individually
        {
            let mut logs = log_capture.lock().unwrap();
            logs.push(LogEntry {
                level: "WARN".to_string(),
                message: "Batch translation failed, retrying individual entries".to_string(),
            });
        }
        
        let mut translated_entries = Vec::with_capacity(batch.len());
        let mut errors = Vec::new();
        
        for (idx, entry) in batch.iter().enumerate() {
            let result = self.translate_single_entry(entry, source_language, target_language, log_capture.clone()).await;
            
            match result {
                Ok(translated_entry) => {
                    translated_entries.push(translated_entry);
                },
                Err(e) => {
                    let error_message = format!("Failed to translate entry {}: {}", idx + 1, e);
                    errors.push(error_message.clone());
                    
                    {
                        let mut logs = log_capture.lock().unwrap();
                        logs.push(LogEntry {
                            level: "ERROR".to_string(),
                            message: error_message,
                        });
                    }
                    
                    // Add the original entry as a fallback
                    translated_entries.push(entry.clone());
                }
            }
        }
        
        // Log any errors
        if !errors.is_empty() {
            // Remove direct warning that breaks the progress bar
            // warn!("Some entries failed to translate: {}", errors.join("; "));
            
            // Instead, add this warning to the log capture
            let error_message = format!("Some entries failed to translate: {}", errors.join("; "));
            let mut logs = log_capture.lock().unwrap();
            logs.push(LogEntry {
                level: "WARN".to_string(),
                message: error_message,
            });
        }
        
        Ok(translated_entries)
    }
    
    /// Translate a batch of subtitle entries
    async fn translate_batch(
        &self,
        batch: &[SubtitleEntry],
        source_language: &str,
        target_language: &str,
        log_capture: Arc<StdMutex<Vec<LogEntry>>>
    ) -> Result<Vec<SubtitleEntry>> {
        // Skip empty batches
        if batch.is_empty() {
            return Ok(Vec::new());
        }
        
        // Combine all entries into a single text for translation
        let mut combined_text = String::new();
        let mut entry_indices = Vec::new();
        
        for (idx, entry) in batch.iter().enumerate() {
            // Add a marker before each entry
            combined_text.push_str(&format!("<<ENTRY_{}>>", idx));
            combined_text.push('\n');
            
            // Add the entry text
            combined_text.push_str(&entry.text);
            combined_text.push('\n');
            
            // Store the entry index
            entry_indices.push(idx);
        }
        
        // Add a final marker
        combined_text.push_str("<<END>>");
        
        // Translate the combined text
        let (translated_text, _) = self.translate_text_with_usage(
            &combined_text,
            source_language,
            target_language,
            Some(log_capture.clone())
        ).await?;
        
        // Split the translated text back into entries
        let mut translated_entries = Vec::with_capacity(batch.len());
        let mut current_idx = 0;
        
        for idx in entry_indices {
            let start_marker = format!("<<ENTRY_{}>>", idx);
            let end_marker = if idx == batch.len() - 1 {
                "<<END>>".to_string()
            } else {
                format!("<<ENTRY_{}>>", idx + 1)
            };
            
            // Find the start and end positions
            let start_pos = translated_text[current_idx..].find(&start_marker)
                .map(|pos| pos + current_idx + start_marker.len())
                .ok_or_else(|| anyhow!("Could not find start marker for entry {}", idx))?;
            
            let end_pos = translated_text[start_pos..].find(&end_marker)
                .map(|pos| pos + start_pos)
                .ok_or_else(|| anyhow!("Could not find end marker for entry {}", idx))?;
            
            // Extract the translated text for this entry
            let mut entry_text = translated_text[start_pos..end_pos].trim().to_string();
            
            // Apply format preservation if enabled
            if self.options.preserve_formatting {
                entry_text = FormatPreserver::preserve_formatting(&batch[idx].text, &entry_text);
            }
            
            // Create a new entry with the translated text
            let mut translated_entry = batch[idx].clone();
            translated_entry.text = entry_text;
            
            // Add the translated entry
            translated_entries.push(translated_entry);
            
            // Update the current position
            current_idx = end_pos;
        }
        
        Ok(translated_entries)
    }
    
    /// Translate a single subtitle entry
    async fn translate_single_entry(
        &self,
        entry: &SubtitleEntry,
        source_language: &str,
        target_language: &str,
        log_capture: Arc<StdMutex<Vec<LogEntry>>>
    ) -> Result<SubtitleEntry> {
        // Skip empty entries
        if entry.text.trim().is_empty() {
            return Ok(entry.clone());
        }
        
        // Translate the entry text
        let (translated_text, _) = self.translate_text_with_usage(
            &entry.text,
            source_language,
            target_language,
            Some(log_capture)
        ).await?;
        
        // Apply format preservation if enabled
        let final_text = if self.options.preserve_formatting {
            FormatPreserver::preserve_formatting(&entry.text, &translated_text)
        } else {
            translated_text
        };
        
        // Create a new entry with the translated text
        let mut translated_entry = entry.clone();
        translated_entry.text = final_text;
        
        Ok(translated_entry)
    }
} 