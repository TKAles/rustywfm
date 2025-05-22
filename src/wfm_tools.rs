// WFMReader Module
// TK Ales, 2022
// Version 1.0 - Corrected version

use std::fs::File;
use std::io::{self, BufWriter, Read, Write};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WfmError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Invalid header size: expected 838 bytes, got {0}")]
    InvalidHeaderSize(usize),
    
    #[error("Unsupported WFM version: {0}")]
    UnsupportedVersion(String),
    
    #[error("Invalid dimensions: implicit={0}, explicit={1}")]
    InvalidDimensions(u8, u8),
    
    #[error("Unsupported time base type")]
    UnsupportedTimeBase,
    
    #[error("No FastFrames found in file")]
    NoFastFrames,
    
    #[error("Parse error: {0}")]
    ParseError(String),
}

pub type Result<T> = std::result::Result<T, WfmError>;

/// Representation of the WFM file header as decoded.
#[derive(Default, Clone, Debug)]
pub struct WfmHeader {
    pub wfm_version: String,
    pub num_impl_dim: u8,
    pub num_expl_dim: u8,
    pub record_type: u8,
    pub expl_dim_type: u8,
    pub time_base: f64,
    pub is_fastframe: bool,
    pub num_fastframes: u32,
    pub curve_byte_offset: u16,
    pub voltage_scale: f64,
    pub voltage_offset: f64,
    pub acq_time_start: f64,
    pub acq_time_scale: f64,
    pub precharge_offset: u16,
    pub postcharge_offset: u16,
    pub usable_record_length: u16,
    pub full_record_length: u16,
}

impl WfmHeader {
    /// Parse the header from a byte array
    fn parse_header(header: &[u8]) -> Result<Self> {
        if header.len() != 838 {
            return Err(WfmError::InvalidHeaderSize(header.len()));
        }
        
        let mut wfm = WfmHeader::default();
        
        // Version check
        let version = std::str::from_utf8(&header[2..10])
            .map_err(|e| WfmError::ParseError(format!("Invalid version string: {}", e)))?;
        
        if version != ":WFM#003" {
            return Err(WfmError::UnsupportedVersion(version.to_string()));
        }
        wfm.wfm_version = version.to_string();
        
        // Get dimensions
        wfm.num_impl_dim = Self::read_u32(&header[0x072..0x076])? as u8;
        wfm.num_expl_dim = Self::read_u32(&header[0x076..0x07a])? as u8;
        
        if wfm.num_impl_dim != 1 || wfm.num_expl_dim != 1 {
            return Err(WfmError::InvalidDimensions(wfm.num_impl_dim, wfm.num_expl_dim));
        }
        
        // Record types
        wfm.record_type = Self::read_u32(&header[0x07a..0x07e])? as u8;
        wfm.expl_dim_type = Self::read_u32(&header[0x0f4..0x0f8])? as u8;
        
        // Time base check
        let tbase = Self::read_u32(&header[0x300..0x304])?;
        if tbase != 0 {
            return Err(WfmError::UnsupportedTimeBase);
        }
        
        // FastFrame information
        let is_ff = Self::read_u32(&header[0x04e..0x052])?;
        if is_ff == 1 {
            wfm.is_fastframe = true;
            wfm.num_fastframes = Self::read_u32(&header[0x048..0x04c])? + 1;
        } else {
            return Err(WfmError::NoFastFrames);
        }
        
        wfm.curve_byte_offset = 838 + ((wfm.num_fastframes - 1) * 54) as u16;
        
        // Voltage and time scaling
        wfm.voltage_scale = Self::read_f64(&header[0x0a8..0x0b0])?;
        wfm.voltage_offset = Self::read_f64(&header[0x0b0..0x0b8])?;
        wfm.acq_time_scale = Self::read_f64(&header[0x1e8..0x1f0])?;
        wfm.acq_time_start = Self::read_f64(&header[0x1f0..0x1f8])?;
        wfm.time_base = wfm.acq_time_scale;
        
        // Pre- and Post-charge information
        wfm.precharge_offset = Self::read_u32(&header[0x336..0x33a])? as u16;
        wfm.postcharge_offset = Self::read_u32(&header[0x33a..0x33e])? as u16;
        wfm.usable_record_length = wfm.postcharge_offset - wfm.precharge_offset;
        wfm.full_record_length = Self::read_u32(&header[0x33e..0x342])? as u16;
        
        Ok(wfm)
    }
    
    fn read_u32(bytes: &[u8]) -> Result<u32> {
        bytes.try_into()
            .map(u32::from_le_bytes)
            .map_err(|_| WfmError::ParseError("Failed to parse u32".to_string()))
    }
    
