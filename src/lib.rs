// src/lib.rs
// WFM Reader Library - Public API

//! # WFM Reader
//! 
//! A Rust library for reading Tektronix WFM v3 files with FastFrame support.
//! 
//! ## Features
//! 
//! - Read WFM v3 files with FastFrame data
//! - Extract voltage scaling and timing information
//! - Export data to CSV format
//! - Access individual frames
//! - Proper error handling
//! 
//! ## Example
//! 
//! ```no_run
//! use wfm_reader::WfmFile;
//! 
//! let mut wfm = WfmFile::new();
//! wfm.load_file("capture.wfm").expect("Failed to load file");
//! 
//! println!("Number of frames: {}", wfm.file_header.num_fastframes);
//! println!("Samples per frame: {}", wfm.file_header.full_record_length);
//! 
//! // Export to CSV
//! wfm.write_csv("output.csv").expect("Failed to write CSV");
//! 
//! // Access a specific frame
//! if let Some(frame) = wfm.get_frame(0) {
//!     println!("First sample: {} V", frame[0]);
//! }
//! ```

mod wfm_tools;

pub use wfm_tools::{WfmFile, WfmHeader, WfmContent, WfmError, Result};