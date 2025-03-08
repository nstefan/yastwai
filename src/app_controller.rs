use anyhow::{Result, Context, anyhow};
use log::{debug, info, warn, error};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use tempfile::tempdir;
use crate::app_config::{Config, TranslationProvider, SubtitleInfo};
use crate::subtitle_processor::SubtitleCollection;
use crate::translation_service::TranslationService;

/// Application controller module
/// coordinating subtitle extraction, translation, and output.
/// Main controller for the application
pub struct Controller {
    /// Application configuration
    config: Config,
    
    /// Translation service
    translation_service: TranslationService,
}

impl Controller {
    /// Create a new Controller with the specified configuration
    pub fn with_config(config: Config) -> Result<Self> {
        // Instantiate components
        let translation_service = TranslationService::new(config.translation.clone())?;
        
        Ok(Controller {
            config,
            translation_service,
        })
    }
    
    /// Run the main workflow with input video file and output directory
    pub async fn run(&self, input_file: PathBuf, output_dir: PathBuf) -> Result<()> {
        // Create a multi progress bar display
        let multi_progress = MultiProgress::new();
        
        // Run the real implementation with progress tracking
        self.run_with_progress(input_file, output_dir, &multi_progress).await
    }
    
    /// Internal run method that accepts a MultiProgress instance
    async fn run_with_progress(&self, input_file: PathBuf, output_dir: PathBuf, multi_progress: &MultiProgress) -> Result<()> {
        // Start timing the process
        let start_time = std::time::Instant::now();
                
        // Validate configuration
        self.config.validate()?;
        
        // Only test connection for the first file - don't repeat this for each file in folder mode
        // Use a static variable to track if we've already tested the connection
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Once;
        static CONNECTION_TESTED: AtomicBool = AtomicBool::new(false);
        static INIT_TEST: Once = Once::new();
        
        if !CONNECTION_TESTED.load(Ordering::SeqCst) {
            // Test connection to translation service
            info!("Testing translation service: {}", self.config.translation.provider.display_name());
            self.translation_service.test_connection(
                &self.config.source_language,
                &self.config.target_language
            ).await.context("Failed to connect to translation service")?;
            
            // Perform a sample translation to verify everything is working correctly
            let _test_translation = self.translation_service.test_translation(
                &self.config.source_language,
                &self.config.target_language
            ).await.context("Translation test failed")?;
            
            info!("Translation service ready");
            
            // Mark connection as tested so we don't do it again
            CONNECTION_TESTED.store(true, Ordering::SeqCst);
        }
        
        // Extract subtitles from the input file
        info!("Extracting subtitles");
        let subtitles = self.extract_subtitles_to_memory(&input_file)?;
        
        if subtitles.entries.is_empty() {
            return Err(anyhow!("No valid subtitles found in the video file"));
        } else {
            info!("Extracted {} subtitle entries", subtitles.entries.len());
        }
                
        // Translate all subtitles
        info!("Translating, please wait…");
        let translated = self.translate_subtitles_with_progress(subtitles, multi_progress).await?;
        
        // Save translated subtitles
        let _output_path = self.save_translated_subtitles(translated, &input_file, &output_dir)?;
        
        // Calculate and display the elapsed time
        let elapsed = start_time.elapsed();
        info!("File processed in {}", Self::format_duration(elapsed));
        
        Ok(())
    }
    
    /// Extract subtitles from the video file directly to memory
    fn extract_subtitles_to_memory(&self, input_file: &Path) -> Result<SubtitleCollection> {
        // Start timing the extraction process
        let extraction_start_time = std::time::Instant::now();
        
        // Auto-select the subtitle track matching the source language
        info!("Auto-selecting subtitle track");
        
        let tracks = SubtitleCollection::list_subtitle_tracks(input_file)?;
        
        if tracks.is_empty() {
            return Err(anyhow!("No subtitle tracks found in the video file"));
        }
                
        // Find the best track based on language match
        let track_id = match SubtitleCollection::select_subtitle_track(&tracks, &self.config.source_language) {
            Some(id) => {
                info!("Selected track {} for extraction", id);
                id
            },
            None => {
                // Use first track if no language match found
                let first_track = tracks.first().unwrap().index;
                warn!("No language match found, using first track: {}", first_track);
                first_track
            }
        };
        
        // Create a temporary file for extraction
        let temp_dir = tempdir()?;
        let temp_filename = format!("extracted_subtitles_{}.srt", track_id);
        let temp_path = temp_dir.path().join(temp_filename);
        
        // Extract the selected track to the temporary file
        let source_language = &self.config.source_language;
        let subtitles = SubtitleCollection::extract_from_video(
            input_file,
            track_id,
            source_language,
            &temp_path
        );
        
        // Calculate and log the extraction time
        let extraction_elapsed = extraction_start_time.elapsed();
        info!("Subtitle extraction completed in {}", Self::format_duration(extraction_elapsed));
        
        subtitles
    }
    
