use std::{
    io::{Read, Seek},
    path::PathBuf,
};

use log::debug;

pub fn get_last_position(file: &PathBuf) -> u64 {
    let metadata = std::fs::metadata(file).unwrap();
    metadata.len()
}

pub fn read_file(file: &PathBuf, pos: u64) -> String {
    let mut file = std::fs::File::open(file).unwrap();
    file.seek(std::io::SeekFrom::Start(pos)).unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).unwrap();
    buffer
}

pub struct FileReader {
    pub path: PathBuf,
    pub pos: u64,
}

impl FileReader {
    pub fn new(file_path: &PathBuf) -> FileReader {
        let pos = get_last_position(file_path);
        let path = file_path.clone();
        debug!(
            "Init file reader for file {:?}, starting from position {}",
            file_path, pos
        );
        FileReader { path, pos }
    }

    pub fn read(&mut self) -> String {
        let content = read_file(&self.path, self.pos);
        self.pos += content.len() as u64;
        debug!(
            "File reader for file {:?}, read {} bytes",
            self.path,
            content.len()
        );
        content
    }

    pub fn read_lines(&mut self) -> Vec<String> {
        let content = self.read();
        content.lines().map(|s| s.to_string()).collect()
    }
}
