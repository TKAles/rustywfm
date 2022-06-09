
use std::default::Default;
use std::fs::File;
use std::io::{Read, Write};
use ascii_converter::*;

fn main() {
    //let test_file = String::from("C:\\narnia\\L0\\RF-00-0100.wfm");
    let test_file = String::from("C:\\narnia\\RF-00-0100.wfm");
    let mut test_wfm_object = WFMFile {..Default::default() };
    test_wfm_object.load_file(test_file);
    
}

#[derive(Default)]
struct WFMFile {
    file_path: String,
    file_header: WFMHeader,
    file_content: WFMContent
}
impl WFMFile {
    
    fn load_file(&mut self, input_file: String)
    {
        self.file_header = WFMHeader { ..Default::default() };
        self.file_path = input_file;
        let mut file_handle = File::open(&self.file_path).unwrap();
        let mut header_buf: [u8; 838] = [0; 838];
        file_handle.read_exact(&mut header_buf).unwrap();
        self.file_header.parse_header(&header_buf);
        
        self.file_content = Default::default();

        let mut record_index = 0;
        let mut full_buf: Vec<u8> = Vec::new();
        let mut curve_buf: Vec<i8> = Vec::with_capacity((self.file_header.full_record_length as u32 * 
                                    self.file_header.num_fastframes) as usize);
        file_handle.read_to_end(&mut full_buf).unwrap();
        while record_index < (self.file_header.num_fastframes - 1)
        {
            let offset_b: usize = (self.file_header.curve_byte_offset as usize) +
                                ((self.file_header.full_record_length as u32 * record_index) as usize);
            let offset_e: usize = offset_b + (self.file_header.full_record_length as usize);

            //println!("Record {}\tOffset_B:{}\tOffset_E:{}", record_index, offset_b, offset_e);
            
            let current_slice = &full_buf[offset_b..offset_e].to_vec();
            let mut current_row: Vec<i8> = Vec::new();
            for point in current_slice
            {
                let point_in_bytes = point.to_le_bytes();
                let point_as_int = i8::from_le_bytes(point_in_bytes);
                current_row.push(point_as_int);
            }
            //println!("Row size is {}", current_row.len());
            curve_buf.append(&mut current_row);
            //println!("Global curve buffer is {} bytes.", curve_buf.len());
            record_index += 1;
        }
        self.file_content.raw_frames = curve_buf;
        self.file_content.scaled_frames = Vec::new();
        for entry in &self.file_content.raw_frames
        {
            self.file_content.scaled_frames.push(
                (*entry as f64 * self.file_header.voltage_scale) + self.file_header.voltage_offset);
        }
    }

    fn write_csv(&self, output_file: String)
    {
        let mut outputbuf = String::new();
        let mut row_index = 0;
        let mut csv_handle = File::create(output_file).unwrap();
        for current_entry in self.file_content.scaled_frames.iter()
        {
            outputbuf += &current_entry.to_string();
            outputbuf += ",";
            row_index += 1;
            if row_index == self.file_header.full_record_length
            {
                row_index = 0;
                outputbuf += "\r\n";
            }
        }
        csv_handle.write(outputbuf.as_bytes()).unwrap();
    }
}
#[derive(Default)]

struct WFMContent {
    raw_frames: Vec<i8>,
    scaled_frames: Vec<f64>
}
/*
impl WFMContent {
    fn parse_buffer(mut self, wfm_buffer: &[u8]) 
    {
        
    }
}
*/
#[derive(Default, Clone)]
struct WFMHeader {
    wfm_version: String,
    num_impl_dim: u8,
    num_expl_dim: u8,
    record_type: u8,
    expl_dim_type: u8,
    time_base: f64,
    is_fastframe: bool,
    num_fastframes: u32,
    curve_byte_offset: u16,
    voltage_scale: f64,
    voltage_offset: f64,
    acq_time_start: f64,
    acq_time_scale: f64,
    precharge_offset: u16,
    postcharge_offset: u16,
    usable_record_length: u16,
    full_record_length: u16,
}

