type Position = String;
type Target = String;
type Resource = String;

use crc::crc32;
use md5;

pub trait Hasher {
    // fn hash(self, value: Union[Resource, Target]) -> Position:
    fn hash(&self, value: Resource) -> Position;
}

struct Md5Hasher {}
impl Hasher for Md5Hasher {
    fn hash(&self, value: Resource) -> Position {
        let digest = md5::compute(value);
        return format!("{:x}", digest);
    }
}

struct Crc32Hasher {}
impl Hasher for Crc32Hasher {
    fn hash(&self, value: Resource) -> Position {
        let digest = crc32::checksum_ieee(value.as_bytes());
        return format!("{}", digest);
    }
}

#[cfg(test)]
struct MockHasher {
    _value: String,
}
#[cfg(test)]
impl Hasher for MockHasher {
    fn hash(&self, _value: Resource) -> Position {
        return self._value.clone();
    }
}
#[cfg(test)]
impl MockHasher {
    fn new() -> MockHasher {
        return MockHasher {_value: String::from("x")};
    }
    fn set_hash_value(&mut self, value: u32) {
        self._value = format!("{}", value);
    }
}

/**
 * Ensure that the hashers give the same values as the original code
 */
#[cfg(test)]
mod hasher_tests {
    #[cfg(test)]
    use crate::{Crc32Hasher, Hasher, Md5Hasher};

    #[test]
    fn crc32() {
        let hasher = Crc32Hasher {};
        assert_eq!(hasher.hash(String::from("test")), "3632233996");
        assert_eq!(hasher.hash(String::from("test")), "3632233996");
        assert_eq!(hasher.hash(String::from("different")), "1812431075");
    }

    #[test]
    fn md5() {
        let hasher = Md5Hasher {};
        assert_eq!(
            hasher.hash(String::from("test")),
            "098f6bcd4621d373cade4e832627b4f6"
        );
        assert_eq!(
            hasher.hash(String::from("test")),
            "098f6bcd4621d373cade4e832627b4f6"
        );
        assert_eq!(
            hasher.hash(String::from("different")),
            "29e4b66fa8076de4d7a26c727b8dbdfa"
        );
    }
}

// ====================================================================

use std::collections::{BTreeMap, HashMap};

pub struct Flexihash<'a> {
    replicas: u32,
    hasher: &'a dyn Hasher,
    position_to_target: BTreeMap<Position, Target>,
    target_to_positions: HashMap<Target, Vec<Position>>,
}
impl<'a> Flexihash<'a> {
    pub fn new(hasher: Option<&'a dyn Hasher>, replicas: Option<u32>) -> Flexihash {
        return Flexihash {
            hasher: hasher.unwrap_or(&Crc32Hasher {}),
            replicas: replicas.unwrap_or(64),
            position_to_target: BTreeMap::new(),
            target_to_positions: HashMap::new(),
        };
    }

    pub fn add_target<S: Into<String>>(&mut self, target: S, weight: u32) -> &Flexihash {
        let target = target.into();
        if self.target_to_positions.contains_key(&target) {
            panic!("Target {} already exists", target);
        }
        let mut positions = Vec::new();
        for i in 0..self.replicas * weight {
            let t = target.clone();
            let sub_target = format!("{}{}", t, i);
            let position = self.hasher.hash(sub_target);
            positions.push(position.clone());
            self.position_to_target
                .insert(position.clone(), target.clone());
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

        let resource_position = self.hasher.hash(resource);

        // let positions: Vec<&Position> = self.position_to_target.keys().cloned().collect();
        // let offset = positions.binary_search(resource_position).expect("Didn't find position");
        // let offset = bisect.bisect_left(ptts, (resource_position, ""));
        let n_targets = self.target_to_positions.len();

        let mut results: Vec<Target> = Vec::new();
        for (position, target) in self.position_to_target.iter() {
            if *position > resource_position {
                if !results.contains(target) {
                    results.push(target.clone());
                    if results.len() == requested_count as usize || results.len() == n_targets {
                        return results;
                    }
                }
            }
        }
        for (_position, target) in self.position_to_target.iter() {
            if !results.contains(target) {
                results.push(target.clone());
                if results.len() == requested_count as usize || results.len() == n_targets {
                    return results;
                }
            }
        }
        return results;
    }
}

#[cfg(test)]
mod compat_tests {
    #[cfg(test)]
    use crate::Flexihash;

