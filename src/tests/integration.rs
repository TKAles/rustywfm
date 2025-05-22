// tests/integration_test.rs
// Integration tests for WFM Reader

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use wfm_reader::{WfmFile, WfmError};

/// Helper to create a test WFM file
fn create_test_wfm_file(path: &str, num_frames: u32, samples_per_frame: u32) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    
    // Create header (838 bytes)
    let mut header = vec![0u8; 838];
    
    // Version
    header[2..10].copy_from_slice(b":WFM#003");
    
    // Dimensions
    header[0x072..0x076].copy_from_slice(&1u32.to_le_bytes());
    header[0x076..0x07a].copy_from_slice(&1u32.to_le_bytes());
    
    // Time base
    header[0x300..0x304].copy_from_slice(&0u32.to_le_bytes());
    
    // FastFrame settings
    header[0x04e..0x052].copy_from_slice(&1u32.to_le_bytes());
    header[0x048..0x04c].copy_from_slice(&(num_frames - 1).to_le_bytes());
    
    // Voltage scaling
    header[0x0a8..0x0b0].copy_from_slice(&0.001f64.to_le_bytes()); // 1mV/div
    header[0x0b0..0x0b8].copy_from_slice(&0.0f64.to_le_bytes());
    
    // Time scaling
    header[0x1e8..0x1f0].copy_from_slice(&1e-9f64.to_le_bytes()); // 1ns/sample
    header[0x1f0..0x1f8].copy_from_slice(&(-10e-6f64).to_le_bytes()); // -10µs start
    
    // Record lengths
    header[0x336..0x33a].copy_from_slice(&0u32.to_le_bytes());
    header[0x33a..0x33e].copy_from_slice(&samples_per_frame.to_le_bytes());
    header[0x33e..0x342].copy_from_slice(&samples_per_frame.to_le_bytes());
    
    file.write_all(&header)?;
    
    // Write FastFrame offset data
    let offset_data = vec![0u8; 54 * (num_frames - 1) as usize];
    file.write_all(&offset_data)?;
    
    // Write waveform data
    for frame in 0..num_frames {
        for sample in 0..samples_per_frame {
            // Create a sine wave pattern
            let phase = 2.0 * std::f64::consts::PI * sample as f64 / samples_per_frame as f64;
            let amplitude = 100.0 * (1.0 + frame as f64 * 0.1); // Increasing amplitude
            let value = (amplitude * phase.sin()) as i8;
            file.write_all(&[value as u8])?;
        }
    }
    
    Ok(())
}

#[test]
fn test_load_and_process_wfm() {
    let test_file = "test_waveform.wfm";
    create_test_wfm_file(test_file, 10, 2500).expect("Failed to create test file");
    
    let mut wfm = WfmFile::new();
    wfm.load_file(test_file).expect("Failed to load WFM file");
    
    // Verify header
    assert_eq!(wfm.file_header.num_fastframes, 10);
    assert_eq!(wfm.file_header.full_record_length, 2500);
    assert_eq!(wfm.file_header.voltage_scale, 0.001);
    
    // Verify data
    assert_eq!(wfm.file_content.raw_frames.len(), 25000);
    assert_eq!(wfm.file_content.scaled_frames.len(), 25000);
    
    // Check frame access
    for i in 0..10 {
        let frame = wfm.get_frame(i).expect("Failed to get frame");
        assert_eq!(frame.len(), 2500);
    }
    
    // Clean up
    fs::remove_file(test_file).ok();
}

