use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;

use rand::Rng;

extern crate bisection;

use bisection::bisect_right;
use rand::prelude::ThreadRng;

#[derive(Debug)]
pub struct WordSelector {
    word_list_path: PathBuf,
    letter_pos: [u64; 26],
    letter_lines_sum: [u64; 27],
}

impl Default for WordSelector {
    fn default() -> Self {
        Self::new(PathBuf::from("/usr/share/dict/words")).unwrap()
    }
}

impl WordSelector {
    pub fn new(word_list_path: PathBuf) -> Result<Self, io::Error> {
        let file = File::open(word_list_path.clone())?;

        let mut reader = BufReader::new(file);

        let mut letter_pos = [0u64; 26];
        let mut letter_lines = [0u64; 26];
        let mut num_lines = 0;
        let mut cur_letter = b'a' - 1;
        let mut cur_pos = 0;
        let mut buffer = String::new();

        fn is_letter(char: u8) -> bool {
            char >= b'a' && char <= b'z'
        }

        loop {
            buffer.clear();
            let len = reader.read_line(&mut buffer)?;

            if len == 0 {
                break;
            }

            let line = buffer.to_ascii_lowercase();
            num_lines += 1;
            let first_char = line.bytes().next().unwrap();

            if !is_letter(first_char) {
                continue;
            }

            if cur_letter != first_char {
                letter_pos[cur_pos] = reader.stream_position()?;

                letter_lines[cur_pos] = num_lines;
                num_lines = 0;

                // println!(
                //     "{}, {}, {}, {}",
                //     char::from(first_char),
                //     cur_pos,
                //     letter_pos[cur_pos],
                //     letter_lines[cur_pos],
                // );

                cur_pos += 1;
                cur_letter = first_char;
            }
        }

        letter_lines.rotate_left(1);
        letter_lines[25] = num_lines;
        let letter_lines_sum: [u64; 26] = letter_lines
            .into_iter()
            .scan(0, |acc, x| {
                *acc += x;

                Some(*acc)
            })
            .collect::<Vec<u64>>()
            .try_into()
            .unwrap();
        let mut letter_lines_sum_ = [0u64; 27];
        letter_lines_sum_[1..].copy_from_slice(&letter_lines_sum[..]);
        let letter_lines_sum = letter_lines_sum_;

        // println!("{:?}", letter_lines);
        // println!("{:?}", letter_lines_sum);

        let word_selector = Self {
            word_list_path,
            letter_pos,
            letter_lines_sum,
        };

        Ok(word_selector)
    }

    fn word_at_letter_offset(
        &self,
        letter_index: usize,
        line_offset: u64,
    ) -> Result<String, io::Error> {
        let file = File::open(self.word_list_path.clone())?;

        let mut reader = BufReader::new(file);

        reader.seek(SeekFrom::Start(self.letter_pos[letter_index]))?;

        let mut buffer = String::new();
        let mut line_no = 0;

        loop {
            buffer.clear();
            reader.read_line(&mut buffer)?;

            if line_no == line_offset {
                break;
            }

            line_no += 1
        }

        // remove trailing newline
        buffer.truncate(buffer.len() - 1);

        Ok(buffer)
    }

    fn new_word_raw(&self, rng: &mut ThreadRng) -> Result<String, io::Error> {
        let line_index = rng.gen_range(self.letter_lines_sum[0]..self.letter_lines_sum[26]);
        // let line_index = 0;

        let letter_lines_sum_index = bisect_right(&self.letter_lines_sum, &line_index);

        let line_offset = self.letter_lines_sum[letter_lines_sum_index] - line_index;

        let letter_index = letter_lines_sum_index - 1;

        // println!(
        //     "{}, {}, {}, {}",
        //     line_index,
        //     letter_lines_sum_index,
        //     self.letter_lines_sum[letter_lines_sum_index],
        //     line_offset
        // );

        self.word_at_letter_offset(letter_index, line_offset)
    }

    pub fn new_word(&self) -> Result<String, io::Error> {
        let mut rng = rand::thread_rng();

        let mut word = "-".to_string();

        while word.len() < 2 || word.len() > 8 || !word.chars().all(|c| c.is_ascii_alphabetic()) {
            word = self.new_word_raw(&mut rng)?;
        }

        word.make_ascii_lowercase();

        Ok(word)
    }
}
