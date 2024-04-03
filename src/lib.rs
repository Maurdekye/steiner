use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    iter::once,
    time::{Duration, Instant},
    vec,
};

use progress_observer::Observer;

pub fn permutations<T: Copy>(set: &[T], size: usize) -> Vec<Vec<T>> {
    match size {
        ..=0 => vec![vec![]],
        size => (0..set.len())
            .flat_map(|i| {
                permutations(&set[i + 1..], size - 1)
                    .into_iter()
                    .map(move |permute| once(set[i]).chain(permute).collect())
            })
            .collect(),
    }
}

#[test]
fn permutations_test() {
    let xs: Vec<usize> = (1..=5).collect();
    println!("{:?}", permutations(&xs, 1));
}

fn larger_sets(
    starting_set: Vec<usize>,
    available_digits: Vec<usize>,
    target_size: usize,
) -> Vec<Vec<usize>> {
    if starting_set.len() == target_size {
        vec![starting_set]
    } else {
        (0..available_digits.len())
            .flat_map(|i| {
                let mut available_digits = available_digits.clone();
                let new_digit = available_digits.remove(i);
                let mut new_set = starting_set.clone();
                new_set.insert(
                    new_set.binary_search(&new_digit).map_or_else(|x| x, |x| x),
                    new_digit,
                );
                larger_sets(new_set, available_digits, target_size)
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
struct SteinerSearch {
    set: Vec<Vec<usize>>,
    unused_permutations: Vec<Vec<usize>>,
    used_permutations: Vec<Vec<usize>>,
}

pub struct SteinerSearcher {
    t: usize,
    k: usize,
    n: usize,
    fringe: Vec<SteinerSearch>,
    set_permutations: HashMap<Vec<usize>, Vec<Vec<usize>>>,
    dead_ends: HashSet<Vec<Vec<usize>>>,
    explored: HashSet<Vec<Vec<usize>>>,
    observer: Observer,
    i: usize,
}

impl SteinerSearcher {
    pub fn new(t: usize, k: usize, n: usize) -> Self {
        assert!(t < k && k < n, "t, k, n must follow t < k < n");

        let mut fringe: Vec<SteinerSearch> = Vec::new();
        fringe.push(SteinerSearch {
            set: Vec::new(),
            unused_permutations: permutations(&(1..=n).collect::<Vec<_>>(), t),
            used_permutations: Vec::new(),
        });
        Self {
            t,
            k,
            n,
            fringe,
            set_permutations: HashMap::new(),
            dead_ends: HashSet::new(),
            explored: HashSet::new(),
            observer: Observer::new(Duration::from_secs(1)),
            i: 0,
        }
    }

    pub fn get_item_permutations(&mut self, set: Vec<usize>) -> Vec<Vec<usize>> {
        match self.set_permutations.entry(set) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => {
                let permutes = permutations(entry.key(), self.t);
                entry.insert(permutes).clone()
            }
        }
    }
}

impl Iterator for SteinerSearcher {
    type Item = Vec<Vec<usize>>;

    fn next(&mut self) -> Option<Self::Item> {
        let Self { k, n, .. } = *self;
        while let Some(steiner_set) = self.fringe.pop() {
            self.i += 1;
            if !self.explored.insert(steiner_set.set.clone()) {
                continue;
            }
            if steiner_set.unused_permutations.is_empty() {
                return Some(steiner_set.set);
            }
            let mut new_fringe = (0..steiner_set.unused_permutations.len())
                .flat_map(|i| {
                    let mut steiner_set = steiner_set.clone();
                    let permutation = steiner_set.unused_permutations.remove(i);
                    let available_digits = (1..=n).filter(|x| !permutation.contains(x)).collect();

                    let mut new_sets = Vec::new();
                    for larger_set in larger_sets(permutation, available_digits, k) {
                        let larger_set_permutations =
                            self.get_item_permutations(larger_set.clone());
                        if steiner_set
                            .used_permutations
                            .iter()
                            .any(|permutation| larger_set_permutations.contains(&permutation))
                        {
                            continue;
                        }
                        let mut steiner_set = steiner_set.clone();
                        steiner_set.set.insert(
                            steiner_set
                                .set
                                .binary_search(&larger_set)
                                .map_or_else(|x| x, |x| x),
                            larger_set,
                        );
                        if self.dead_ends.contains(&steiner_set.set)
                            || self.explored.contains(&steiner_set.set)
                        {
                            continue;
                        }
                        steiner_set
                            .unused_permutations
                            .retain(|p| !larger_set_permutations.contains(p));
                        steiner_set
                            .used_permutations
                            .extend(larger_set_permutations);
                        new_sets.push(steiner_set)
                    }
                    new_sets
                })
                .collect::<Vec<_>>();
            if self.observer.tick() {
                println!(
                    "i: {}, fringe: {}, dead ends: {}, explored: {}, new fringe: {}",
                    self.i,
                    self.fringe.len(),
                    self.dead_ends.len(),
                    self.explored.len(),
                    new_fringe.len()
                );
            }
            if new_fringe.is_empty() {
                self.dead_ends.insert(steiner_set.set);
            } else {
                new_fringe.sort_by(|a, b| a.set.cmp(&b.set));
                new_fringe.dedup_by(|a, b| a.set == b.set);
                self.fringe.extend(new_fringe);
            }
        }
        None
    }
}

#[test]
fn steiner_test() {
    let start = Instant::now();
    let solutions: Vec<_> = SteinerSearcher::new(2, 3, 13)
        .map(|set| {
            println!("{set:?}");
            set
        })
        .collect();
    let dur = Instant::now().duration_since(start);
    println!("Solutions:");
    for solution in solutions {
        println!("{solution:?}");
    }
    println!("Took {:.2} s", dur.as_secs_f32());
}
