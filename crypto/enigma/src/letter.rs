use anyhow::{bail, Error, Result};

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Letter(u8);

pub const LETTER_COUNT: u8 = 26;

impl Letter {
    pub fn index(self) -> usize {
        self.0 as _
    }

    pub fn increment(&mut self) {
        *self = *self + Self(1)
    }

    pub fn iter_all() -> impl DoubleEndedIterator<Item = Letter> {
        (0..LETTER_COUNT).map(Letter)
    }
}

impl From<Letter> for char {
    fn from(value: Letter) -> Self {
        (b'A' + value.0) as _
    }
}

impl TryFrom<usize> for Letter {
    type Error = Error;

    fn try_from(index: usize) -> Result<Self> {
        if index >= LETTER_COUNT as usize {
            bail!("letter index cannot exceed {LETTER_COUNT}");
        }
        Ok(Self(index as u8))
    }
}

impl TryFrom<char> for Letter {
    type Error = Error;

    fn try_from(letter: char) -> Result<Self> {
        if !letter.is_ascii_alphabetic() {
            bail!("\"{letter}\" is not a latin letter")
        }
        if letter.is_ascii_lowercase() {
            Ok(Self(letter as u8 - b'a'))
        } else {
            Ok(Self(letter as u8 - b'A'))
        }
    }
}

impl std::ops::Add for Letter {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let result = self.0 + rhs.0;
        if result < LETTER_COUNT {
            Self(result)
        } else {
            Self(result - LETTER_COUNT)
        }
    }
}

impl std::ops::Sub for Letter {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        if self < rhs {
            Self(LETTER_COUNT + self.0 - rhs.0)
        } else {
            Self(self.0 - rhs.0)
        }
    }
}

impl std::fmt::Display for Letter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", char::from(*self))
    }
}

impl std::fmt::Debug for Letter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", char::from(*self), self.0)
    }
}
