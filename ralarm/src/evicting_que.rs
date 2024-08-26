use std::collections::VecDeque;

pub struct EvictingQue<T> {
    deque: VecDeque<T>,
    capacity: usize,
}

impl<T> EvictingQue<T> {
    pub fn new(capacity: usize) -> EvictingQue<T> {
        EvictingQue {
            deque: VecDeque::<T>::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, elem: T) -> Option<T> {
        let evicted_elem = match self.deque.len() >= self.capacity {
            true => self.deque.pop_front(),
            false => None,
        };
        self.deque.push_back(elem);
        evicted_elem
    }

    pub fn len(&self) -> usize {
        self.deque.len()
    }
}

impl<T, const S: usize> From<([T; S], usize)> for EvictingQue<T> {
    fn from(value: ([T; S], usize)) -> Self {
        let mut deque = EvictingQue::new(value.1);
        for elem in value.0 {
            deque.push(elem);
        }
        deque
    }
}

impl<'a, T> IntoIterator for &'a EvictingQue<T> {
    type Item = &'a T;
    type IntoIter = std::collections::vec_deque::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.deque.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evicts_old_elements() {
        let mut que = EvictingQue::<i32>::from(([1, 2, 3], 3));

        assert_eq!(que.push(100), Some(1));

        assert_eq!(que.push(100), Some(2));
        assert_eq!(que.push(100), Some(3));
    }

    #[test]
    fn does_not_evict_until_capacity() {
        let mut que = EvictingQue::<i32>::from(([1, 2, 3], 5));

        assert!(que.push(100).is_none());

        assert!(que.push(100).is_none());
        assert!(que.push(100).is_some());
    }

    #[test]
    fn should_clip_array_from_back_if_capacity_extends_length() {
        let mut que = EvictingQue::<i32>::from(([1, 2, 3], 2));

        assert_eq!(que.push(100), Some(2));
        assert_eq!(que.push(100), Some(3));
    }

    #[test]
    fn should_iterate_through_elements() {
        let mut que = EvictingQue::from(([1, 2, 3, 4, 5], 4));
        que.push(6);

        let mut elems = Vec::<i32>::new();

        for elem in &que {
            elems.push(*elem)
        }

        assert_eq!(elems, vec![3, 4, 5, 6])
    }

    #[test]
    fn internal_should_never_change_capacity() {
        let capacity = 5;
        let mut que = EvictingQue::<i32>::new(capacity);

        for i in 0..1_000 {
            que.push(i);

            assert_eq!(que.deque.capacity(), capacity)
        }
    }
}
