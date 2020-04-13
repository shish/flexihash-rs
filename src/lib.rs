// #![feature(test)]
use crc::crc32;
use md5;
use std::collections::{BTreeMap, HashMap};

pub type Position = u128;
pub type Target = String;
pub type Resource = String;

#[derive(Debug)]
pub enum Hasher {
    Crc32,
    Md5,
    Mock(String),
}

pub fn hash<S: Into<String>>(hasher: &Hasher, value: S) -> Position {
    let value = value.into();
    return match hasher {
        Hasher::Crc32 => crc32::checksum_ieee(value.as_bytes()) as u128,
        Hasher::Md5 => u128::from_be_bytes(md5::compute(value).0),
        Hasher::Mock(val) => u128::from_str_radix(val, 10).unwrap(),
    };
}

#[cfg(test)]
mod test_hashers {
    use super::*;

    #[test]
    fn test_md5() {
        assert_eq!(
            hash(&Hasher::Md5, "test"),
            u128::from_str_radix("098f6bcd4621d373cade4e832627b4f6", 16).unwrap()
        );
        assert_eq!(
            hash(&Hasher::Md5, "test"),
            u128::from_str_radix("098f6bcd4621d373cade4e832627b4f6", 16).unwrap()
        );
        assert_eq!(
            hash(&Hasher::Md5, "different"),
            u128::from_str_radix("29e4b66fa8076de4d7a26c727b8dbdfa", 16).unwrap()
        );
    }

    #[test]
    fn test_crc32() {
        assert_eq!(hash(&Hasher::Crc32, String::from("test")), 3632233996);
        assert_eq!(hash(&Hasher::Crc32, String::from("test")), 3632233996);
        assert_eq!(
            hash(&Hasher::Crc32, String::from("different")),
            1812431075
        );
    }
}

/*
#[cfg(test)]
mod hasher_benchmarks {
    extern crate test;
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_crc32(b: &mut Bencher) {
        b.iter(|| hash(&Hasher::Crc32, String::from("test")));
    }

    #[bench]
    fn bench_md5(b: &mut Bencher) {
        b.iter(|| hash(&Hasher::Md5, String::from("test")));
    }
}
*/

#[derive(Debug)]
pub struct Flexihash {
    replicas: u32,
    hasher: Hasher,
    position_to_target: BTreeMap<Position, Target>,
    sorted_position_to_target: Vec<(Position, Target)>,
    target_to_positions: HashMap<Target, Vec<Position>>,
}

/*
 * Basic methods
 */
impl Flexihash {
    pub fn new() -> Flexihash {
        return Flexihash {
            hasher: Hasher::Crc32,
            replicas: 64,
            position_to_target: BTreeMap::new(),
            sorted_position_to_target: Vec::new(),
            target_to_positions: HashMap::new(),
        };
    }

    pub fn set_hasher(&mut self, hasher: Hasher) {
        self.hasher = hasher;
    }

    pub fn set_replicas(&mut self, replicas: u32) {
        self.replicas = replicas;
    }
}

/*
 * Formatting
 */
use std::fmt;

impl fmt::Display for Flexihash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Flexihash({:?})", self.target_to_positions.keys())
    }
}

#[cfg(test)]
mod test_formatting {
    use super::*;

    #[test]
    fn to_string() {
        let mut fh = Flexihash::new();
        fh.add_target("foo", 2);
        fh.add_target("bar", 4);
        assert_eq!(fh.to_string(), "Flexihash([\"foo\", \"bar\"])");
    }

    #[test]
    fn debug() {
        let mut fh = Flexihash::new();
        fh.add_target("foo", 2);
        fh.add_target("bar", 4);
        assert_eq!(format!("{:?}", fh).to_string().len() > 10, true);
    }
}

/*
 * Add / remove targets
 */
