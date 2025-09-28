use std::collections::VecDeque;
use std::ops::{Index, IndexMut};
use zmq;
use std::sync::{Arc, Mutex};

// TODO: Add pipe name to initilization
// TODO: Figure out thread-safety -> https://blog.rust-lang.org/2015/04/10/Fearless-Concurrency/
pub struct Pipe<T> {
    deque: VecDeque<T>,
    max_capacity: usize,
    address: String,
    socket: zmq::Socket,
}

impl<T> Pipe<T> {
    pub fn new(max_capacity: usize, address: String) -> Self {
        // Connect to ZMQ socket at the address
        let context = zmq::Context::new();
        let socket = context.socket(zmq::PUB).expect("Failed to create socket");
        socket.connect(&address).expect("Failed to connect to address");

        Pipe {
            deque: VecDeque::with_capacity(max_capacity),
            max_capacity,
            address,
            socket,
        }
    }

    /// Create a new Pipe wrapped in Arc<Mutex<>> for thread-safe sharing
    pub fn new_shared(max_capacity: usize, address: String) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::new(max_capacity, address)))
    }

    pub fn push_back(&mut self, item: T) -> Option<T> {
        let result = if self.deque.len() == self.max_capacity {
            let popped_item = self.deque.pop_front();
            self.deque.push_back(item);
            popped_item
        } else {
            self.deque.push_back(item);
            // TODO: this should return something better than none...Result vs. option?
            None
        };

        self.report_status();

        result
    }

    pub fn pop_front(&mut self) -> Option<T> {
        let result = if self.deque.len() == 0 {
            None
        } else {
            self.deque.pop_front()
        };

        self.report_status();

        result
    }

    pub fn len(&mut self) -> usize {
        self.deque.len()
    }

    fn report_status(&mut self) {
        // TODO: We will also want to optionally set it up to not be done on every change of the pipe
        // Send current size of deque out zmq socket
        // TODO: compose this as some standard message format that also has the pipename
        let size_str = self.deque.len().to_string();
        // TODO: Handle send errors
        let _ = self.socket.send(size_str.as_bytes(), 0);
    }

}

impl<T> Index<usize> for Pipe<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.deque[index]
    }
}

impl<T> IndexMut<usize> for Pipe<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.deque[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Import everything from the parent module

    #[test]
    fn test_pipe_new() {
        let pipe: Pipe<u32> = Pipe::new(10, String::from("tcp://127.0.0.1:5555"));
        assert_eq!(pipe.max_capacity, 10);
    }

    #[test]
    fn test_push_back_within_capacity() {
        let mut pipe: Pipe<u32> = Pipe::new(3, String::from("tcp://127.0.0.1:5555"));
        assert_eq!(pipe.push_back(1), None);
        assert_eq!(pipe.push_back(2), None);
        assert_eq!(pipe.push_back(3), None);
        assert_eq!(pipe.len(), 3);
        assert_eq!(pipe[0], 1);
        assert_eq!(pipe[1], 2);
        assert_eq!(pipe[2], 3);
    }

    #[test]
    fn test_push_back_over_capacity() {
        let mut pipe: Pipe<u32> = Pipe::new(2, String::from("tcp://127.0.0.1:5555"));
        assert_eq!(pipe.push_back(10), None);
        assert_eq!(pipe.push_back(20), None);
        // Now at capacity, next push should evict 10
        assert_eq!(pipe.push_back(30), Some(10));
        assert_eq!(pipe.len(), 2);
        assert_eq!(pipe[0], 20);
        assert_eq!(pipe[1], 30);
    }

    #[test]
    #[should_panic]
    fn test_index_out_of_bounds() {
        let mut pipe: Pipe<u32> = Pipe::new(1, String::from("tcp://127.0.0.1:5555"));
        pipe.push_back(42);
        let _ = pipe[1]; // Should panic
    }

    #[test]
    fn test_pipe_with_strings() {
        let mut pipe: Pipe<String> = Pipe::new(2, String::from("tcp://127.0.0.1:5555"));
        assert_eq!(pipe.push_back("foo".to_string()), None);
        assert_eq!(pipe.push_back("bar".to_string()), None);
        assert_eq!(pipe[0], "foo");
        assert_eq!(pipe[1], "bar");
        assert_eq!(pipe.push_back("baz".to_string()), Some("foo".to_string()));
        assert_eq!(pipe[0], "bar");
        assert_eq!(pipe[1], "baz");
    }

    #[test]
    #[should_panic]
    fn test_empty_pipe_index_panic() {
        let pipe: Pipe<u32> = Pipe::new(2, String::from("tcp://127.0.0.1:5555"));
        let _ = pipe[0]; // Should panic
    }
}
