use std::fs;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

// i) Custom Thread Pool Implementation (better than naive threading because of memory management and performance)


// A unit of work executed by the pool
type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    // Wrapped in Option so we can gracefully shut down later
    sender: Option<mpsc::Sender<Job>>,
}

struct Worker {
    id: usize,
    handle: Option<thread::JoinHandle<()>>,
}

impl ThreadPool {
    /// Create a new thread pool with a fixed number of workers
    pub fn new(size: usize) -> Self {
        assert!(size > 0, "Thread pool size must be greater than zero");

        let (sender, receiver) = mpsc::channel();
        let shared_receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::spawn(id, Arc::clone(&shared_receiver)));
        }

        Self {
            workers,
            sender: Some(sender),
        }
    }

    /// Submit a task to be executed by the pool
    pub fn execute<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(task);

        // Using expect gives a clearer failure message than unwrap
        self.sender
            .as_ref()
            .expect("ThreadPool has been shut down")
            .send(job)
            .expect("Failed to send job to worker threads");
    }
}

impl Worker {
    /// Spawn a new worker thread that continuously pulls jobs
    fn spawn(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let handle = thread::spawn(move || {
            loop {
                // Lock scope kept tight to avoid holding the mutex unnecessarily
                let job_result = {
                    let lock = receiver.lock().expect("Worker failed to lock receiver");
                    lock.recv()
                };

                match job_result {
                    Ok(job) => {
                        // You might log here in a real server
                        job();
                    }
                    Err(_) => {
                        // Channel closed => shutdown signal
                        break;
                    }
                }
            }
        });

        Self {
            id,
            handle: Some(handle),
        }
    }
}

//  ii) Web Server Implementation with Visitor Counter (using Arc and Mutex for thread safety)

fn main() {
    // Bind server to localhost:8000
    let listener = TcpListener::bind("127.0.0.1:8000")
        .expect("Failed to bind to address");

    // Fixed-size thread pool
    let pool = ThreadPool::new(4);

    // Shared state: visitor counter
    let visitor_count = Arc::new(Mutex::new(0));

    println!("Server running at http://127.0.0.1:8000");

    for incoming in listener.incoming() {
        let stream = match incoming {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("Connection failed: {}", err);
                continue;
            }
        };

        let counter = Arc::clone(&visitor_count);

        pool.execute(move || {
            handle_connection(stream, counter);
        });
    }
}

fn handle_connection(mut stream: TcpStream, counter: Arc<Mutex<usize>>) {
    let mut reader = BufReader::new(&mut stream);

    // Extract the first request line (e.g., "GET / HTTP/1.1")
    let request_line = match reader.lines().next() {
        Some(Ok(line)) => line,
        _ => {
            eprintln!("Failed to read request line");
            return;
        }
    };

    // Route handling (basic pattern matching)
    let (status, file, content_type, should_count) = match request_line.as_str() {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "index.html", "text/html", true),
        "GET /style.css HTTP/1.1" => ("HTTP/1.1 200 OK", "style.css", "text/css", false),
        "GET /script.js HTTP/1.1" => ("HTTP/1.1 200 OK", "script.js", "application/javascript", false),
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html", "text/html", false),
    };

    // Update visitor count only for main page hits
    if should_count {
        let mut count = counter.lock().expect("Failed to lock counter");
        *count += 1;
        println!("Visitors so far: {}", *count);
    }

    // Load file contents
    let body = fs::read_to_string(file)
        .unwrap_or_else(|_| String::from("<h1>404 - File not found</h1>"));

    let response = format!(
        "{status}\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n{}",
        body.len(),
        content_type,
        body
    );

    // Send response
    if let Err(err) = stream.write_all(response.as_bytes()) {
        eprintln!("Failed to send response: {}", err);
    }
}