type Position = String;
type Target = String;
type Resource = String;

use std::collections::{BTreeMap, HashMap};
use crc::crc32;
use md5;

pub trait Hasher {
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
