use std::{
    fs::File,
    io::{self, BufRead, BufReader, Cursor, Seek, SeekFrom},
    path::PathBuf,
};

use crate::textgen::WordSelector;
#[derive(Debug)]
pub struct BookSelector<T> {
    reader: BufReader<T>,
    offset: u64,
}

impl<T: Seek + io::Read> BookSelector<T> {
    pub fn new(reader: BufReader<T>) -> Result<Self, io::Error> {
        let book_selector = Self { reader, offset: 0 };
        Ok(book_selector)
    }
}

impl<T: Seek + io::Read> Iterator for BookSelector<T> {
    type Item = Result<String, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = vec![];
        if let Err(e) = self.reader.seek(SeekFrom::Start(self.offset)) {
            return Some(Err(e));
        }
        match self.reader.read_until(b' ', &mut buffer) {
            Ok(len) => {
                if len == 0 {
                    return None;
                }
                self.offset += len as u64;
                Some(Ok(String::from_utf8(buffer).unwrap()))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

impl BookSelector<File> {
    pub fn from_path(file_path: PathBuf) -> Result<Self, io::Error> {
        let file = File::open(file_path)?;

        let reader = BufReader::new(file);

        Self::new(reader)
    }
}

impl BookSelector<Cursor<String>> {
    pub fn from_string(word_list: String) -> Result<Self, io::Error> {
        let cursor = Cursor::new(word_list);
        let reader = BufReader::new(cursor);

        BookSelector::new(reader)
    }
}

impl<T: Seek + io::Read> WordSelector for BookSelector<T> {
    fn new_word(&mut self) -> Result<String, io::Error> {
        loop {
            match self.next() {
                Some(word) => {
                    if let Ok(mut w) = word {
                        w = w.replace("\n", " ");
                        if w.trim() != "" && w.is_ascii() {
                            return Ok(w.trim().to_string());
                        }
                    }
                }
                None => {
                    return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
                }
            }
        }
    }
}