    #[test]
    fn same_results_as_original() {
        let mut fh = Flexihash::new(None, None);

        fh.add_targets(vec![
            String::from("a"),
            String::from("b"),
            String::from("c"),
            String::from("d"),
        ]);
        fh.remove_target(String::from("d"));

        assert_eq!(fh.lookup(String::from("1")), "a");
        assert_eq!(fh.lookup(String::from("2")), "b");
        assert_eq!(fh.lookup(String::from("3")), "a");
    }
}

#[cfg(test)]
mod flexihash_tests {
    #[cfg(test)]
    use crate::{Flexihash, MockHasher};

    #[test]
    #[should_panic(expected = "No targets set")]
    fn lookup_throws_exception_on_empty() {
        let fh = Flexihash::new(None, None);
        fh.lookup("test");
    }

    #[test]
    #[should_panic(expected = "Need to request at least 1 resource")]
    fn lookup_list_throws_exception_on_zero() {
        let fh = Flexihash::new(None, None);
        fh.lookup_list("test", 0);
    }

    #[test]
    fn lookup_list_returns_with_short_list_if_all_targets_used() {
        let mut fh = Flexihash::new(None, None);
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
    fn get_all_targets_empty() {
        let fh = Flexihash::new(None, None);
        assert_eq!(fh.get_all_targets().len(), 0);
    }

    #[test]
    #[should_panic]
    fn add_target_throws_exception_on_duplicate_target() {
        let mut fh = Flexihash::new(None, None);
        fh.add_target("t-a", 1);
        fh.add_target("t-a", 1);
    }

    #[test]
    fn add_target_and_get_all_targets() {
        let mut fh = Flexihash::new(None, None);
        fh.add_target("t-a", 1);
        fh.add_target("t-b", 1);
        fh.add_target("t-c", 1);

        assert_eq!(fh.get_all_targets(), ["t-a", "t-b", "t-c"]);
    }

    #[test]
    fn add_targets_and_get_all_targets() {
        let targets = vec!["t-a", "t-b", "t-c"];

        let mut fh = Flexihash::new(None, None);
        fh.add_targets(targets.clone());
        assert_eq!(fh.get_all_targets(), targets);
    }

    #[test]
    fn remove_target() {
        let mut fh = Flexihash::new(None, None);
        fh.add_target("t-a", 1);
        fh.add_target("t-b", 1);
        fh.add_target("t-c", 1);
        fh.remove_target("t-b");

        assert_eq!(fh.get_all_targets(), ["t-a", "t-c"]);
    }

    #[test]
    #[should_panic(expected = "Target 'not-there' does not exist")]
    fn remove_target_fails_on_missing_target() {
        let mut fh = Flexihash::new(None, None);
        fh.remove_target("not-there");
    }

    #[test]
    fn hash_space_repeatable_lookups() {
        let mut fh = Flexihash::new(None, None);
        for i in 1..10 {
            fh.add_target(format!("target{}", i), 1);
        }
        assert_eq!(fh.lookup("t1"), fh.lookup("t1"));
        assert_eq!(fh.lookup("t2"), fh.lookup("t2"));
    }

    /*
    #[test]
    fn hash_space_lookups_are_valid_targets() {
        targets = ["target" + str(i) for i in range(1, 10)];

        let mut fh = Flexihash::new(None, None);
        fh.add_targets(targets);

        for i in 1..10 {
            self.assertTrue(
                fh.lookup("r" + str(i)) in targets,
            "target must be in list of targets",
            )
        }
    }
    */

    #[test]
    fn hash_space_consistent_lookups_after_adding_and_removing() {
        let mut fh = Flexihash::new(None, None);
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
        let mut fh1 = Flexihash::new(None, None);
        for i in 1..10 {
            fh1.add_target(format!("target{}", i), 1);
        }
        let mut results1 = Vec::new();
        for i in 1..100 {
            results1.push(fh1.lookup(format!("t{}", i)));
        }
        let mut fh2 = Flexihash::new(None, None);
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
        let mut fh = Flexihash::new(None, None);
        for i in 1..10 {
            fh.add_target(format!("target{}", i), 1);
        }
        let targets = fh.lookup_list("resource", 2);

        assert_eq!(targets.len(), 2);
        assert_ne!(targets[0], targets[1]);
    }

    #[test]
    fn get_multiple_targets_with_only_one_target() {
        let mut fh = Flexihash::new(None, None);
        fh.add_target("single-target", 1);

        let targets = fh.lookup_list("resource", 2);

        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0], "single-target");
    }

