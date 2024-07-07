use crate::components::{Plugboard, Reflector, Rotor, RotorSetting};
use crate::letter::Letter;
use anyhow::{Context, Result};
use log::debug;

#[derive(Clone)]
pub struct Enigma {
    plugboard: Plugboard,
    rotors: Vec<Rotor>,
    reflector: &'static Reflector,
}

impl Enigma {
    pub fn new<T, U>(plugboard: T, rotors: U) -> Result<Self>
    where
        T: IntoIterator<Item = (Letter, Letter)>,
        U: IntoIterator<Item = RotorSetting>,
    {
        Ok(Self {
            plugboard: Plugboard::new(plugboard)?,
            rotors: rotors.into_iter().map(Rotor::new).collect(),
            reflector: Reflector::get(),
        })
    }

    pub fn translate(&mut self, input: Letter) -> Letter {
        debug!("input: {input}");

        // turn rotors

        let mut should_turn = true;
        let mut iter = self.rotors.iter_mut().rev().peekable();
        while let Some(rotor) = iter.next() {
            let next_should_turn = iter.peek().is_some() && rotor.about_to_turnover();
            if should_turn || next_should_turn {
                rotor.turn();
            }
            should_turn = next_should_turn;
        }

        debug!(
            "current rotor positions: {}",
            self.rotors
                .iter()
                .map(|rotor| char::from(rotor.position()))
                .collect::<String>()
        );

        // translate

        let mut x = self.plugboard.translate(input);
        debug!("plugboard: {x}");

        for (i, rotor) in self.rotors.iter().enumerate().rev() {
            x = rotor.translate_forward(x);
            debug!("rotor {}: {x}", i + 1);
        }

        x = self.reflector.translate(x);
        debug!("reflector: {x}");

        for (i, rotor) in self.rotors.iter().enumerate() {
            x = rotor.translate_backward(x);
            debug!("rotor {}: {x}", i + 1);
        }

        x = self.plugboard.translate(x);
        debug!("plugboard (output): {x}");

        x
    }

    pub fn translate_str(&mut self, input: &str) -> Result<String> {
        let mut result = String::new();
        for x in input.chars() {
            result.push(
                self.translate(x.try_into().context("invalid plaintext")?)
                    .into(),
            );
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::RotorType;

    struct TestCase {
        plugboard: &'static [(char, char)],
        rotors: [RotorType; 3],
        ring_settings: [char; 3],
        initial_positions: [char; 3],
        plaintext: &'static str,
        ciphertext: &'static str,
    }

    const TEST_CASES: &[TestCase] = &[
        TestCase {
            plugboard: &[],
            rotors: [RotorType::I, RotorType::II, RotorType::III],
            ring_settings: ['A', 'A', 'A'],
            initial_positions: ['A', 'A', 'A'],
            plaintext: "ABCDEF",
            ciphertext: "BJELRQ",
        },
        TestCase {
            plugboard: &[],
            rotors: [RotorType::I, RotorType::II, RotorType::III],
            ring_settings: ['A', 'A', 'A'],
            initial_positions: ['A', 'B', 'T'],
            plaintext: "ABCDEF",
            ciphertext: "NYEWVO",
        },
        TestCase {
            plugboard: &[],
            rotors: [RotorType::I, RotorType::II, RotorType::III],
            ring_settings: ['A', 'A', 'A'],
            initial_positions: ['A', 'D', 'U'],
            plaintext: "ABCDEF",
            ciphertext: "EEUNGT",
        },
        TestCase {
            plugboard: &[],
            rotors: [RotorType::I, RotorType::II, RotorType::III],
            ring_settings: ['A', 'B', 'C'],
            initial_positions: ['A', 'A', 'Z'],
            plaintext: "ABCDEF",
            ciphertext: "KUQRAH",
        },
        TestCase {
            plugboard: &[],
            rotors: [RotorType::I, RotorType::II, RotorType::III],
            ring_settings: ['A', 'E', 'S'],
            initial_positions: ['A', 'D', 'U'],
            plaintext: "ABCDEF",
            ciphertext: "BCMJMR",
        },
        TestCase {
            plugboard: &[
                ('B', 'X'),
                ('G', 'K'),
                ('W', 'Y'),
                ('E', 'F'),
                ('P', 'Q'),
                ('S', 'N'),
            ],
            rotors: [RotorType::II, RotorType::III, RotorType::I],
            ring_settings: ['D', 'E', 'S'],
            initial_positions: ['A', 'A', 'A'],
            plaintext: "ABCDEF",
            ciphertext: "SDKZQX",
        },
    ];

    #[test]
    fn it_encrypts_test_cases_correctly() {
        for case in TEST_CASES {
            let rotor_settings: [RotorSetting; 3] = std::array::from_fn(|i| RotorSetting {
                rotor_type: case.rotors[i],
                ring_setting: case.ring_settings[i].try_into().unwrap(),
                initial_position: case.initial_positions[i].try_into().unwrap(),
            });
            let mut enigma = Enigma::new(
                case.plugboard
                    .iter()
                    .map(|&(x, y)| (x.try_into().unwrap(), y.try_into().unwrap())),
                rotor_settings,
            )
            .unwrap();
            let ciphertext = enigma.translate_str(case.plaintext).unwrap();
            println!("{} -> {ciphertext}", case.plaintext);
            assert_eq!(ciphertext, case.ciphertext);
        }
    }
}