    fn read_f64(bytes: &[u8]) -> Result<f64> {
        bytes.try_into()
            .map(f64::from_le_bytes)
            .map_err(|_| WfmError::ParseError("Failed to parse f64".to_string()))
    }
}

/// Container for WFM file content
#[derive(Default, Debug)]
pub struct WfmContent {
    pub raw_frames: Vec<i8>,
    pub scaled_frames: Vec<f64>,
}

/// Main WFM file reader
#[derive(Default)]
pub struct WfmFile {
    pub file_path: String,
    pub file_header: WfmHeader,
    pub file_content: WfmContent,
}

impl WfmFile {
    /// Create a new WfmFile instance
    pub fn new() -> Self {
        WfmFile::default()
    }
    
    /// Load a WFM file from the given path
    pub fn load_file<P: AsRef<Path>>(&mut self, input_file: P) -> Result<()> {
        self.file_path = input_file.as_ref().to_string_lossy().to_string();
        
        let mut file_handle = File::open(&input_file)?;
        
        // Read and parse header
        let mut header_buf = [0u8; 838];
        file_handle.read_exact(&mut header_buf)?;
        self.file_header = WfmHeader::parse_header(&header_buf)?;
        
        // Read curve data
        self.file_content = WfmContent::default();
        
        let total_samples = self.file_header.full_record_length as usize * 
                           self.file_header.num_fastframes as usize;
        
        self.file_content.raw_frames.reserve(total_samples);
        self.file_content.scaled_frames.reserve(total_samples);
        
        // Read all remaining data
        let mut full_buf = Vec::new();
        file_handle.read_to_end(&mut full_buf)?;
        
        // Process each frame
        for record_index in 0..self.file_header.num_fastframes {
            // full_buf starts after the header, so we need to adjust the offset
            let offset_b = ((self.file_header.curve_byte_offset - 838) as usize) +
                          (self.file_header.full_record_length as usize * record_index as usize);
            let offset_e = offset_b + self.file_header.full_record_length as usize;
            
            if offset_e > full_buf.len() {
                return Err(WfmError::ParseError(
                    format!("Unexpected end of file at frame {}: offset {} > buffer length {}", 
                            record_index, offset_e, full_buf.len())
                ));
            }
            
            // Convert bytes to signed integers and scale
            for &byte in &full_buf[offset_b..offset_e] {
                let value = byte as i8;
                self.file_content.raw_frames.push(value);
                
                let scaled = (value as f64 * self.file_header.voltage_scale) + 
                            self.file_header.voltage_offset;
                self.file_content.scaled_frames.push(scaled);
            }
        }
        
        Ok(())
    }
    
    /// Write the scaled data to a CSV file
    pub fn write_csv<P: AsRef<Path>>(&self, output_file: P) -> Result<()> {
        let file = File::create(output_file)?;
        let mut writer = BufWriter::new(file);
        
        // Write header row
        writeln!(writer, "Sample,Voltage")?;
        
        // Write data
        for (idx, &value) in self.file_content.scaled_frames.iter().enumerate() {
            let frame_num = idx / self.file_header.full_record_length as usize;
            let sample_num = idx % self.file_header.full_record_length as usize;
            writeln!(writer, "{},{},{}", frame_num, sample_num, value)?;
        }
        
        writer.flush()?;
        Ok(())
    }
    
    /// Write frames as separate columns
    pub fn write_csv_by_frame<P: AsRef<Path>>(&self, output_file: P) -> Result<()> {
        let file = File::create(output_file)?;
        let mut writer = BufWriter::new(file);
        
        // Write header
        write!(writer, "Sample")?;
        for i in 0..self.file_header.num_fastframes {
            write!(writer, ",Frame{}", i)?;
        }
        writeln!(writer)?;
        
        // Write data row by row
        for sample in 0..self.file_header.full_record_length as usize {
            write!(writer, "{}", sample)?;
            
            for frame in 0..self.file_header.num_fastframes as usize {
                let idx = frame * self.file_header.full_record_length as usize + sample;
                write!(writer, ",{}", self.file_content.scaled_frames[idx])?;
            }
            writeln!(writer)?;
        }
        
        writer.flush()?;
        Ok(())
    }
    
    /// Get voltage data for a specific frame
    pub fn get_frame(&self, frame_index: u32) -> Option<&[f64]> {
        if frame_index >= self.file_header.num_fastframes {
            return None;
        }
        
        let start = frame_index as usize * self.file_header.full_record_length as usize;
        let end = start + self.file_header.full_record_length as usize;
        
        Some(&self.file_content.scaled_frames[start..end])
    }
    
