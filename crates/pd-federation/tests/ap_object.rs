use std::path::PathBuf;

use pd_federation::ap_type::object;

// Returns a list of all files in the given directory path
fn get_test_data(path: PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if path.is_dir() {
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                files.push(path);
            }
        }
    }
    files
}

#[test]
fn serde_object() {
    let files = get_test_data(PathBuf::from("tests/datas/objects"));
    for file in files {
        let filename = file.file_name().unwrap().to_os_string();
        let data = std::fs::read(file).unwrap();
        if let Err(error) = sonic_rs::from_slice::<object::Object>(&data) {
            panic!(
                "Failed to deserialize object ({}): {error}",
                filename.display()
            );
        }
    }
}
