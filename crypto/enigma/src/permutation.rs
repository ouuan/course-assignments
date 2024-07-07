use crate::letter::{Letter, LETTER_COUNT};
use anyhow::{bail, Result};

#[derive(Clone, Default)]
pub struct Permutation([Letter; LETTER_COUNT as _]);

impl Permutation {
    pub fn inv(&self) -> Self {
        let mut p = Self::default();
        for (i, x) in self.0.iter().enumerate() {
            p[*x] = i.try_into().unwrap();
        }
        p
    }

    pub fn identity() -> Self {
        Self(std::array::from_fn(|i| i.try_into().unwrap()))
    }
}

impl std::ops::Index<Letter> for Permutation {
    type Output = Letter;

    fn index(&self, index: Letter) -> &Self::Output {
        &self.0[index.index()]
    }
}

impl std::ops::IndexMut<Letter> for Permutation {
    fn index_mut(&mut self, index: Letter) -> &mut Self::Output {
        &mut self.0[index.index()]
    }
}

impl TryFrom<&str> for Permutation {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self> {
        if value.len() != LETTER_COUNT as _ {
            bail!("invalid letter permutation length {}", value.len());
        }

        let result = Self(
            value
                .chars()
                .map(Letter::try_from)
                .collect::<Result<Vec<_>>>()?
                .try_into()
                .unwrap(),
        );

        let mut used = vec![false; LETTER_COUNT as _];

        for x in &result.0 {
            if used[x.index()] {
                bail!("invalid letter permutation: duplicate {x}");
            }
            used[x.index()] = true;
        }

        Ok(result)
    }
}
