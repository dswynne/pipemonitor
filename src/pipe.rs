use std::collections::VecDeque;
use zmq;

// TODO: Add pipe name to initilization
pub struct Pipe<T> {
    deque: VecDeque<T>,
    max_capacity: usize,
    address: String,
    socket: zmq::Socket
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
            socket
        }
    }

    pub fn push_back(&mut self, item: T) -> Option<T> {
        let result = if self.deque.len() == self.max_capacity {
            let popped_item = self.deque.pop_front();
            self.deque.push_back(item);
            popped_item
        } else {
            self.deque.push_back(item);
            None
        };

        // TODO: Wrap this whole section into a helper function since we will want it for pop_front too
        // TODO: We will also want to optionally set it up to not be done on every change of the pipe
        // Send current size of deque out zmq socket
        // TODO: compose this as some standard message format that also has the pipename
        let size_str = self.deque.len().to_string();
        // TODO: Handle send errors
        // TODO: Figure out why send causes hang
        // let _ = self.socket.send(size_str.as_bytes(), 0);

        result
    }

    // Other VecDeque methods like pop_front, pop_back, etc., can be
    // implemented by simply delegating to the inner `deque`.
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
        assert_eq!(pipe.deque.len(), 3);
        assert_eq!(pipe.deque[0], 1);
        assert_eq!(pipe.deque[1], 2);
        assert_eq!(pipe.deque[2], 3);
    }
}