mod wfmtools;
use wfmtools::wfm_tools;


fn main() {
    let test_file = String::from("C:\\narnia\\L0\\RF-00-0100.wfm");
    //let test_file = String::from("C:\\narnia\\RF-00-0100.wfm");
    let mut test_wfm_object = wfm_tools::WFMFile::new();
    test_wfm_object.load_file(test_file);
}