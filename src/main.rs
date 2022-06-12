mod wfmtools;
mod srasprocessor;
use wfmtools::wfm_tools;
use srasprocessor::scan_data;

fn main() {
    let test_file = String::from("C:\\narnia\\L0\\DC-00-0100.wfm");
    //let test_file = String::from("C:\\narnia\\RF-00-0100.wfm");
    let mut test_wfm_object = wfm_tools::WFMFile::new();
    test_wfm_object.load_file(test_file);
    let mut test_scandata = scan_data::ScanLine::new();
    test_scandata.scaled_data = test_wfm_object.file_content.scaled_frames;
    test_scandata.line_length = test_wfm_object.file_header.num_fastframes;
    test_scandata.record_length = test_wfm_object.file_header.full_record_length as u32;
    test_scandata.process_dc_values();
}