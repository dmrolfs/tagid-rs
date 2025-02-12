use once_cell::sync::Lazy;
use tailcall::tailcall;

pub static BASE_23: Lazy<Alphabet> = Lazy::new(|| Alphabet::new("ABCDEFGHJKLMNPQRSTUVXYZ"));

pub trait Codec {
    fn encode(&self, number: i64) -> String;
    fn decode(&self, rep: &str) -> i64;
}

#[derive(Debug, Clone)]
pub struct AlphabetCodec(Alphabet);

impl Default for AlphabetCodec {
    fn default() -> Self {
        Self::new(BASE_23.clone())
    }
}

impl AlphabetCodec {
    pub const fn new(alphabet: Alphabet) -> Self {
        Self(alphabet)
    }
}

#[derive(Debug, Default)]
struct ResultWithIndex {
    pub result: i64,
    pub pos: usize,
}

impl ResultWithIndex {
    pub const fn increment_w_result(self, result: i64) -> Self {
        Self {
            result,
            pos: self.pos + 1,
        }
    }
}

impl Codec for AlphabetCodec {
    fn encode(&self, number: i64) -> String {
        do_encode(&self.0, number, String::default())
    }

    fn decode(&self, rep: &str) -> i64 {
        rep.chars()
            .rev()
            .fold(ResultWithIndex::default(), |acc, c| {
                let encoded_part = self.0.index_of(c) as i64;
                let base_placement = (self.0.base as i64).pow(acc.pos as u32);
                let acc_inc = encoded_part * base_placement;
                let new_acc = acc.result + acc_inc;
                acc.increment_w_result(new_acc)
            })
            .result
    }
}

#[tailcall]
fn do_encode(alphabet: &Alphabet, number: i64, mut acc: String) -> String {
    let modulo = (number % alphabet.base as i64) as usize;
    let part = alphabet.value_of(modulo);
    acc.insert(0, part);
    if number < alphabet.base as i64 {
        acc
    } else {
        do_encode(alphabet, number / alphabet.base as i64, acc)
    }
}

#[derive(Debug, Clone)]
pub struct Alphabet {
    pub elements: String,
    pub base: usize,
}

impl Alphabet {
    pub fn new(base: impl Into<String>) -> Self {
        let elements = base.into();
        let base = elements.len();
        Self { elements, base }
    }

    pub fn value_of(&self, pos: usize) -> char {
        self.elements
            .chars()
            .nth(pos)
            .expect("failed on attempted pretty id codec out-of-bounds access.")
    }

    pub fn index_of(&self, c: char) -> usize {
        let pos = self.elements.chars().position(|a| a == c);
        pos.expect("failed to pretty id character in alphabet")
    }
}
