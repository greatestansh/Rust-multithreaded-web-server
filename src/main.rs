use std::fs;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

// i) Custom Thread Pool Implementation (better than naive threading because of memory management and performance)
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();
            match message {
                Ok(job) => job(),
                Err(_) => break,
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

//  ii) Web Server Implementation with Visitor Counter (using Arc and Mutex for thread safety)

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
    let pool = ThreadPool::new(4); // Requirement: Handle up to 4 incoming connections
    
    // Shared visitor counter using Arc and Mutex for thread safety
    let visitor_count = Arc::new(Mutex::new(0));

    println!("Server listening on port 8000...");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let counter_clone = Arc::clone(&visitor_count);

        pool.execute(move || {
            handle_connection(stream, counter_clone);
        });
    }
}

fn handle_connection(mut stream: TcpStream, counter: Arc<Mutex<usize>>) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = match buf_reader.lines().next() {
        Some(Ok(line)) => line,
        _ => return,
    };

    let (status_line, filename, content_type, increment_counter) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "index.html", "text/html", true),
        "GET /style.css HTTP/1.1" => ("HTTP/1.1 200 OK", "style.css", "text/css", false),
        "GET /script.js HTTP/1.1" => ("HTTP/1.1 200 OK", "script.js", "application/javascript", false),
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html", "text/html", false), // Optional 404 handling
    };

    // Only increment counter for main page visits, not for fetching CSS/JS
    if increment_counter {
        let mut num = counter.lock().unwrap();
        *num += 1;
        println!("Visitor count: {}", *num);
    }

    // Read file contents (ensure these files exist in your root directory)
    let contents = fs::read_to_string(filename).unwrap_or_else(|_| String::from("File not found"));
    let length = contents.len();

    let response = format!(
        "{status_line}\r\nContent-Length: {length}\r\nContent-Type: {content_type}\r\n\r\n{contents}"
    );

    stream.write_all(response.as_bytes()).unwrap();
}