extern crate rand;

use std::num::Wrapping;
use std::time::Instant;
use rand::random;

fn jump(mut key: u64, nbucket: u32) -> u32 {
    assert!(nbucket > 0);
    let (mut b, mut j) = (-1 as i64, 0 as i64);
    
    while j < nbucket as _ {
        b = j;
        key = (Wrapping(2862933555777941757 as u64) * Wrapping(key) + Wrapping(1)).0;
        j = (((b + 1) as f64) * (((1u64 << 31) as f64) / (((key >> 33) + 1) as f64)))
            as i64;
    }

    assert!((b as u32) < nbucket, "{}", b as u32);
    b as u32
}

// select nstripe servers with jump hash for RAID-0 data distribution
fn jump_n(mut key: u64, nbucket: u32, nstripe: u32, distinct: bool) -> Vec<u32> {
    let mut buckets = vec!();
    assert!(nbucket >= nstripe);
    
    for _ in 0..nstripe {
        let b = loop {
            let b = jump(key, nbucket);
            if !distinct || !buckets.contains(&b) {
                break b;
            }
            key += 1;
        };
        buckets.push(b);
    }

    assert_eq!(buckets.len(), nstripe as usize);
    buckets
}

fn parse_arg() -> (u32, u32, u32) {
    let hint = "arg: <nbucket> <nkey> <nstripe>";
    let arg: Vec<u32> = std::env::args().skip(1).map(|arg| arg.parse().expect(&arg)).collect();
    assert!(arg.len() == 3, "{}", hint);
    (arg[0], arg[1], arg[2])
}

fn test_dist(nbucket: u32, nkey: u32, nstripe: u32, distinct: bool) {
    let mut items = vec![0; nbucket as usize];

    let begin = Instant::now();
    for _ in 0..nkey {
        let key = random::<u64>();
        let buckets = jump_n(key, nbucket, nstripe, distinct);

        for bucket in buckets {
            items[bucket as usize] += 1;
        }
    }
    let duration = begin.elapsed();
    
    let (min, max) = (*items.iter().min().unwrap(), *items.iter().max().unwrap());
    let total = (nkey * nstripe) as f64;
    let (min_percent, max_percent) = (min as f64 / total, max as f64 / total);
    println!("  {}: min {:.6}%, max {:.6}%, avg {:.6}%, iops {:.4}k",
        if distinct { "distinct" } else { "random n" },
        min_percent, max_percent, 1 as f64 / nbucket as f64,
        nkey as f64 / 1000 as f64 / duration.as_micros() as f64 * 1000000 as f64);
}

fn test_moved(nbucket: u32, nkey: u32, nstripe: u32, distinct: bool, add_nbucket: u32) {
    let mut moved = 0;
    for _ in 0..nkey {
        let key = random::<u64>();
        let old = jump_n(key, nbucket, nstripe, distinct);
        let new = jump_n(key, nbucket + add_nbucket, nstripe, distinct);

        for i in 0..nstripe as usize {
            if old[i] != new[i] {
                moved += 1;
            }
        }
    }
    println!("  {}: total {}, moved {}, percent {:.2}%",
        if distinct { "distinct" } else { "random n" },
        nkey * nstripe, moved, moved as f64 / (nkey * nstripe) as f64 * 100.0);
}

fn main() {
    let (nbucket, nkey, nstripe) = parse_arg();

    println!("test data distribution.");
    test_dist(nbucket, nkey, nstripe, false);
    test_dist(nbucket, nkey, nstripe, true);
    println!();

    let mut add_nbucket = 1;
    while add_nbucket <= nbucket {
        println!("test data movement when add {} buckets.", add_nbucket);
        test_moved(nbucket, nkey, nstripe, false, add_nbucket);
        test_moved(nbucket, nkey, nstripe, true, add_nbucket);
        add_nbucket *= 2;
    }
}
