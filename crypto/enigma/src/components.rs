use crate::permutation::Permutation;
use crate::Letter;
use anyhow::{bail, Result};
use log::info;
use std::sync::OnceLock;

#[derive(Clone)]
pub struct Rotor {
    ring_setting: Letter,
    position: Letter,
    turnover_notch: Letter,
    permutation: &'static Permutation,
    permutation_inv: &'static Permutation,
}

impl Rotor {
    /// Create a new rotor with given ring setting, initial position, and rotor type (I/II/III)
    pub fn new(setting: RotorSetting) -> Self {
        Self {
            ring_setting: setting.ring_setting,
            position: setting.initial_position,
            turnover_notch: setting.rotor_type.turnover_notch(),
            permutation: setting.rotor_type.permutation(),
            permutation_inv: setting.rotor_type.permutation_inv(),
        }
    }

    /// Translate a letter in the forward translation
    pub fn translate_forward(&self, input: Letter) -> Letter {
        self.permutation[input + self.position - self.ring_setting] - self.position
            + self.ring_setting
    }

    /// Translate a letter in the backward translation
    pub fn translate_backward(&self, input: Letter) -> Letter {
        self.permutation_inv[input + self.position - self.ring_setting] - self.position
            + self.ring_setting
    }

    /// Returns whether turnover will occur in next turn
    pub fn about_to_turnover(&self) -> bool {
        self.position == self.turnover_notch
    }

    /// Turn the rotor
    pub fn turn(&mut self) {
        self.position.increment();
    }

    pub fn position(&self) -> Letter {
        self.position
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RotorType {
    I,
    II,
    III,
}

impl TryFrom<char> for RotorType {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self> {
        let result = match value {
            '1' => Self::I,
            '2' => Self::II,
            '3' => Self::III,
            _ => bail!("invalid rotor type {value}, expecting 1/2/3"),
        };
        Ok(result)
    }
}

impl std::fmt::Display for RotorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::I => "I",
            Self::II => "II",
            Self::III => "III",
        };
        write!(f, "{name}")
    }
}

static ROTOR_I: OnceLock<Permutation> = OnceLock::new();
static ROTOR_II: OnceLock<Permutation> = OnceLock::new();
static ROTOR_III: OnceLock<Permutation> = OnceLock::new();

static INV_I: OnceLock<Permutation> = OnceLock::new();
static INV_II: OnceLock<Permutation> = OnceLock::new();
static INV_III: OnceLock<Permutation> = OnceLock::new();

impl RotorType {
    fn permutation(self) -> &'static Permutation {
        match self {
            Self::I => ROTOR_I.get_or_init(|| "EKMFLGDQVZNTOWYHXUSPAIBRCJ".try_into().unwrap()),
            Self::II => ROTOR_II.get_or_init(|| "AJDKSIRUXBLHWTMCQGZNPYFVOE".try_into().unwrap()),
            Self::III => ROTOR_III.get_or_init(|| "BDFHJLCPRTXVZNYEIWGAKMUSQO".try_into().unwrap()),
        }
    }

    fn permutation_inv(self) -> &'static Permutation {
        match self {
            Self::I => INV_I.get_or_init(|| self.permutation().inv()),
            Self::II => INV_II.get_or_init(|| self.permutation().inv()),
            Self::III => INV_III.get_or_init(|| self.permutation().inv()),
        }
    }

    fn turnover_notch(self) -> Letter {
        match self {
            Self::I => 'Q',
            Self::II => 'E',
            Self::III => 'V',
        }
        .try_into()
        .unwrap()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RotorSetting {
    pub rotor_type: RotorType,
    pub ring_setting: Letter,
    pub initial_position: Letter,
}

impl std::fmt::Display for RotorSetting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "type {}, ring setting {}, initial position {}",
            self.rotor_type, self.ring_setting, self.initial_position
        )
    }
}

#[derive(Clone)]
pub struct Reflector(Permutation);

static REFLECTOR: OnceLock<Reflector> = OnceLock::new();

impl Reflector {
    pub fn get() -> &'static Reflector {
        REFLECTOR.get_or_init(|| Self("YRUHQSLDPXNGOKMIEBFZCWVJAT".try_into().unwrap()))
    }

    pub fn translate(&self, input: Letter) -> Letter {
        self.0[input]
    }
}

#[derive(Clone)]
pub struct Plugboard(Permutation);

impl Plugboard {
    pub fn new<T>(pairs: T) -> Result<Self>
    where
        T: IntoIterator<Item = (Letter, Letter)>,
    {
        let mut permutation = Permutation::identity();

        for (x, y) in pairs {
            if permutation[x] != x {
                bail!("duplicate plugboard setting for letter {x}");
            }
            if permutation[y] != y {
                bail!("duplicate plugboard setting for letter {y}");
            }
            permutation[x] = y;
            permutation[y] = x;
            info!("plugboard pair: ({x}, {y})");
        }

        Ok(Self(permutation))
    }

    pub fn translate(&self, input: Letter) -> Letter {
        self.0[input]
    }
}
