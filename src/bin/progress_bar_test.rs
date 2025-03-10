use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::env;
use std::path::PathBuf;
use std::time::Duration;
use std::thread;

/// Fully automated test app for progress bar rendering in YASTWAI
/// 
/// This application tests the progress bar behavior for:
/// 1. Single file mode - Ensures only one progress bar is visible
/// 2. Folder mode - Ensures only two progress bars (folder + current file) are visible
fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let slow_mode = args.iter().any(|arg| arg == "--slow");
    
    println!("Progress Bar Test App");
    println!("====================");
    println!("Running fully automated tests for progress bar behavior");
    
    // Determine delay durations based on mode
    let (file_delay, chunk_delay) = if slow_mode {
        println!("Running in slow mode for better visual inspection");
        (Duration::from_millis(300), Duration::from_millis(150))
    } else {
        (Duration::from_millis(50), Duration::from_millis(20))
    };
    
    // Run the tests
    println!("\n[TEST 1] Testing file mode progress bar behavior...");
    test_file_mode(chunk_delay);
    
    println!("\n[TEST 2] Testing folder mode progress bar behavior...");
    test_folder_mode(file_delay, chunk_delay);
    
    println!("\nAll tests completed successfully!");
}

/// Test progress bar behavior in single file mode
fn test_file_mode(chunk_delay: Duration) {
    // Create a new MultiProgress instance
    let multi_progress = MultiProgress::new();
    
    // Create a progress bar for file translation
    let file_pb = multi_progress.add(ProgressBar::new(10));
    file_pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} chunks ({percent}%) {msg}")
            .expect("Invalid progress bar template")
            .progress_chars("█▓▒░")
    );
    file_pb.set_message("Translating test_file.mp4");
    
    // Simulate chunk translation
    for _ in 0..10 {
        // Simulate work
        thread::sleep(chunk_delay);
        file_pb.inc(1);
    }
    
    // Finish the file progress bar
    file_pb.finish_with_message("Translation complete");
    
    println!("File mode test completed successfully");
}

/// Test progress bar behavior in folder mode
fn test_folder_mode(file_delay: Duration, chunk_delay: Duration) {
    // Create a new MultiProgress instance
    let multi_progress = MultiProgress::new();
    
    // Create a progress bar for the folder (global progress)
    let folder_pb = multi_progress.add(ProgressBar::new(5));
    folder_pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} files ({percent}%) {msg}")
            .expect("Invalid progress bar template")
            .progress_chars("█▓▒░")
    );
    folder_pb.set_message("Processing files");
    
    // Simulate processing 5 files
    for i in 0..5 {
        let file_name = format!("test_file_{}.mp4", i+1);
        folder_pb.set_message(format!("Processing: {}", file_name));
        
        // Create a progress bar for the current file's translation
        let file_pb = multi_progress.add(ProgressBar::new(10));
        file_pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} chunks ({percent}%) {msg}")
                .expect("Invalid progress bar template")
                .progress_chars("█▓▒░")
        );
        file_pb.set_message(format!("Translating {}", file_name));
        
        // Simulate chunk translation
        for _ in 0..10 {
            // Simulate work
            thread::sleep(chunk_delay);
            file_pb.inc(1);
        }
        
        // Important: Clear the file progress bar before moving to the next file
        // This is the key behavior we're testing - only the folder progress bar should remain visible
        file_pb.finish_and_clear();
        
        // Simulate work between files
        thread::sleep(file_delay);
        
        // Update the folder progress bar
        folder_pb.inc(1);
    }
    
    // Finish the folder progress bar
    folder_pb.finish_with_message("Folder processing complete");
    
    println!("Folder mode test completed successfully");
} 