impl Flexihash {
    pub fn add_target<S: Into<String>>(&mut self, target: S, weight: u32) -> &Flexihash {
        let target = target.into();
        if self.target_to_positions.contains_key(&target) {
            panic!("Target {} already exists", target);
        }
        let mut positions = Vec::new();
        for i in 0..self.replicas * weight {
            let t = target.clone();
            let sub_target = format!("{}{}", t, i);
            let position = hash(&self.hasher, sub_target);
            positions.push(position.clone());
            self.position_to_target
                .insert(position.clone(), target.clone());
        }
        self.sorted_position_to_target = Vec::with_capacity(self.position_to_target.len());
        for (k, v) in self.position_to_target.iter() {
            self.sorted_position_to_target.push((k.clone(), v.clone()));
        }
        self.target_to_positions.insert(target.clone(), positions);
        return self;
    }

    pub fn add_targets<S: Into<String>>(&mut self, targets: Vec<S>) -> &Flexihash {
        for target in targets {
            self.add_target(target, 1);
        }
        return self;
    }

    pub fn remove_target<S: Into<String>>(&mut self, target: S) -> &Flexihash {
        let target = target.into();
        if let Some(position_list) = self.target_to_positions.get(target.as_str()) {
            for position in position_list {
                self.position_to_target.remove(position);
            }
            self.sorted_position_to_target = Vec::new();
            for (k, v) in self.position_to_target.iter() {
                self.sorted_position_to_target.push((k.clone(), v.clone()));
            }
            self.target_to_positions.remove(target.as_str());
        } else {
            panic!("Target '{}' does not exist", target);
        }

        return self;
    }

    pub fn get_all_targets(&self) -> Vec<Target> {
        let mut targets = Vec::new();
        for (k, _) in self.target_to_positions.iter() {
            targets.push(k.clone());
        }
        targets.sort();
        return targets;
    }
}

#[cfg(test)]
mod test_add_remove {
    use super::*;

    #[test]
    fn get_all_targets_empty() {
        let fh = Flexihash::new();
        assert_eq!(fh.get_all_targets().len(), 0);
    }

    #[test]
    #[should_panic]
    fn add_target_throws_exception_on_duplicate_target() {
        let mut fh = Flexihash::new();
        fh.add_target("t-a", 1);
        fh.add_target("t-a", 1);
    }

    #[test]
    fn add_target_and_get_all_targets() {
        let mut fh = Flexihash::new();
        fh.add_target("t-a", 1);
        fh.add_target("t-b", 1);
        fh.add_target("t-c", 1);

        assert_eq!(fh.get_all_targets(), ["t-a", "t-b", "t-c"]);
    }

    #[test]
    fn add_targets_and_get_all_targets() {
        let targets = vec!["t-a", "t-b", "t-c"];

        let mut fh = Flexihash::new();
        fh.add_targets(targets.clone());
        assert_eq!(fh.get_all_targets(), targets);
    }

    #[test]
    fn remove_target() {
        let mut fh = Flexihash::new();
        fh.add_target("t-a", 1);
        fh.add_target("t-b", 1);
        fh.add_target("t-c", 1);
        fh.remove_target("t-b");

        assert_eq!(fh.get_all_targets(), ["t-a", "t-c"]);
    }

    #[test]
    #[should_panic(expected = "Target 'not-there' does not exist")]
    fn remove_target_fails_on_missing_target() {
        let mut fh = Flexihash::new();
        fh.remove_target("not-there");
    }
}

/*
 * Lookups
 */
impl Flexihash {
    pub fn lookup<S: Into<String>>(&self, resource: S) -> Target {
        let targets = self.lookup_list(resource, 1);
        if let Some(target) = targets.get(0) {
            return target.clone();
        } else {
            panic!("No targets set");
        }
    }

    pub fn lookup_list<S: Into<String>>(&self, resource: S, requested_count: u32) -> Vec<Target> {
        let resource = resource.into();
        if requested_count == 0 {
            panic!("Need to request at least 1 resource");
        }
        if self.target_to_positions.len() == 0 {
            return Vec::new();
        }
        if self.target_to_positions.len() == 1 {
            // if only one item, return first entry
            for (k, _) in self.target_to_positions.iter() {
                return vec![k.clone()];
            }
        }

        let resource_position = hash(&self.hasher, resource);
        let n_targets = self.target_to_positions.len();

        let mut results: Vec<Target> = Vec::new();
        let s = String::new();
        let offset = match self
            .sorted_position_to_target
            .binary_search(&(resource_position, s))
        {
            Ok(pos) => pos,
            Err(pos) => pos,
        };
        for i in (offset..self.sorted_position_to_target.len()).chain(0..offset) {
            if let Some((_, target)) = self.sorted_position_to_target.get(i) {
                if !results.contains(target) {
                    results.push(target.clone());
                    if results.len() == requested_count as usize || results.len() == n_targets {
                        return results;
                    }
                }
            }
        }
        return results;
    }
}

