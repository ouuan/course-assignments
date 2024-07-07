use crate::letter::{Letter, LETTER_COUNT};
use crate::permutation::Permutation;
use crate::{Enigma, RotorSetting, RotorType};
use anyhow::{Context, Result};
use itertools::Itertools;
use log::info;
use rayon::prelude::*;

fn rings_in_permutation(p: Permutation) -> Vec<u8> {
    let mut result = Vec::new();
    let mut visited = vec![false; LETTER_COUNT as usize];
    for i in Letter::iter_all() {
        if visited[i.index()] {
            continue;
        }
        let mut count = 0;
        let mut x = i;
        loop {
            count += 1;
            visited[x.index()] = true;
            x = p[x];
            if visited[x.index()] {
                break;
            }
        }
        result.push(count);
    }
    result.sort_unstable();
    result
}

fn rings_in_enigma(enigma: Enigma) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let mut p14 = Permutation::default();
    let mut p25 = Permutation::default();
    let mut p36 = Permutation::default();

    for x in Letter::iter_all() {
        let mut enigma = enigma.clone();
        let c = [(); 6].map(|_| enigma.translate(x));
        p14[c[0]] = c[3];
        p25[c[1]] = c[4];
        p36[c[2]] = c[5];
    }

    (
        rings_in_permutation(p14),
        rings_in_permutation(p25),
        rings_in_permutation(p36),
    )
}

pub fn crack(p14: &str, p25: &str, p36: &str) -> Result<Vec<Vec<RotorSetting>>> {
    let r14 = rings_in_permutation(p14.try_into().context("invalid 14 permutation")?);
    let r25 = rings_in_permutation(p25.try_into().context("invalid 25 permutation")?);
    let r36 = rings_in_permutation(p36.try_into().context("invalid 36 permutation")?);

    info!(
        "target ring lengths: 14: {:?} 25: {:?} 36: {:?}",
        &r14, &r25, &r36
    );

    let all_letters = Letter::iter_all().collect::<Vec<_>>();
    let results =
        all_letters
            .par_iter()
            .map(|rs1| {
                let mut max = 0;
                let mut best = Vec::new();
                for ((rs2, rs3), rotor_types) in all_letters
                    .iter()
                    .cartesian_product(all_letters.iter())
                    .cartesian_product(
                        [RotorType::I, RotorType::II, RotorType::III]
                            .into_iter()
                            .permutations(3),
                    )
                {
                    let rotors = rotor_types.iter().zip([rs1, rs2, rs3]).map(
                        |(&rotor_type, &ring_setting)| RotorSetting {
                            rotor_type,
                            initial_position: Letter::default(),
                            ring_setting,
                        },
                    );
                    let enigma = Enigma::new([], rotors.clone()).unwrap();
                    let (e14, e25, e36) = rings_in_enigma(enigma);
                    let count = r14.iter().zip(e14).filter(|(x, y)| *x == y).count()
                        + r25.iter().zip(e25).filter(|(x, y)| *x == y).count()
                        + r36.iter().zip(e36).filter(|(x, y)| *x == y).count();
                    if count > max {
                        max = count;
                        best.clear();
                    }
                    if count == max {
                        best.push(rotors.collect_vec());
                    }
                }

                info!("job finished: max correct ring count {max}");

                (max, best)
            })
            .collect::<Vec<_>>();

    let max = results.iter().map(|(count, _)| *count).max().unwrap();
    info!("global max correct ring count: {max}");

    Ok(results
        .into_iter()
        .filter_map(|(count, best)| if count == max { Some(best) } else { None })
        .flatten()
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_crack_test_case() {
        let results = crack(
            "ELCONWDIAPKSZHFBQTJYRGVXMU",
            "MRWJFDVSQEXUCONHBIPLTGAYZK",
            "WADFRPOLNTVCHMYBJQIGEUSKZX",
        )
        .unwrap();
        for result in &results {
            println!("crack result:");
            for (i, rotor) in result.iter().enumerate() {
                println!("rotor {}: {rotor}", i + 1);
            }
        }
        assert!(results.contains(&vec![
            RotorSetting {
                rotor_type: RotorType::II,
                ring_setting: 'D'.try_into().unwrap(),
                initial_position: Letter::default(),
            },
            RotorSetting {
                rotor_type: RotorType::III,
                ring_setting: 'E'.try_into().unwrap(),
                initial_position: Letter::default(),
            },
            RotorSetting {
                rotor_type: RotorType::I,
                ring_setting: 'S'.try_into().unwrap(),
                initial_position: Letter::default(),
            },
        ]))
    }
}
