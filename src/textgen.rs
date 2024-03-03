//! Utilities for generating/selecting new (random) words for the typing
//! test.

use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Cursor, Seek, SeekFrom};
use std::path::PathBuf;

use rand::Rng;

use bisection::bisect_right;
use rand::prelude::ThreadRng;

/// Efficient selector of words from a word list.
///
/// The word list is given by a BufReader.
///
/// ### Assumptions
///
/// The word list is assumed to:
/// - Have a list of words separated by newline.
/// - Use only English alphabet and **ASCII**.
/// - Be **sorted alphabetically**.
///     - In case-insensitive manner.
///     - For example, both "Apple" and "apple" must appear before words
///       started with "b".
/// - Be a file that is **not modified** while the object is alive.
/// - Have no empty lines except at the end of the file.
///
/// Note: only words between length 2 and 8, inclusive, are considered.
/// Having no words matching the criteria may lead to an infinite loop.
///
/// ### Algorithm
///
/// During initialization, the [`RawWordSelector`] iterates through all
/// the words in the list and builds an index mapping each letter (of
/// the alphabet) to its byte position in the file and the cumulative
/// number of words present starting with it.
///
/// To select a (pesudo-)random word, a random number between 0
/// (inclusive) and number of lines (exclusive) is generated. Using
/// binary search, the index of where this number lies in the cumulative
/// no. of words list is found. Using this index, the byte offset of the
/// corresponding letter is found. The file is then read starting from
/// this byte offset and read line-by-line until the correct word (at
/// line `number - cumulative num. words` from the starting of this
/// letter).
///
/// ### Time complexity
///
/// Initialization: `O(n)`
///
/// Selecting a word: `O(1)` (best case) or `O(n)` (worst case)
///
/// ### Space complexity
///
/// `O(1)` (only needs fixed length arrays).
#[derive(Debug)]
pub struct RawWordSelector<T> {
    reader: BufReader<T>,
    letter_pos: [u64; 26],
    letter_lines_sum: [u64; 27],
}

impl<T: Seek + io::Read> RawWordSelector<T> {
    /// Create from any arbitrary [`BufReader`].
    ///
    /// Please ensure that assumptions defined at
    /// [`RawWordSelector#assumptions`] are valid for the contents.
    pub fn new(mut reader: BufReader<T>) -> Result<Self, io::Error> {
        let mut letter_pos = [0u64; 26];
        let mut letter_lines = [0u64; 26];
        let mut num_lines = 0;
        let mut cur_letter = b'a' - 1;
        let mut cur_pos = 0;
        let mut buffer = String::new();

        fn is_letter(char: u8) -> bool {
            char.is_ascii_lowercase()
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

                if cur_pos >= 26 {
                    break;
                }
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
            reader,
            letter_pos,
            letter_lines_sum,
        };

        Ok(word_selector)
    }

    fn word_at_letter_offset(
        &mut self,
        letter_index: usize,
        line_offset: u64,
    ) -> Result<String, io::Error> {
        self.reader
            .seek(SeekFrom::Start(self.letter_pos[letter_index]))?;

        let mut buffer = String::new();
        let mut line_no = 0;

        loop {
            buffer.clear();
            self.reader.read_line(&mut buffer)?;

            if line_no == line_offset {
                break;
            }

            line_no += 1
        }

        // remove trailing newline
        buffer.truncate(buffer.len() - 1);

        Ok(buffer)
    }

    fn new_word_raw(&mut self, rng: &mut ThreadRng) -> Result<String, io::Error> {
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
}

impl RawWordSelector<File> {
    /// Create from a file at a path given by a [`PathBuf`].
    ///
    /// Please ensure that assumptions defined at
    /// [`RawWordSelector#assumptions`] are valid for this file.
    pub fn from_path(word_list_path: PathBuf) -> Result<Self, io::Error> {
        let file = File::open(word_list_path)?;

        let reader = BufReader::new(file);

        Self::new(reader)
    }
}

impl RawWordSelector<Cursor<String>> {
    /// Create from a String representing the word list file.
    ///
    /// Please ensure that assumptions defined at
    /// [`RawWordSelector#assumptions`] are valid for the contents.
    pub fn from_string(word_list: String) -> Result<Self, io::Error> {
        let cursor = Cursor::new(word_list);
        let reader = BufReader::new(cursor);

        RawWordSelector::new(reader)
    }
}

/// Describes a thing that provides new words.
pub trait WordSelector {
    /// Returns a new word.
    fn new_word(&mut self) -> Result<String, io::Error>;

    /// Returns a [`Vec`] containing `num_words` words.
    fn new_words(&mut self, num_words: usize) -> Result<Vec<String>, io::Error> {
        (0..num_words).map(|_| self.new_word()).collect()
    }
}

impl<T: Seek + io::Read> WordSelector for RawWordSelector<T> {
    fn new_word(&mut self) -> Result<String, io::Error> {
        let mut rng = rand::thread_rng();

        let mut word = "-".to_string();

        while word.len() < 2 || word.len() > 8 || !word.chars().all(|c| c.is_ascii_alphabetic()) {
            word = self.new_word_raw(&mut rng)?;
        }

        word.make_ascii_lowercase();

        Ok(word)
    }
}

pub struct SequentialFileWordSelector {
    words: VecDeque<String>,
}

impl SequentialFileWordSelector {
    pub fn from_path(path: PathBuf) -> Result<Self, io::Error> {
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let words: VecDeque<String> = reader
            .lines()
            .filter_map(|line| line.ok())
            .flat_map(|line| {
                let words: Vec<String> = line.split_whitespace().map(String::from).collect();
                words.into_iter().filter(|word| {
                    word.chars()
                        .all(|c| c.is_ascii() && c.is_alphanumeric() && c.is_ascii_graphic())
                })
            })
            .collect();
        Ok(Self { words })
    }
}

impl WordSelector for SequentialFileWordSelector {
    fn new_word(&mut self) -> Result<String, io::Error> {
        match self.words.pop_front() {
            Some(word) => Ok(word),
            None => Err(io::Error::new(
                io::ErrorKind::Other,
                "No more words available",
            )),
        }
    }
}
