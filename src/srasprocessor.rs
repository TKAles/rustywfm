// SRAS Processing Modules

pub mod ScanData
{
    #[derive(Default)]
    pub struct ScanLine {
        pub line_length: u32,
        pub record_length: u32,
        pub scaled_data: Vec<f64>,
        pub pixel_psd_distributions: Vec<f64>,
        pub pixel_freq_bins: Vec<f64>,
        pub pixel_dc_values: Vec<f32>
    }

    impl ScanLine
    {
        pub fn new() -> ScanLine {
            let sline = ScanLine {
                ..Default::default()
            };
            sline
        }

        pub fn process_dc_values(&mut self) {
            let mut pixel_idx = 0;
            self.pixel_dc_values = Vec::with_capacity(self.line_length as usize);
            while pixel_idx < (self.line_length - 1)
            {
                let offset_b: usize = (pixel_idx * self.record_length) as usize;
                let offset_e: usize = offset_b + (self.record_length as usize);
                let current_frame: Vec<f64> = self.scaled_data[offset_b..offset_e].to_vec();
                let mut sumval: f64 = 0.0;
                for current_value in current_frame.iter()
                {
                    sumval += current_value;
                }
                sumval = sumval / (self.record_length as f64);
                self.pixel_dc_values.push(sumval as f32);
                println!("Processed pixel #{}", pixel_idx);
                pixel_idx += 1;
            }
            
            let mut outputstr: String = String::from("Pixel Values:\n");
            for current_dc_value in self.pixel_dc_values.iter()
            {
                outputstr += &current_dc_value.to_string();
                outputstr += ", ";
            }
            println!("{}", outputstr);
        }
    }

}