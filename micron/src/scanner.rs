#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Char {
    pub char: char,
    pub n: usize,
}

impl Char {
    pub fn new(_char: char, n: usize) -> Self {
        Self { char: _char, n }
    }

    pub fn from_source(source: &String) -> Vec<Self> {
        let mut new_chars = Vec::new();

        for (n, _char) in source.chars().enumerate() {
            new_chars.push(Self { char: _char, n: n });
        }

        new_chars
    }

    pub fn extract(chars: &Vec<Char>) -> Vec<char> {
        let mut new_chars = Vec::new();

        for _char in chars.iter() {
            new_chars.push(_char.char);
        }

        new_chars
    }
}