    #[test]
    fn get_more_targets_than_exist() {
        let mut fh = Flexihash::new(None, None);
        fh.add_target("target1", 1);
        fh.add_target("target2", 1);

        let targets = fh.lookup_list("resource", 4);

        assert_eq!(targets.len(), 2);
        assert_ne!(targets[0], targets[1]);
    }

    #[test]
    fn get_multiple_targets_needing_to_loop_to_start() {
        let mut mock_hasher = MockHasher::new();
        mock_hasher.set_hash_value(10);
    }

    /*
    #[test]
    fn get_multiple_targets_needing_to_loop_to_start() {
        let mut mock_hasher = MockHasher::new();
        let mut fh = Flexihash::new(Some(&mock_hasher), Some(1));

        mock_hasher.set_hash_value(10);
        fh.add_target("t1", 1);

        mock_hasher.set_hash_value(20);
        fh.add_target("t2", 1);

        mock_hasher.set_hash_value(30);
        fh.add_target("t3", 1);

        mock_hasher.set_hash_value(40);
        fh.add_target("t4", 1);

        mock_hasher.set_hash_value(50);
        fh.add_target("t5", 1);

        mock_hasher.set_hash_value(35);
        let targets = fh.lookup_list("resource", 4);

        assert_eq!(targets, ["t4", "t5", "t1", "t2"]);
    }

    #[test]
    fn get_multiple_targets_without_getting_any_before_loop_to_start() {
        let mut mock_hasher = MockHasher::new();
        let mut fh = Flexihash::new(Some(&mock_hasher), Some(1));

        mock_hasher.set_hash_value(10);
        fh.add_target("t1", 1);

        mock_hasher.set_hash_value(20);
        fh.add_target("t2", 1);

        mock_hasher.set_hash_value(30);
        fh.add_target("t3", 1);

        mock_hasher.set_hash_value(100);
        let targets = fh.lookup_list("resource", 2);

        assert_eq!(targets, ["t1", "t2"]);
    }

    #[test]
    fn get_multiple_targets_without_needing_to_loop_to_start() {
        let mut mock_hasher = MockHasher::new();
        let mut fh = Flexihash::new(Some(&mock_hasher), Some(1));

        mock_hasher.set_hash_value(10);
        fh.add_target("t1", 1);

        mock_hasher.set_hash_value(20);
        fh.add_target("t2", 1);

        mock_hasher.set_hash_value(30);
        fh.add_target("t3", 1);

        mock_hasher.set_hash_value(15);
        let targets = fh.lookup_list("resource", 2);

        assert_eq!(targets, ["t2", "t3"]);
    }

    #[test]
    fn fallback_precedence_when_server_removed() {
        let mut mock_hasher = MockHasher::new();
        let mut fh = Flexihash::new(Some(&mock_hasher), Some(1));

        mock_hasher.set_hash_value(10);
        fh.add_target("t1", 1);

        mock_hasher.set_hash_value(20);
        fh.add_target("t2", 1);

        mock_hasher.set_hash_value(30);
        fh.add_target("t3", 1);

        mock_hasher.set_hash_value(15);

        assert_eq!(fh.lookup("resource"), "t2");
        assert_eq!(fh.lookup_list("resource", 3), ["t2", "t3", "t1"]);

        fh.remove_target("t2");

        assert_eq!(fh.lookup("resource"), "t3");
        assert_eq!(fh.lookup_list("resource", 3), ["t3", "t1"]);

        fh.remove_target("t3");

        assert_eq!(fh.lookup("resource"), "t1");
        assert_eq!(fh.lookup_list("resource", 3), ["t1"]);
    }
    */
}
