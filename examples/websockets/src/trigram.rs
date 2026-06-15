use std::fmt::{self, Display, Formatter};

// a binary encoding of a trigram, where the `n`th-lowest-order bit indicates
// whether the `n`th line from the top is broken. this matches the way the
// trigrams are laid out in Unicode's Miscellaneous Symbols block. we maintain
// the invariant that only the three lowest-order bits of the encoding byte are
// allowed to be non-zero
#[derive(Clone, Copy, PartialEq)]
pub struct Trigram(u8);

impl Trigram {
    pub fn new() -> Self {
        Self(0)
    }

    // check whether the `N`th line from the top is broken
    pub fn broken<const N: u8>(&self) -> bool {
        const {
            assert!(N < 3);
        }

        let Self(code) = self;
        code >> N & 1 > 0
    }

    // toggle the `n`th line between broken and unbroken. do nothing if the line
    // index `n` is out of range
    pub fn flip(&mut self, n: u8) {
        if n < 3 {
            let Self(code) = self;
            *code ^= 1 << n;
        }
    }

    pub fn name(&self) -> String {
        format!(
            "{}",
            ["乾", "兌", "離", "震", "巽", "坎", "艮", "坤"][usize::from(*self)],
        )
    }
}

impl Display for Trigram {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            ["☰", "☱", "☲", "☳", "☴", "☵", "☶", "☷"][usize::from(*self)],
        )
    }
}

// the three lowest-order bits of a byte can be interpreted as a trigram.
// instead of just storing whatever byte we receive, we choose a normal form by
// zeroing out all the bits except the three lowest-order ones
impl From<u8> for Trigram {
    fn from(code: u8) -> Self {
        Self(code & 0b111)
    }
}

impl From<Trigram> for u8 {
    fn from(trigram: Trigram) -> Self {
        let Trigram(code) = trigram;
        code
    }
}

impl From<Trigram> for usize {
    fn from(value: Trigram) -> Self {
        u8::from(value).into()
    }
}