/**
 * Ensure the Flexihash class gives the same results as the original code
 */
#[cfg(test)]
mod test_compat {
    #[cfg(test)]
    use crate::Flexihash;
    use md5;
    use std::collections::HashMap;

    #[test]
    fn same_results_as_original() {
        // generate lots of test cases, and use md5 to be "random" in case
        // a fast hasher (eg crc) doesn't distribute data well
        let mut fh = Flexihash::new();
        let mut results = HashMap::new();

        for n in vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"].iter() {
            let target = format!("{:032x}", md5::compute(n.to_string()));
            fh.add_target(target.clone(), 1);
            results.insert(target, 0);
        }

        for n in 0..1000 {
            let target = format!("{:032x}", md5::compute(n.to_string()));
            let position = fh.lookup(target);
            match results.get_mut(&position) {
                Some(v) => {
                    *v += 1;
                }
                None => {
                    results.insert(position.clone(), 1);
                }
            };
        }

        let mut expected = HashMap::new();
        expected.insert("0cc175b9c0f1b6a831c399e269772661".to_string(), 105);
        expected.insert("2510c39011c5be704182423e3a695e91".to_string(), 54);
        expected.insert("363b122c528f54df4a0446b6bab05515".to_string(), 113);
        expected.insert("4a8a08f09d37b73795649038408b5f33".to_string(), 119);
        expected.insert("8277e0910d750195b448797616e091ad".to_string(), 168);
        expected.insert("865c0c0b4ab0e063e5caa3387c1a8741".to_string(), 74);
        expected.insert("8fa14cdd754f91cc6554c9e71929cce7".to_string(), 94);
        expected.insert("92eb5ffee6ae2fec3ad71c777531578f".to_string(), 63);
        expected.insert("b2f5ff47436671b6e533d8dc3614845d".to_string(), 124);
        expected.insert("e1671797c52e15f763380b45e841ec32".to_string(), 86);

        assert_eq!(results, expected)
    }
}

#[cfg(test)]
mod test_lookups {
    use super::*;

    #[test]
    #[should_panic(expected = "No targets set")]
    fn lookup_throws_exception_on_empty() {
        let fh = Flexihash::new();
        fh.lookup("test");
    }

    #[test]
    #[should_panic(expected = "Need to request at least 1 resource")]
    fn lookup_list_throws_exception_on_zero() {
        let fh = Flexihash::new();
        fh.lookup_list("test", 0);
    }

    #[test]
    fn lookup_list_returns_with_short_list_if_all_targets_used() {
        let mut fh = Flexihash::new();
        // both have CRC32 of 1253617450
        fh.add_target("x", 1);
        fh.add_target("y", 1); // make the list non-empty, non-one-value, to avoid shortcuts
        fh.add_target("80726", 1); // add a value
        fh.add_target("14746907", 1); // add a different value with the same hash, to clobber the first
        fh.remove_target("14746907"); // remove the fourth value; with the third clobbered, only X and Y are left
        let result = fh.lookup_list("test", 3); // try to get 3 results, our target list is X, Y, 80726
        assert_eq!(result.len(), 2); // but 80726 isn't reachable since it was clobbered
        assert_eq!(result.contains(&String::from("x")), true); // all that's left is x
        assert_eq!(result.contains(&String::from("y")), true); // and y
    }

    #[test]
    fn hash_space_repeatable_lookups() {
        let mut fh = Flexihash::new();
        for i in 1..10 {
            fh.add_target(format!("target{}", i), 1);
        }
        assert_eq!(fh.lookup("t1"), fh.lookup("t1"));
        assert_eq!(fh.lookup("t2"), fh.lookup("t2"));
    }

