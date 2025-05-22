// src/main.rs
// Example command-line application for WFM Reader

use std::env;
use std::process;
use wfm_reader::WfmFile;

fn print_usage() {
    eprintln!("Usage: wfm_reader <command> <wfm_file> [options]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  info <file>              Display WFM file information");
    eprintln!("  convert <file> <output>  Convert WFM to CSV");
    eprintln!("  frames <file> <output>   Export frames as columns to CSV");
    eprintln!("  extract <file> <frame>   Extract a single frame to stdout");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  wfm_reader info capture.wfm");
    eprintln!("  wfm_reader convert capture.wfm output.csv");
    eprintln!("  wfm_reader frames capture.wfm frames.csv");
    eprintln!("  wfm_reader extract capture.wfm 0 > frame0.txt");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        print_usage();
        process::exit(1);
    }
    
    let command = &args[1];
    let input_file = &args[2];
    
    // Load the WFM file
    let mut wfm = WfmFile::new();
    if let Err(e) = wfm.load_file(input_file) {
        eprintln!("Error loading WFM file '{}': {}", input_file, e);
        process::exit(1);
    }
    
    match command.as_str() {
        "info" => {
            print_file_info(&wfm);
        }
        
        "convert" => {
            if args.len() < 4 {
                eprintln!("Error: Missing output file argument");
                print_usage();
                process::exit(1);
            }
            
            let output_file = &args[3];
            if let Err(e) = wfm.write_csv(output_file) {
                eprintln!("Error writing CSV file '{}': {}", output_file, e);
                process::exit(1);
            }
            
            println!("Successfully converted {} to {}", input_file, output_file);
            println!("Total samples written: {}", wfm.file_content.scaled_frames.len());
        }
        
        "frames" => {
            if args.len() < 4 {
                eprintln!("Error: Missing output file argument");
                print_usage();
                process::exit(1);
            }
            
            let output_file = &args[3];
            if let Err(e) = wfm.write_csv_by_frame(output_file) {
                eprintln!("Error writing frame CSV file '{}': {}", output_file, e);
                process::exit(1);
            }
            
            println!("Successfully exported {} frames to {}", 
                     wfm.file_header.num_fastframes, output_file);
        }
        
        "extract" => {
            if args.len() < 4 {
                eprintln!("Error: Missing frame number argument");
                print_usage();
                process::exit(1);
            }
            
            let frame_num: u32 = match args[3].parse() {
                Ok(n) => n,
                Err(_) => {
                    eprintln!("Error: Invalid frame number '{}'", args[3]);
                    process::exit(1);
                }
            };
            
            match wfm.get_frame(frame_num) {
                Some(frame_data) => {
                    let times = wfm.get_time_values();
                    println!("# Frame {} from {}", frame_num, input_file);
                    println!("# Time (s), Voltage (V)");
                    
                    for (i, &voltage) in frame_data.iter().enumerate() {
                        println!("{:.12e}, {:.6e}", times[i], voltage);
                    }
                }
                None => {
                    eprintln!("Error: Frame {} not found (file has {} frames)", 
                             frame_num, wfm.file_header.num_fastframes);
                    process::exit(1);
                }
            }
        }
        
        _ => {
            eprintln!("Error: Unknown command '{}'", command);
            print_usage();
            process::exit(1);
        }
    }
}

fn print_file_info(wfm: &WfmFile) {
    println!("WFM File Information");
    println!("====================");
    println!();
    println!("File: {}", wfm.file_path);
    println!("Version: {}", wfm.file_header.wfm_version);
    println!();
    
    println!("Acquisition Parameters:");
    println!("  FastFrame enabled: {}", wfm.file_header.is_fastframe);
    println!("  Number of frames: {}", wfm.file_header.num_fastframes);
    println!("  Samples per frame: {}", wfm.file_header.full_record_length);
    println!("  Usable samples: {}", wfm.file_header.usable_record_length);
    println!("  Total samples: {}", wfm.file_content.scaled_frames.len());
    println!();
    
    println!("Voltage Scaling:");
    println!("  Scale factor: {} V/division", wfm.file_header.voltage_scale);
    println!("  Offset: {} V", wfm.file_header.voltage_offset);
    
    // Calculate voltage range from data
    if !wfm.file_content.scaled_frames.is_empty() {
        let min_v = wfm.file_content.scaled_frames.iter()
            .fold(f64::INFINITY, |a, &b| a.min(b));
        let max_v = wfm.file_content.scaled_frames.iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        println!("  Data range: {:.3} V to {:.3} V", min_v, max_v);
        println!("  Peak-to-peak: {:.3} V", max_v - min_v);
    }
    println!();
    
    println!("Time Scaling:");
    println!("  Sample interval: {:.3e} s ({:.3} MHz sample rate)", 
             wfm.file_header.acq_time_scale,
             1.0 / wfm.file_header.acq_time_scale / 1e6);
    println!("  Acquisition start: {:.6e} s", wfm.file_header.acq_time_start);
    
    let duration = wfm.file_header.full_record_length as f64 * wfm.file_header.acq_time_scale;
    println!("  Frame duration: {:.6e} s", duration);
    println!();
    
    println!("Data Layout:");
    println!("  Header size: 838 bytes");
    println!("  Curve data offset: {} bytes", wfm.file_header.curve_byte_offset);
    println!("  Precharge offset: {}", wfm.file_header.precharge_offset);
    println!("  Postcharge offset: {}", wfm.file_header.postcharge_offset);
    println!();
    
    // Show statistics for first few frames
    println!("Frame Statistics (first {} frames):", 3.min(wfm.file_header.num_fastframes));
    for i in 0..wfm.file_header.num_fastframes.min(3) {
        if let Some(frame) = wfm.get_frame(i) {
            let min = frame.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = frame.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let avg = frame.iter().sum::<f64>() / frame.len() as f64;
            let rms = (frame.iter().map(|&x| x * x).sum::<f64>() / frame.len() as f64).sqrt();
            
            println!("  Frame {}: min={:.3}V, max={:.3}V, avg={:.3}V, rms={:.3}V", 
                     i, min, max, avg, rms);
        }
    }
}