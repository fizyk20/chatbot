use rand::{self, Rng};
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

fn to_4u8(x: u32) -> [u8; 4] {
    let mut result = [0; 4];
    result[0] = (x & 0xFF) as u8;
    result[1] = ((x >> 8) & 0xFF) as u8;
    result[2] = ((x >> 16) & 0xFF) as u8;
    result[3] = ((x >> 24) & 0xFF) as u8;
    result
}

fn to_u32(x: &[u8]) -> Option<u32> {
    if x.len() < 4 {
        return None;
    }
    Some(x[0] as u32 + ((x[1] as u32) << 8) + ((x[2] as u32) << 16) + ((x[3] as u32) << 24))
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Word {
    Start1,
    Start2,
    Word(u32),
    End,
}

impl Word {
    pub fn into_bytes(&self) -> [u8; 5] {
        let mut result = [0; 5];
        match *self {
            Word::Start1 => {
                result[0] = 1;
            }
            Word::Start2 => {
                result[0] = 2;
            }
            Word::End => {
                result[0] = 0xFF;
            }
            Word::Word(i) => {
                let i_bytes = to_4u8(i);
                for j in 0..4 {
                    result[j + 1] = i_bytes[j];
                }
            }
        }
        result
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Word> {
        if bytes.len() < 5 {
            return None;
        }

        match bytes[0] {
            0 => Some(Word::Word(to_u32(&bytes[1..5]).unwrap())),
            1 => Some(Word::Start1),
            2 => Some(Word::Start2),
            0xFF => Some(Word::End),
            _ => None,
        }
    }
}

pub type Entry = (Word, Word);

struct ByteReader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> ByteReader<'a> {
    fn new(bytes: &[u8]) -> ByteReader {
        ByteReader {
            bytes: bytes,
            cursor: 0,
        }
    }

    fn read_u32(&mut self) -> u32 {
        let result = to_u32(&self.bytes[self.cursor..self.cursor + 4]).unwrap();
        self.cursor += 4;
        result
    }

    fn read_word(&mut self) -> Word {
        let result = Word::from_bytes(&self.bytes[self.cursor..self.cursor + 5]).unwrap();
        self.cursor += 5;
        result
    }

    fn read_string(&mut self) -> String {
        let word_length = to_u32(&self.bytes[self.cursor..self.cursor + 4]).unwrap() as usize;
        self.cursor += 4;
        let word =
            ::std::str::from_utf8(&self.bytes[self.cursor..self.cursor + word_length]).unwrap();
        self.cursor += word_length;
        word.to_string()
    }
}

pub struct Dictionary {
    words: Vec<String>,
    index_map: HashMap<String, usize>,
    dict: HashMap<Entry, BTreeMap<Word, u32>>,
}

impl Dictionary {
    #[allow(unused)]
    pub fn new() -> Dictionary {
        Dictionary {
            words: Vec::new(),
            index_map: HashMap::new(),
            dict: HashMap::new(),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        // push number of words
        result.extend_from_slice(&to_4u8(self.words.len() as u32));
        // push each word preceded by its length
        for word in (&self.words).into_iter() {
            let bytes = word.as_bytes();
            result.extend_from_slice(&to_4u8(bytes.len() as u32));
            result.extend_from_slice(bytes);
        }
        // write dict
        // first, the number of entries
        result.extend_from_slice(&to_4u8(self.dict.len() as u32));
        // now the entries
        for key in self.dict.keys() {
            // first, the key
            result.extend_from_slice(&key.0.into_bytes());
            result.extend_from_slice(&key.1.into_bytes());
            // second, possible results
            let data = &self.dict[key];
            // btreemap length
            result.extend_from_slice(&to_4u8(data.len() as u32));
            // and entries
            for (word, chance) in data {
                result.extend_from_slice(&word.into_bytes());
                result.extend_from_slice(&to_4u8(*chance));
            }
        }

        result
    }

    fn from_bytes(bytes: &[u8]) -> Option<Dictionary> {
        let mut reader = ByteReader::new(bytes);
        let num_words = reader.read_u32();
        let mut words = Vec::new();
        let mut index_map = HashMap::new();
        // read words
        for i in 0..num_words {
            let word = reader.read_string();
            words.push(word.clone());
            index_map.insert(word.to_lowercase(), i as usize);
        }
        // read entry map
        let num_entries = reader.read_u32();
        let mut hashmap = HashMap::new();
        for _ in 0..num_entries {
            // first entry word
            let word1 = reader.read_word();
            // second entry word
            let word2 = reader.read_word();
            let num_results = reader.read_u32();
            let mut results = BTreeMap::new();
            for _ in 0..num_results {
                let word = reader.read_word();
                let chance = reader.read_u32();
                results.insert(word, chance);
            }
            hashmap.insert((word1, word2), results);
        }
        Some(Dictionary {
            words: words,
            index_map: index_map,
            dict: hashmap,
        })
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut file = File::create(path)?;
        let bytes = self.to_bytes();
        file.write_all(&bytes)?;
        Ok(())
    }

    pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Dictionary> {
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        if let Some(dict) = Dictionary::from_bytes(&bytes) {
            Ok(dict)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid dictionary input",
            ))
        }
    }

    fn insert_word<S: AsRef<str>>(&mut self, word: S) -> usize {
        if let Some(index) = self.index_map.get(&word.as_ref().to_lowercase()) {
            return *index;
        }
        self.words.push(word.as_ref().to_string());
        self.index_map.insert(
            word.as_ref().to_lowercase().to_string(),
            self.words.len() - 1,
        );
        self.words.len() - 1
    }

    pub fn learn_from_line<S: AsRef<str>>(&mut self, line: S) {
        let words = line.as_ref().split_whitespace();
        let mut words_new = vec![Word::Start1, Word::Start2];
        words_new.extend(words.map(|x| Word::Word(self.insert_word(x) as u32)));
        words_new.push(Word::End);

        for window in words_new.windows(3) {
            let entry = (window[0], window[1]);
            let word = window[2];
            if let Some(data) = self.dict.get_mut(&entry) {
                if let Some(chance) = data.get_mut(&word) {
                    *chance += 1;
                    continue;
                }
                data.insert(word, 1);
                continue;
            }
            let mut map = BTreeMap::new();
            map.insert(word, 1);
            self.dict.insert(entry, map);
        }
    }

    fn get_next_word(&self, w1: Word, w2: Word) -> Option<Word> {
        let mut rng = rand::thread_rng();
        let possibilities;
        if let Some(p) = self.dict.get(&(w1, w2)) {
            possibilities = p;
        } else {
            return None;
        }
        let mut sum = 0;
        for (_, v) in possibilities.iter() {
            sum += *v;
        }

        let mut random = rng.gen_range(0, sum);
        for (&k, &v) in possibilities.iter() {
            if random < v {
                return Some(k);
            }
            random -= v;
        }

        None
    }

    pub fn generate_sentence(&self) -> String {
        let mut w1 = Word::Start1;
        let mut w2 = Word::Start2;

        let mut words = Vec::new();
        loop {
            let next_word;
            if let Some(nw) = self.get_next_word(w1, w2) {
                next_word = nw;
            } else {
                break;
            }
            if next_word == Word::End {
                break;
            }
            if let Word::Word(index) = next_word {
                words.push(self.words[index as usize].clone());
            }
            w1 = w2;
            w2 = next_word;
        }

        if !words.is_empty() {
            let mut result = words[0].to_string();
            for word in &words[1..] {
                result.push_str(" ");
                result.push_str(word);
            }
            result
        } else {
            String::new()
        }
    }
}