impl WFMHeader {
    fn parse_header(&mut self, header: &[u8])
    {
        // First idiot check. Did we pass 838 bytes into this function?
        if header.len() != 838
        {
            // Come on dude....
            panic!("Supplied vector was not 838 bytes!");
        }
        // Version check
        let version = &header[2..10];
        let version = decimals_to_string(&version.to_vec()).unwrap();
        // WFM version 3 string to check against
        let v3string = String::from(":WFM#003");
        if version.eq(&v3string)
        {
            self.wfm_version = String::from(version);
        } else {
            panic!("Not a v3 file!");
        }
        // Get # of implicit (time) and explicit (volts) dimensions
        // if not 1 ... problems
        let nimpd = &header[0x072..0x076];
        let nexpd = &header[0x076..0x07a];
        self.num_impl_dim = u32::from_le_bytes(
            nimpd.try_into().expect("Conversion Failed")) as u8;
        self.num_expl_dim = u32::from_le_bytes(
            nexpd.try_into().expect("Conversion Failed")) as u8;
        
        if(self.num_expl_dim != 1) && (self.num_impl_dim != 1) {
            // Panic if not a V v. t file
            panic!("I don't understand multidimensional data!");
        }

        let rtype = &header[0x07a..0x07e];
        let extype = &header[0x0f4..0x0f8];
        let tbase = &header[0x300..0x304];
        if u32::from_le_bytes(tbase.try_into().expect("TimeBase Failure")) != 0
        {
            panic!("Don't understand any other datatype than BASE_TIME");
        }

        self.record_type = u32::from_le_bytes(
                            rtype.try_into().expect("Record Type import failed")) as u8;
        self.expl_dim_type = u32::from_le_bytes(
                            extype.try_into().expect("Explicit Dimension failed")) as u8;
        
        let isff = &header[0x04e..0x052];
        let isff = u32::from_le_bytes(isff.try_into().expect("FastFrame type failure"));

        if isff == 1 {
            self.is_fastframe = true;
            let numframes = &header[0x048..0x04c];
            self.num_fastframes = u32::from_le_bytes(
                numframes.try_into()
                .expect("Error parsing the number of FastFrames in the file.")) + 1;
        } else if isff == 0 {
            self.is_fastframe = false;
            panic!("No FastFrames found!");
        }

        self.curve_byte_offset = 838 + ((self.num_fastframes-1)*54) as u16;
        println!("Curve data begins as offset {}", self.curve_byte_offset);

        // Voltage information and scaling info.
        let vscale = &header[0x0a8..0x0b0];
        let voffset = &header[0x0b0..0x0b8];
        let tscale = &header[0x1e8..0x1f0];
        let tstart = &header[0x1f0..0x1f8];

        self.voltage_scale = f64::from_le_bytes(vscale.try_into().expect("Problem parsing voltage scale"));
        self.voltage_offset = f64::from_le_bytes(voffset.try_into().expect("Problem parsing voltage offset"));
        self.acq_time_scale = f64::from_le_bytes(tscale.try_into().expect("Problem parsing time step"));
        self.acq_time_start = f64::from_le_bytes(tstart.try_into().expect("Problem parsing start trigger time"));
        self.time_base = self.acq_time_scale;
        println!("{}\t{}\t{}\t{}", self.voltage_scale, self.voltage_offset, self.acq_time_scale,
                    self.acq_time_start);

        // Trigger detail information is not implemented yet.

        // Pre- and Post-charge information
        let preoff = &header[0x336..0x33a];
        let postoff = &header[0x33a..0x33e];
        self.precharge_offset = u32::from_le_bytes(
            preoff.try_into().expect("Precharge conversion failed")) as u16;
        self.postcharge_offset = u32::from_le_bytes(
            postoff.try_into().expect("Postcharge conversion failed")) as u16;
        self.usable_record_length = self.postcharge_offset - 
                                    self.precharge_offset;
        let fullrec = &header[0x33e..0x342];
        self.full_record_length = u32::from_le_bytes(fullrec
            .try_into().expect("Full record length parse failed")) as u16;
        println!("Precharge: {} Postcharge: {} Record Length: {} Full Record: {}",
                    self.precharge_offset, self.postcharge_offset, self.usable_record_length, self.full_record_length);
   
        }   
}