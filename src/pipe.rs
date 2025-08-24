use std::collections::VecDeque;

pub struct Pipe<T> {
    deque: VecDeque<T>,
    max_capacity: usize,
}

impl<T> Pipe<T> {
    pub fn new(max_capacity: usize) -> Self {
        Pipe {
            deque: VecDeque::with_capacity(max_capacity),
            max_capacity,
        }
    }

    pub fn push_back(&mut self, item: T) -> Option<T> {
        if self.deque.len() == self.max_capacity {
            let popped_item = self.deque.pop_front();
            self.deque.push_back(item);
            popped_item
        } else {
            self.deque.push_back(item);
            None
        }
    }

    // Other VecDeque methods like pop_front, pop_back, etc., can be
    // implemented by simply delegating to the inner `deque`.
}

#[cfg(test)]
mod tests {
    use super::*; // Import everything from the parent module

    #[test]
    fn test_pipe_new() {
        let pipe: Pipe<u32> = Pipe::new(10);
        assert_eq!(pipe.max_capacity, 10);
    }

    #[test]
    fn test_push_back_within_capacity() {
        let mut pipe: Pipe<u32> = Pipe::new(3);
        assert_eq!(pipe.push_back(1), None);
        assert_eq!(pipe.push_back(2), None);
        assert_eq!(pipe.push_back(3), None);
        assert_eq!(pipe.deque.len(), 3);
        assert_eq!(pipe.deque[0], 1);
        assert_eq!(pipe.deque[1], 2);
        assert_eq!(pipe.deque[2], 3);
    }
}