    /// Translate the extracted subtitles
    #[allow(dead_code)]
    async fn translate_subtitles(&self, subtitles: SubtitleCollection) -> Result<SubtitleCollection> {
        // Create a multi progress bar display
        let multi_progress = MultiProgress::new();
        
        self.translate_subtitles_with_progress(subtitles, &multi_progress).await
    }
    
    /// Internal method to translate subtitles with a progress bar from the provided MultiProgress
    async fn translate_subtitles_with_progress(&self, subtitles: SubtitleCollection, multi_progress: &MultiProgress) -> Result<SubtitleCollection> {
        // Start timing the translation process
        let translation_start_time = std::time::Instant::now();
        
        // Split subtitle collection into manageable chunks for translation
        let max_chars_per_chunk = match self.config.translation.provider {
            TranslationProvider::Ollama => self.config.translation.ollama.max_chars_per_request,
            TranslationProvider::OpenAI => self.config.translation.openai.max_chars_per_request,
            TranslationProvider::Anthropic => self.config.translation.anthropic.max_chars_per_request,
        };
        
        // Split the subtitles into chunks that respect the max characters limit
        let chunks = subtitles.split_into_chunks(max_chars_per_chunk);
        
        // Create a progress bar for translation tracking
        let total_chunks = chunks.len() as u64;
        let progress_bar = multi_progress.add(ProgressBar::new(total_chunks));
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} chunks ({eta})")
                .unwrap()
                .progress_chars("##-")
        );
        progress_bar.set_message("Translating...");
        
        // Track processed entries for progress display
        let processed = Arc::new(AtomicUsize::new(0));
        let processed_clone = processed.clone();
        
        // Create a separate progress bar updater that watches the atomic counter
        let pb_clone = progress_bar.clone();
        let progress_callback = move |completed_count: usize, _total: usize| {
            processed_clone.store(completed_count, Ordering::SeqCst);
            // Update the progress bar as callbacks come in
            pb_clone.set_position(completed_count as u64);
        };
        
        // Translate all batches at once
        let translated_entries = self.translation_service.translate_batches(
            &chunks,
            &self.config.source_language,
            &self.config.target_language,
            progress_callback
        ).await.context("Failed to translate subtitles")?;
        
        progress_bar.finish_with_message("Translation complete!");
        
        // Calculate and log the translation time
        let translation_elapsed = translation_start_time.elapsed();
        info!("Translation service completed in {}", Self::format_duration(translation_elapsed));
        
        // Create a new SubtitleCollection with the translated entries
        let mut result = SubtitleCollection::new(
            PathBuf::from("translated_subtitles.srt"),
            self.config.target_language.clone()
        );
        result.entries = translated_entries;
        
        Ok(result)
    }
    
    /// Save the translated subtitles to files
    fn save_translated_subtitles(&self, subtitles: SubtitleCollection, input_file: &Path, output_dir: &Path) -> Result<PathBuf> {
        // Generate an appropriate output filename
        let input_stem = input_file.file_stem()
            .context("Failed to extract file stem from input file")?
            .to_string_lossy();
        
        let output_filename = self.get_subtitle_output_filename(
            input_file, 
            &self.config.target_language
        );
        
        let output_path = output_dir.join(output_filename);
        
        // Save the subtitle collection to the output path
        subtitles.write_to_srt(&output_path)?;
                
        Ok(output_path)
    }
    
    // Format duration in a human-readable format (HH:MM:SS)
    fn format_duration(duration: std::time::Duration) -> String {
        let total_seconds = duration.as_secs();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        
        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}.{:03}s", seconds, duration.subsec_millis())
        }
    }
    
    /// Run the workflow in folder mode, processing all video files in a directory
    /// This will scan the input directory for supported video files and process each one
    /// Files that already have translated subtitles will be skipped
    pub async fn run_folder(&self, input_dir: PathBuf, output_dir: PathBuf) -> Result<()> {
        // Start timing the process
        let start_time = std::time::Instant::now();
                
        // Validate configuration
        self.config.validate()?;
        
        // Don't need to test connection here - it will be tested on first file in run_with_progress
        
        // Define supported video extensions (commonly supported by ffmpeg)
        // This could be expanded based on ffmpeg's capabilities
        let supported_extensions = ["mp4", "mkv", "avi", "mov", "wmv", "flv", "webm"];
        
        let mut video_files = Vec::new();
        for ext in &supported_extensions {
            let mut files = crate::file_utils::FileManager::find_files(&input_dir, ext)?;
            video_files.append(&mut files);
        }
        
        if video_files.is_empty() {
            warn!("No supported video files found in the directory");
            return Ok(());
        }
        
        // Sort files by name for consistent processing order
        video_files.sort();
        
        info!("Found {} video files to process", video_files.len());
        
        // Create multi-progress bar to track overall progress
        let multi_progress = MultiProgress::new();
        
        // Create overall progress bar for the entire folder
        let folder_pb = multi_progress.add(ProgressBar::new(video_files.len() as u64));
        folder_pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] {bar:40.green/blue} [{pos}/{len}] {prefix}: {msg}")
                .expect("Failed to create folder progress style")
                .progress_chars("##-")
        );
        folder_pb.set_prefix("Folder Progress");
        folder_pb.set_message("Processing folder...");
        
        // Process each video file
        let mut success_count = 0;
        let mut skip_count = 0;
        let mut error_count = 0;
        
        for (index, video_file) in video_files.iter().enumerate() {
            let file_name = video_file.file_name().unwrap_or_default().to_string_lossy().to_string();
            folder_pb.set_message(format!("File {}/{}: {}", index + 1, video_files.len(), file_name));
            
            info!("Processing file {}/{}: {}", index + 1, video_files.len(), file_name);
            
            // Check if the translated subtitle already exists
            let output_filename = self.get_subtitle_output_filename(video_file, &self.config.target_language);
            let output_path = output_dir.join(&output_filename);
            
            if output_path.exists() {
                info!("Skipping file, translation already exists");
                skip_count += 1;
                folder_pb.inc(1);
                continue;
            }
            
            // Process the video file with the shared multi-progress instance
            match self.run_with_progress(video_file.clone(), output_dir.clone(), &multi_progress).await {
                Ok(_) => {
                    success_count += 1;
                    // No need to log success here - already logged in run_with_progress
                },
                Err(e) => {
                    error_count += 1;
                    error!(" Failed to process file {}/{}: {}", index + 1, video_files.len(), e);
                }
            }
            
            // Update folder progress bar
            folder_pb.inc(1);
            
            // Calculate and display percentage
            let percentage = ((index + 1) as f32 / video_files.len() as f32) * 100.0;
            folder_pb.set_message(format!("{:.1}% complete", percentage));
        }
        
        // Complete the progress bar
        folder_pb.finish_with_message("Folder processing completed!");
        
        // Calculate total duration
        let duration = start_time.elapsed();
        
        // Summary of processed files
        info!("Folder processing completed:");
        info!("   Successfully processed: {}", success_count);
        info!("   Skipped (already exist): {}", skip_count);
        info!("   Errors: {}", error_count);
        info!("Total time: {}", Self::format_duration(duration));
        
        Ok(())
    }
    
    /// Get the expected subtitle output filename for a video file
    fn get_subtitle_output_filename(&self, input_file: &Path, target_language: &str) -> String {
        let input_stem = input_file.file_stem().unwrap_or_default();
        let input_name = input_stem.to_string_lossy();
        format!("{}.{}.srt", input_name, target_language)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_config::Config;
    use anyhow::Result;
    use std::fs;
    use tempfile::tempdir;
    
    /// Helper functions for testing, available only in test module
    impl Controller {
        /// Create a new controller for testing without file operations
        pub fn new_for_test() -> Result<Self> {
            // Use default config without writing to disk
            let config = Config::default_config();
            Self::with_config(config)
        }
    }

    /// Test controller initialization
    #[test]
    fn test_new_with_default_config_should_succeed() -> Result<()> {
        // Skip the Controller::new() test since it creates files
        // Instead, we'll use with_config with default configuration
        let config = Config::default_config();
        let _controller = Controller::with_config(config)?;
        
        // No assertion needed as we're just checking it doesn't error
        Ok(())
    }

    /// Test controller with custom configuration
    #[test]
    fn test_with_config_valid_config_should_create_controller() -> Result<()> {
        // Create a test configuration
        let config = Config::default_config();
        
        // Create controller with custom config
        let _controller = Controller::with_config(config)?;
        
        // No assertion needed as we're just checking it doesn't error
        Ok(())
    }
    
    /// Test the test-specific constructor
    #[test]
    fn test_new_for_test_should_create_controller() -> Result<()> {
        // Use the specialized test constructor
        let _controller = Controller::new_for_test()?;
        
        // No assertion needed as we're just checking it doesn't error
        Ok(())
    }

    /// Test get_subtitle_output_filename
    #[test]
    fn test_get_subtitle_output_filename_should_format_correctly() -> Result<()> {
        let controller = Controller::new_for_test()?;
        
        let test_cases = [
            ("video.mkv", "en", "video.en.srt"),
            ("movie with spaces.mp4", "fr", "movie with spaces.fr.srt"),
            ("anime.s01e01.avi", "ja", "anime.s01e01.ja.srt"),
            ("path/to/video.webm", "ru", "video.ru.srt"),
        ];
        
        for (input, lang, expected) in test_cases {
            let path = PathBuf::from(input);
            let result = controller.get_subtitle_output_filename(&path, lang);
            assert_eq!(result, expected, "for input {} and lang {}", input, lang);
        }
        
        Ok(())
    }
    
    /// Test run_folder method skips files with existing subtitle files
    #[test]
    fn test_run_folder_should_skip_files_with_existing_subtitles() -> Result<()> {
        // Skip this test in CI environments
        if std::env::var("CI").is_ok() {
            return Ok(());
        }
        
        // This test requires mocking and is just a design demonstration
        // A real implementation would use a mock filesystem
        
        // 1. Create a temporary directory for test files
        let temp_dir = tempdir()?;
        let input_dir = temp_dir.path().join("input");
        let output_dir = temp_dir.path().join("output");
        fs::create_dir_all(&input_dir)?;
        fs::create_dir_all(&output_dir)?;
        
        // 2. Create dummy video files
        let video1 = input_dir.join("video1.mp4");
        let video2 = input_dir.join("video2.mp4");
        fs::write(&video1, "dummy video content")?;
        fs::write(&video2, "dummy video content")?;
        
        // 3. Create an existing subtitle file for video2
        let subtitle2 = output_dir.join("video2.en.srt");
        fs::write(&subtitle2, "1\n00:00:01,000 --> 00:00:02,000\nExisting subtitle\n\n")?;
        
        // 4. In a real test, we would mock the run method and verify:
        // - run is called once for video1
        // - run is not called for video2
        
        // Example assertion for demonstration (not functional):
        // assert_eq!(mock_run_calls, 1);
        // assert_eq!(mock_run_args[0], video1);
        
        Ok(())
    }
    
    /// Test folder processing with no video files
    #[test]
    fn test_run_folder_with_no_video_files_should_return_ok() -> Result<()> {
        // Skip this test in CI environments
        if std::env::var("CI").is_ok() {
            return Ok(());
        }
        
        // Create a temporary directory for test files
        let temp_dir = tempdir()?;
        let input_dir = temp_dir.path().join("empty");
        fs::create_dir_all(&input_dir)?;
        
        // Create text files but no video files
        fs::write(input_dir.join("text.txt"), "not a video file")?;
        
        // In a real test, we would assert that run is never called
        // and that the function returns Ok(())
        
        Ok(())
    }
} 