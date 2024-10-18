use std::fmt;

use rand::distributions::Standard;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::lookup_table::lut_phf::phf_shared;
use crate::lookup_table::lut_phf::phf_shared::{HashKey, PhfHash};

const DEFAULT_LAMBDA: usize = 5;
const FIXED_SEED: u64 = 1_234_567_890;

pub struct HashState {
    pub key: HashKey,
    pub displacements: Vec<(u32, u32)>,
    pub map: Vec<usize>,
}

#[inline]
pub fn generate_hash<H: PhfHash>(entries: &[H]) -> HashState {
    SmallRng::seed_from_u64(FIXED_SEED)
        .sample_iter(Standard)
        .find_map(|key| try_generate_hash(entries, key))
        .expect("failed to solve PHF")
}

fn try_generate_hash<H: PhfHash>(entries: &[H], key: HashKey) -> Option<HashState> {
    struct Bucket {
        idx: usize,
        keys: Vec<usize>,
    }

    let hashes: Vec<_> = entries.iter().map(|entry| phf_shared::hash(entry, &key)).collect();

    let buckets_len = (hashes.len() + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA;
    let mut buckets = (0..buckets_len)
        .map(|i| Bucket { idx: i, keys: vec![] })
        .collect::<Vec<_>>();

    for (i, hash) in hashes.iter().enumerate() {
        buckets[(hash.g % (buckets_len as u32)) as usize].keys.push(i);
    }

    // Sort descending
    buckets.sort_by(|a, b| a.keys.len().cmp(&b.keys.len()).reverse());

    let table_len = hashes.len();
    let mut map = vec![None; table_len];
    let mut disps = vec![(0_u32, 0_u32); buckets_len];

    // store whether an element from the bucket being placed is
    // located at a certain position, to allow for efficient overlap
    // checks. It works by storing the generation in each cell and
    // each new placement-attempt is a new generation, so you can tell
    // if this is legitimately full by checking that the generations
    // are equal. (A u64 is far too large to overflow in a reasonable
    // time for current hardware.)
    let mut try_map = vec![0_u64; table_len];
    let mut generation = 0_u64;

    // the actual values corresponding to the markers above, as
    // (index, key) pairs, for adding to the main map once we've
    // chosen the right disps.
    let mut values_to_add = vec![];

    'buckets: for bucket in &buckets {
        for d1 in 0..(table_len as u32) {
            'disps: for d2 in 0..(table_len as u32) {
                values_to_add.clear();
                generation += 1;

                for &key in &bucket.keys {
                    let idx =
                        (phf_shared::displace(hashes[key].f1, hashes[key].f2, d1, d2) % (table_len as u32)) as usize;
                    if map[idx].is_some() || try_map[idx] == generation {
                        continue 'disps;
                    }
                    try_map[idx] = generation;
                    values_to_add.push((idx, key));
                }

                // We've picked a good set of disps
                disps[bucket.idx] = (d1, d2);
                for &(idx, key) in &values_to_add {
                    map[idx] = Some(key);
                }
                continue 'buckets;
            }
        }

        // Unable to find displacements for a bucket
        return None;
    }

    Some(HashState {
        key,
        displacements: disps,
        map: map.into_iter().map(|i| i.expect("Fail at collecting map")).collect(),
    })
}

// ##############################
// #      Added by Ricardo      #
// ##############################

impl fmt::Display for HashState {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "HashState {{\n  key: {},\n  displacements: {:?},\n  map: {:?}\n}}",
            self.key, self.displacements, self.map
        )
    }
}

impl HashState {
    #[inline]
    pub fn get_index<T: ?Sized + phf_shared::PhfHash>(&self, key: &T) -> Option<usize> {
        // Calculate the hashes using the provided key and hash function
        let hashes = phf_shared::hash(key, &self.key);

        // Get the index from the displacement map
        let index = phf_shared::get_index(&hashes, &self.displacements, self.map.len()) as usize;

        // Retrieve the value from the map, if the index is within bounds
        (index < self.map.len()).then(|| self.map[index])
    }
}