    #[test]
    fn hash_space_lookups_are_valid_targets() {
        let mut fh = Flexihash::new();
        let mut targets = Vec::new();
        for i in 1..10 {
            targets.push(format!("targets{}", i));
        }
        fh.add_targets(targets.clone());

        for i in 1..10 {
            assert_eq!(targets.contains(&fh.lookup(format!("r{}", i))), true)
        }
    }

    #[test]
    fn hash_space_consistent_lookups_after_adding_and_removing() {
        let mut fh = Flexihash::new();
        for i in 1..10 {
            fh.add_target(format!("target{}", i), 1);
        }
        let mut results1 = Vec::new();
        for i in 1..100 {
            results1.push(fh.lookup(format!("t{}", i)));
        }
        fh.add_target("new-target", 1);
        fh.remove_target("new-target");
        fh.add_target("new-target", 1);
        fh.remove_target("new-target");

        let mut results2 = Vec::new();
        for i in 1..100 {
            results2.push(fh.lookup(format!("t{}", i)));
        }
        // This is probably optimistic, as adding/removing a target may
        // clobber existing targets and is not expected to restore them.
        assert_eq!(results1, results2);
    }

    #[test]
    fn hash_space_consistent_lookups_with_new_instance() {
        let mut fh1 = Flexihash::new();
        for i in 1..10 {
            fh1.add_target(format!("target{}", i), 1);
        }
        let mut results1 = Vec::new();
        for i in 1..100 {
            results1.push(fh1.lookup(format!("t{}", i)));
        }
        let mut fh2 = Flexihash::new();
        for i in 1..10 {
            fh2.add_target(format!("target{}", i), 1);
        }
        let mut results2 = Vec::new();
        for i in 1..100 {
            results2.push(fh2.lookup(format!("t{}", i)));
        }
        assert_eq!(results1, results2);
    }

    #[test]
    fn get_multiple_targets() {
        let mut fh = Flexihash::new();
        for i in 1..10 {
            fh.add_target(format!("target{}", i), 1);
        }
        let targets = fh.lookup_list("resource", 2);

        assert_eq!(targets.len(), 2);
        assert_ne!(targets[0], targets[1]);
    }

    #[test]
    fn get_multiple_targets_with_only_one_target() {
        let mut fh = Flexihash::new();
        fh.add_target("single-target", 1);

        let targets = fh.lookup_list("resource", 2);

        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0], "single-target");
    }

    #[test]
    fn get_more_targets_than_exist() {
        let mut fh = Flexihash::new();
        fh.add_target("target1", 1);
        fh.add_target("target2", 1);

        let targets = fh.lookup_list("resource", 4);

        assert_eq!(targets.len(), 2);
        assert_ne!(targets[0], targets[1]);
    }

    #[test]
    fn get_multiple_targets_needing_to_loop_to_start() {
        let mut fh = Flexihash::new();
        fh.set_replicas(1);

        fh.set_hasher(Hasher::Mock("10".to_string()));
        fh.add_target("t1", 1);

        fh.set_hasher(Hasher::Mock("20".to_string()));
        fh.add_target("t2", 1);

        fh.set_hasher(Hasher::Mock("30".to_string()));
        fh.add_target("t3", 1);

        fh.set_hasher(Hasher::Mock("40".to_string()));
        fh.add_target("t4", 1);

        fh.set_hasher(Hasher::Mock("50".to_string()));
        fh.add_target("t5", 1);

        fh.set_hasher(Hasher::Mock("35".to_string()));
        let targets = fh.lookup_list("resource", 4);

        assert_eq!(targets, ["t4", "t5", "t1", "t2"]);
    }

    #[test]
    fn get_multiple_targets_without_getting_any_before_loop_to_start() {
        let mut fh = Flexihash::new();
        fh.set_replicas(1);

        fh.set_hasher(Hasher::Mock("10".to_string()));
        fh.add_target("t1", 1);

        fh.set_hasher(Hasher::Mock("20".to_string()));
        fh.add_target("t2", 1);

        fh.set_hasher(Hasher::Mock("30".to_string()));
        fh.add_target("t3", 1);

        fh.set_hasher(Hasher::Mock("99".to_string()));
        let targets = fh.lookup_list("resource", 2);

        assert_eq!(targets, ["t1", "t2"]);
    }

    #[test]
    fn get_multiple_targets_without_needing_to_loop_to_start() {
        let mut fh = Flexihash::new();
        fh.set_replicas(1);

        fh.set_hasher(Hasher::Mock("10".to_string()));
        fh.add_target("t1", 1);

        fh.set_hasher(Hasher::Mock("20".to_string()));
        fh.add_target("t2", 1);

        fh.set_hasher(Hasher::Mock("30".to_string()));
        fh.add_target("t3", 1);

        fh.set_hasher(Hasher::Mock("15".to_string()));
        let targets = fh.lookup_list("resource", 2);

        assert_eq!(targets, ["t2", "t3"]);
    }

    #[test]
    fn fallback_precedence_when_server_removed() {
        let mut fh = Flexihash::new();
        fh.set_replicas(1);

        fh.set_hasher(Hasher::Mock("10".to_string()));
        fh.add_target("t1", 1);

        fh.set_hasher(Hasher::Mock("20".to_string()));
        fh.add_target("t2", 1);

        fh.set_hasher(Hasher::Mock("30".to_string()));
        fh.add_target("t3", 1);

        fh.set_hasher(Hasher::Mock("15".to_string()));

        assert_eq!(fh.lookup("resource"), "t2");
        assert_eq!(fh.lookup_list("resource", 3), ["t2", "t3", "t1"]);

        fh.remove_target("t2");

        assert_eq!(fh.lookup("resource"), "t3");
        assert_eq!(fh.lookup_list("resource", 3), ["t3", "t1"]);

        fh.remove_target("t3");

        assert_eq!(fh.lookup("resource"), "t1");
        assert_eq!(fh.lookup_list("resource", 3), ["t1"]);
    }
}

