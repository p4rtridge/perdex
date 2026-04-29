use std::path::PathBuf;

use pd_federation::ap_type::actor;

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
fn serde_actor() {
    let files = get_test_data(PathBuf::from("tests/datas/actors"));
    for file in files {
        let filename = file.file_name().unwrap().to_os_string();
        let data = std::fs::read(file).unwrap();
        if let Err(error) = sonic_rs::from_slice::<actor::Actor>(&data) {
            panic!(
                "Failed to deserialize actor ({}): {error}",
                filename.display()
            );
        }
    }
}