#[test]
fn test_csv_export() {
    let test_file = "test_export.wfm";
    let csv_file = "test_output.csv";
    
    create_test_wfm_file(test_file, 3, 100).expect("Failed to create test file");
    
    let mut wfm = WfmFile::new();
    wfm.load_file(test_file).expect("Failed to load WFM file");
    
    // Test standard CSV export
    wfm.write_csv(csv_file).expect("Failed to write CSV");
    
    // Verify CSV exists and has content
    let csv_content = fs::read_to_string(csv_file).expect("Failed to read CSV");
    let lines: Vec<&str> = csv_content.lines().collect();
    assert_eq!(lines.len(), 301); // Header + 300 data lines
    
    // Test frame-based CSV export
    let frame_csv = "test_frames.csv";
    wfm.write_csv_by_frame(frame_csv).expect("Failed to write frame CSV");
    
    let frame_content = fs::read_to_string(frame_csv).expect("Failed to read frame CSV");
    let frame_lines: Vec<&str> = frame_content.lines().collect();
    assert_eq!(frame_lines.len(), 101); // Header + 100 samples
    
    // Clean up
    fs::remove_file(test_file).ok();
    fs::remove_file(csv_file).ok();
    fs::remove_file(frame_csv).ok();
}

#[test]
fn test_error_handling() {
    // Test non-existent file
    let mut wfm = WfmFile::new();
    let result = wfm.load_file("non_existent.wfm");
    assert!(matches!(result, Err(WfmError::Io(_))));
    
    // Test invalid WFM file
    let bad_file = "bad.wfm";
    File::create(bad_file).unwrap().write_all(b"This is not a WFM file").unwrap();
    
    let mut wfm = WfmFile::new();
    let result = wfm.load_file(bad_file);
    assert!(result.is_err());
    
    fs::remove_file(bad_file).ok();
}

#[test]
fn test_time_values() {
    let test_file = "test_time.wfm";
    create_test_wfm_file(test_file, 1, 1000).expect("Failed to create test file");
    
    let mut wfm = WfmFile::new();
    wfm.load_file(test_file).expect("Failed to load WFM file");
    
    let times = wfm.get_time_values();
    assert_eq!(times.len(), 1000);
    
    // Check first and last time values
    assert_eq!(times[0], -10e-6); // -10µs
    assert_eq!(times[999], -10e-6 + 999.0 * 1e-9); // -10µs + 999ns
    
    fs::remove_file(test_file).ok();
}

// Example program showing how to use the library
#[test]
fn example_usage() {
    println!("\n=== WFM Reader Example Usage ===\n");
    
    let test_file = "example.wfm";
    create_test_wfm_file(test_file, 5, 500).expect("Failed to create test file");
    
    // Load the file
    let mut wfm = WfmFile::new();
    match wfm.load_file(test_file) {
        Ok(_) => println!("Successfully loaded WFM file"),
        Err(e) => {
            println!("Error loading file: {}", e);
            return;
        }
    }
    
    // Print header information
    println!("\nFile Information:");
    println!("  Version: {}", wfm.file_header.wfm_version);
    println!("  Number of frames: {}", wfm.file_header.num_fastframes);
    println!("  Samples per frame: {}", wfm.file_header.full_record_length);
    println!("  Voltage scale: {} V/div", wfm.file_header.voltage_scale);
    println!("  Time scale: {} s/sample", wfm.file_header.acq_time_scale);
    println!("  Acquisition start: {} s", wfm.file_header.acq_time_start);
    
    // Access individual frames
    println!("\nFrame Statistics:");
    for i in 0..wfm.file_header.num_fastframes.min(3) {
        if let Some(frame) = wfm.get_frame(i) {
            let min = frame.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = frame.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let avg = frame.iter().sum::<f64>() / frame.len() as f64;
            
            println!("  Frame {}: min={:.3}V, max={:.3}V, avg={:.3}V", 
                     i, min, max, avg);
        }
    }
    
    // Export to CSV
    if let Err(e) = wfm.write_csv_by_frame("example_output.csv") {
        println!("Error writing CSV: {}", e);
    } else {
        println!("\nExported data to example_output.csv");
    }
    
    // Clean up
    fs::remove_file(test_file).ok();
    fs::remove_file("example_output.csv").ok();
}