/*
extern crate test;

#[cfg(test)]
mod lookup_list_bench {
    use super::*;
    use test::Bencher;

    #[bench]
    fn one_of_one(b: &mut Bencher) {
        let mut fh = Flexihash::new();
        fh.add_target("olive", 10);

        b.iter(|| fh.lookup_list("foobar", 1));
    }

    #[bench]
    fn one_of_two(b: &mut Bencher) {
        let mut fh = Flexihash::new();
        fh.add_target("olive", 10);
        fh.add_target("acacia", 10);

        b.iter(|| fh.lookup_list("foobar", 1));
    }

    #[bench]
    fn two_of_two(b: &mut Bencher) {
        let mut fh = Flexihash::new();
        fh.add_target("olive", 10);
        fh.add_target("acacia", 10);

        b.iter(|| fh.lookup_list("foobar", 2));
    }

    #[bench]
    fn three_of_two(b: &mut Bencher) {
        let mut fh = Flexihash::new();
        fh.add_target("olive", 10);
        fh.add_target("acacia", 10);

        b.iter(|| fh.lookup_list("foobar", 3));
    }
}

#[cfg(test)]
mod flexihash_bench {
    use super::*;
    use test::Bencher;

    #[bench]
    fn init(b: &mut Bencher) {
        b.iter(|| {
            Flexihash::new()
        });
    }
}

#[cfg(test)]
mod add_target_bench {
    use super::*;
    use test::Bencher;

    #[bench]
    fn one(b: &mut Bencher) {
        b.iter(|| {
            let mut fh = Flexihash::new();
            fh.add_target("olive", 10);
        });
    }

    #[bench]
    fn two(b: &mut Bencher) {
        b.iter(|| {
            let mut fh = Flexihash::new();
            fh.add_target("olive", 10);
            fh.add_target("acacia", 10);
        });
    }

    #[bench]
    fn three(b: &mut Bencher) {
        b.iter(|| {
            let mut fh = Flexihash::new();
            fh.add_target("olive", 10);
            fh.add_target("acacia", 10);
            fh.add_target("rose", 10);
        });
    }

    #[bench]
    fn many(b: &mut Bencher) {
        b.iter(|| {
            let mut fh = Flexihash::new();
            for n in 0..10 {
                fh.add_target(format!("olive{}", n), 10);
            }
        });
    }
}
*/