    /// Get time values for samples
    pub fn get_time_values(&self) -> Vec<f64> {
        let mut times = Vec::with_capacity(self.file_header.full_record_length as usize);
        
        for i in 0..self.file_header.full_record_length {
            let t = self.file_header.acq_time_start + 
                   (i as f64 * self.file_header.acq_time_scale);
            times.push(t);
        }
        
        times
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    fn create_test_header() -> Vec<u8> {
        let mut header = vec![0u8; 838];
        
        // Write version string
        header[2..10].copy_from_slice(b":WFM#003");
        
        // Set dimensions to 1
        header[0x072..0x076].copy_from_slice(&1u32.to_le_bytes());
        header[0x076..0x07a].copy_from_slice(&1u32.to_le_bytes());
        
        // Set time base to 0
        header[0x300..0x304].copy_from_slice(&0u32.to_le_bytes());
        
        // Enable FastFrame
        header[0x04e..0x052].copy_from_slice(&1u32.to_le_bytes());
        
        // Set number of frames (stored as n-1)
        header[0x048..0x04c].copy_from_slice(&4u32.to_le_bytes()); // 5 frames
        
        // Set voltage scale and offset
        header[0x0a8..0x0b0].copy_from_slice(&0.01f64.to_le_bytes());
        header[0x0b0..0x0b8].copy_from_slice(&0.0f64.to_le_bytes());
        
        // Set time scale and start
        header[0x1e8..0x1f0].copy_from_slice(&1e-9f64.to_le_bytes());
        header[0x1f0..0x1f8].copy_from_slice(&(-5e-6f64).to_le_bytes());
        
        // Set record lengths
        header[0x336..0x33a].copy_from_slice(&0u32.to_le_bytes());
        header[0x33a..0x33e].copy_from_slice(&1000u32.to_le_bytes());
        header[0x33e..0x342].copy_from_slice(&1000u32.to_le_bytes());
        
        header
    }
    
    #[test]
    fn test_header_parsing() {
        let header_data = create_test_header();
        let header = WfmHeader::parse_header(&header_data).unwrap();
        
        assert_eq!(header.wfm_version, ":WFM#003");
        assert_eq!(header.num_impl_dim, 1);
        assert_eq!(header.num_expl_dim, 1);
        assert_eq!(header.num_fastframes, 5);
        assert_eq!(header.voltage_scale, 0.01);
        assert_eq!(header.voltage_offset, 0.0);
        assert_eq!(header.full_record_length, 1000);
    }
    
    #[test]
    fn test_invalid_header_size() {
        let header_data = vec![0u8; 100];
        let result = WfmHeader::parse_header(&header_data);
        assert!(matches!(result, Err(WfmError::InvalidHeaderSize(100))));
    }
    
    #[test]
    fn test_file_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        
        // Write test header
        let header = create_test_header();
        temp_file.write_all(&header).unwrap();
        
        // Write FastFrame offset data (54 bytes per frame, 4 frames since num_fastframes-1)
        temp_file.write_all(&vec![0u8; 54 * 4]).unwrap();
        
        // Write test curve data (5 frames, 1000 samples each)
        for frame in 0..5 {
            for sample in 0..1000 {
                let value = ((sample as i32 - 500) / 5) as i8;
                temp_file.write_all(&[value as u8]).unwrap();
            }
        }
        
        temp_file.flush().unwrap();
        
        // Load the file
        let mut wfm = WfmFile::new();
        let result = wfm.load_file(temp_file.path());
        assert!(result.is_ok(), "Failed to load file: {:?}", result.err());
        
        assert_eq!(wfm.file_header.num_fastframes, 5);
        assert_eq!(wfm.file_content.raw_frames.len(), 5000);
        assert_eq!(wfm.file_content.scaled_frames.len(), 5000);
    }
    
    #[test]
    fn test_frame_access() {
        let mut wfm = WfmFile::new();
        wfm.file_header.num_fastframes = 2;
        wfm.file_header.full_record_length = 3;
        wfm.file_content.scaled_frames = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        
        let frame0 = wfm.get_frame(0).unwrap();
        assert_eq!(frame0, &[1.0, 2.0, 3.0]);
        
        let frame1 = wfm.get_frame(1).unwrap();
        assert_eq!(frame1, &[4.0, 5.0, 6.0]);
        
        assert!(wfm.get_frame(2).is_none());
    }
    
    #[test]
    fn test_time_values() {
        let mut wfm = WfmFile::new();
        wfm.file_header.acq_time_start = 0.0;
        wfm.file_header.acq_time_scale = 0.1;
        wfm.file_header.full_record_length = 5;
        
        let times = wfm.get_time_values();
        let expected = vec![0.0, 0.1, 0.2, 0.3, 0.4];
        
        assert_eq!(times.len(), expected.len());
        for (i, (&actual, &expected)) in times.iter().zip(expected.iter()).enumerate() {
            assert!((actual - expected).abs() < 1e-10, 
                    "Time value {} mismatch: {} != {}", i, actual, expected);
        }
    }
}