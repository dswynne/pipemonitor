use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use pipemonitor::pipe::Pipe;

fn main() {
    // Create a shared Pipe wrapped in Arc<Mutex<>>
    let pipe = Pipe::new_shared(5, String::from("tcp://127.0.0.1:5555"));

    // Producer thread: pushes numbers 1..=10 into the pipe
    let pipe_producer = Arc::clone(&pipe);
    let producer = thread::spawn(move || {
        for i in 1..=10 {
            let mut pipe = pipe_producer.lock().unwrap();
            let evicted = pipe.push_back(i);
            if let Some(val) = evicted {
                println!("Producer: pushed {}, evicted {}", i, val);
            } else {
                println!("Producer: pushed {}", i);
            }
            drop(pipe);
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Consumer thread: reads the front of the pipe every 150ms
    let pipe_consumer = Arc::clone(&pipe);
    let consumer = thread::spawn(move || {
        for _ in 0..10 {
            thread::sleep(Duration::from_millis(150));
            let mut pipe = pipe_consumer.lock().unwrap();
            if pipe.len() > 0 {
                println!("Consumer: front value is {}", pipe[0]);
            } else {
                println!("Consumer: pipe is empty");
            }
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}
