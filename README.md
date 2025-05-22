# WFM Reader

A Rust library and command-line tool for reading Tektronix WFM v3 files with FastFrame support.

## Features

- **Full WFM v3 Support**: Read header information, voltage scaling, time base, and FastFrame data
- **Multiple Export Options**: Export to CSV with samples in rows or frames in columns
- **Frame Access**: Extract individual frames for analysis
- **Robust Error Handling**: Comprehensive error messages for debugging
- **Memory Efficient**: Buffered I/O for CSV writing
- **Command-Line Tool**: Easy-to-use CLI for common operations

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
wfm_reader = "1.0.0"
```

Or install the command-line tool:

```bash
cargo install wfm_reader
```

## Library Usage

```rust
use wfm_reader::WfmFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load a WFM file
    let mut wfm = WfmFile::new();
    wfm.load_file("oscilloscope_capture.wfm")?;
    
    // Access header information
    println!("Number of frames: {}", wfm.file_header.num_fastframes);
    println!("Voltage scale: {} V/div", wfm.file_header.voltage_scale);
    println!("Sample rate: {} Hz", 1.0 / wfm.file_header.acq_time_scale);
    
    // Export to CSV
    wfm.write_csv("output.csv")?;
    
    // Access individual frames
    for i in 0..wfm.file_header.num_fastframes {
        if let Some(frame) = wfm.get_frame(i) {
            let max_voltage = frame.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            println!("Frame {} max voltage: {:.3} V", i, max_voltage);
        }
    }
    
    Ok(())
}
```

## Command-Line Usage

### Display file information
```bash
wfm_reader info capture.wfm
```

### Convert to CSV (all samples in rows)
```bash
wfm_reader convert capture.wfm output.csv
```

### Export frames as columns
```bash
wfm_reader frames capture.wfm frames.csv
```

### Extract a single frame
```bash
wfm_reader extract capture.wfm 0 > frame0.txt
```

## File Format Support

This library supports Tektronix WFM version 3 files with the following features:

- FastFrame data (required)
- Single implicit dimension (time)
- Single explicit dimension (voltage)
- 8-bit signed integer raw data
- Voltage scaling and offset
- Time base and acquisition start time
- Pre-charge and post-charge offsets

## Building from Source

```bash
# Clone the repository
git clone https://github.com/TKAles/rustywfm.git
cd wfm_reader

# Build the library
cargo build --release

# Run tests
cargo test

# Build and install the CLI tool
cargo install --path .
```

## Contributing
This is a hella spare time project though so don't expect much if only issues are submitted. Pulls will be gotten to eventually.

## License

This project is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Technical Notes
### Voltage Calculation

Actual voltage = (raw_value × voltage_scale) + voltage_offset

### Time Calculation

Time for sample n = acquisition_start_time + (n × time_scale)
