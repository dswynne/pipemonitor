use std::sync::{Arc};
use std::thread;
use std::time::Duration;
use pipemonitor::pipe::{Pipe, PipeMessage};

fn main() {
    // Create a shared Pipe wrapped in Arc<Mutex<>>
    let pipe = Pipe::new_shared(5, String::from("tcp://127.0.0.1:5555"), String::from("SharedPipe"));

    // ZMQ listener thread: subscribes and deserializes PipeMessage
    // TODO: Figure out why this doesn't get any data from the producer
    // TODO: Implement this within the actual pipemonitor
    let zmq_listener = thread::spawn(|| {
        let ctx = zmq::Context::new();
        let sub = ctx.socket(zmq::SUB).expect("Failed to create SUB socket");
        sub.connect("tcp://127.0.0.1:5555").expect("Failed to connect SUB");
        sub.set_subscribe(b"").expect("Failed to subscribe");
        sub.set_rcvtimeo(500).expect("Failed to set receive timeout"); // 500 ms timeout
        for _ in 0..10 {
            match sub.recv_bytes(0) {
                Ok(msg) => {
                    match serde_json::from_slice::<pipemonitor::pipe::PipeMessage>(&msg) {
                        Ok(pipe_msg) => println!("ZMQ Listener: received PipeMessage: name={}, size={}, max_capacity={}", pipe_msg.name, pipe_msg.size, pipe_msg.max_capacity),
                        Err(e) => println!("ZMQ Listener: failed to deserialize: {}", e),
                    }
                },
                Err(e) => {
                    if e == zmq::Error::EAGAIN {
                        println!("ZMQ Listener: receive timed out");
                    } else {
                        println!("ZMQ Listener: receive error: {}", e);
                    }
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Give the listener time to connect before producing
    thread::sleep(std::time::Duration::from_millis(300));

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
    // TODO: Add a second pipe here that does something with the data so that we can see two pipes in our demo
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
    zmq_listener.join().unwrap();
}
