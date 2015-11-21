/*
 * Copyright 2015 Ben Ashford
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#![crate_type = "lib"]
#![crate_name = "lazysort"]

use std::cmp::Ordering;
use std::cmp::Ordering::{Less, Equal, Greater};
use std::fmt::Debug;

fn pivot(lower: usize, upper: usize) -> usize {
    return upper + ((lower - upper) / 2);
}

pub struct LazySortIterator<T: Debug, F> {
    data: Vec<T>,
    work: Vec<(usize, usize)>,
    by: F,
}

impl<T, F> LazySortIterator<T, F> where
    T: Debug,
    F: FnMut(&T, &T) -> Ordering,
{
    fn new(data: Vec<T>, by: F) -> Self where
        F: FnMut(&T, &T) -> Ordering
    {
        let l = data.len();
        LazySortIterator {
            data: data,
            work: if l == 0 {
                vec![]
            } else {
                vec![(l - 1, 0)]
            },
            by: by
        }
    }

    fn partition(&mut self, lower: usize, upper: usize, p: usize) -> usize {
        assert!(lower >= upper);
        assert!(p <= lower);
        assert!(p >= upper);

        let length = lower - upper;
        if length == 0 {
            p
        } else {
            let lasti = lower;
            let (mut i, mut nextp) = (upper, upper);
            self.data.swap(lasti, p);
            while i < lasti {
                match (self.by)(&self.data[i], &self.data[lasti]) {
                    Greater => {
                        if i != nextp {
                            self.data.swap(i, nextp);
                        }
                        nextp = nextp + 1;
                    },
                    Equal => (),
                    Less => ()
                }
                i = i + 1;
            }
            self.data.swap(nextp, lasti);
            nextp
        }
    }

    fn qsort(&mut self, lower: usize, upper: usize) -> T {
        if lower == upper {
            assert!(lower == self.data.len() - 1);
            return self.data.pop().expect("Non empty vector");
        }

        let p = pivot(lower, upper);
        let p = self.partition(lower, upper, p);

        if p == lower {
            self.work.push((p - 1, upper));
            self.qsort(lower, p)
        } else {
            self.work.push((p, upper));
            self.qsort(lower, p + 1)
        }
    }
}

pub trait Sorted {
    type Item: Debug + Ord;

    fn sorted(self) ->
        LazySortIterator<Self::Item, fn(&Self::Item, &Self::Item) -> Ordering>;
}

pub trait SortedPartial {
    type Item: Debug + PartialOrd;

    fn sorted_partial(self, first: bool) ->
        LazySortIterator<Self::Item, fn(&Self::Item, &Self::Item) -> Ordering>;
}

pub trait SortedBy {
    type Item: Debug;

    fn sorted_by<F>(self, F) -> LazySortIterator<Self::Item, F>
        where F: Fn(&Self::Item, &Self::Item) -> Ordering;
}

impl<T, I> Sorted for I where
    T: Debug + Eq + Ord,
    I: Iterator<Item=T>
{
    type Item = T;

    fn sorted(self) -> LazySortIterator<T, fn(&Self::Item, &Self::Item) -> Ordering> {
        LazySortIterator::new(self.collect(), Ord::cmp)
    }
}

fn partial_cmp_first<T: PartialOrd>(a: &T, b: &T) -> Ordering {
    match a.partial_cmp(b) {
        Some(order) => order,
        None        => Less
    }
}

fn partial_cmp_last<T: PartialOrd>(a: &T, b: &T) -> Ordering {
    match a.partial_cmp(b) {
        Some(order) => order,
        None        => Greater
    }
}

impl<T, I> SortedPartial for I where
    T: Debug + PartialOrd,
    I: Iterator<Item=T>
{
    type Item = T;

    fn sorted_partial(self, first: bool) -> LazySortIterator<T, fn(&Self::Item, &Self::Item) -> Ordering> {
        if first {
            LazySortIterator::new(self.collect(), partial_cmp_first)
        } else {
            LazySortIterator::new(self.collect(), partial_cmp_last)
        }
    }
}

impl<T, I> SortedBy for I where
    T: Debug,
    I: Iterator<Item=T>,
{
    type Item = T;

    fn sorted_by<F>(self, by: F) -> LazySortIterator<T, F> where
        F: Fn(&T, &T) -> Ordering
    {
        LazySortIterator::new(self.collect(), by)
    }
}

impl<T, F> Iterator for LazySortIterator<T, F> where
    T: Debug,
    F: FnMut(&T, &T) -> Ordering,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        match self.work.pop() {
            Some(next_work) => {
                let (lower, upper) = next_work;
                Some(self.qsort(lower, upper))
            },
            None => None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = self.data.len();
        (l, Some(l))
    }
}

// TESTS

#[cfg(test)]
mod tests {
    use super::Sorted;
    use super::SortedPartial;
    use super::SortedBy;

    #[test]
    fn sorted_test() {
        let expected: Vec<u64> = vec![1u64, 1, 1, 3, 4, 6, 7, 9, 22];
        let before: Vec<u64> = vec![9u64, 7, 1, 1, 6, 3, 1, 4, 22];
        let after: Vec<u64> = before.iter().sorted().map(|x| *x).collect();

        assert_eq!(expected, after);
    }

    #[test]
    fn empty_test() {
        let before: Vec<u64> = vec![];
        let after: Vec<u64> = before.iter().sorted().map(|x| *x).collect();
        assert_eq!(before, after);
    }

    #[test]
    fn sorted_partial_test() {
        let expected: Vec<f64> = vec![0.9_f64, 1.0, 1.0, 1.1, 75.3, 75.3];
        let before: Vec<f64> = vec![1.0_f64, 1.1, 0.9, 75.3, 1.0, 75.3];
        let after: Vec<f64> = before.iter().sorted_partial(true).map(|x| *x).collect();

        assert_eq!(expected, after);
    }

    #[test]
    fn sorted_by_test() {
        let expected: Vec<u64> = vec![4, 1, 3, 2];
        let before: Vec<(f64, u64)> = vec![(0.2, 1),
                                           (0.9, 2),
                                           (0.4, 3),
                                           (0.1, 4)];

        let after: Vec<u64> = before.iter()
            .sorted_by(|&a, &b| {
                let (ax, _) = *a;
                let (bx, _) = *b;
                ax.partial_cmp(&bx).unwrap()
            })
            .map(|&(_, y)| y)
            .collect();

        assert_eq!(expected, after);
    }
}
