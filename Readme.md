# Rust Multithreaded Web Server

A lightweight, multithreaded HTTP web server built entirely from scratch using Rust's standard library (`std`). 

This project demonstrates core systems programming concepts, including raw TCP socket management, HTTP protocol parsing, concurrency, and thread pool implementation, without relying on external crates like `tokio` or `actix-web`.

## Features
* **Custom Thread Pool:** Manages a fixed number of worker threads (4) to handle multiple incoming TCP connections simultaneously without the overhead of spawning a new thread for every request.
* **Static File Routing:** It parses raw HTTP GET requests to safely serve HTML, CSS, and JavaScript and provides a default 404 response when no file is found..
* **Shared State Management:** * Safely manages an internal state (the amount of visitors) between multiple threads.
** **Panic-Free Response to Error Conditions:** Implements error-handling with pattern matching and `Result` types to return errors without causing a panic and crashing the server.
* **Zero-Dependency Frontend:**  Hosts a modern developer hub frontend using only HTML, CSS, and vanilla JS.

---

## How the Code Works

### 1. The Thread Pool Architecture
Creating a new thread for every single web request can overwhelm a system. Instead, this server uses a **Thread Pool** pattern:
* **The Channel (`mpsc`):** A Multiple-Producer, Single-Consumer channel acts as a queue. The main thread (Producer) accepts incoming TCP connections and sends them down the channel as `Job`s (closures).
* **The Workers:** 4 workers get started after server booting and work infinitely in a loop, waiting for jobs to appear in the channel
* **Locking the Queue:** As we have only one consumer from `mpsc` channel, the receiving side of the channel is wrapped inside `Arc<Mutex<Receiver>>`.  Workers compete to lock the mutex, get a job(lol), unlock the mutex, and execute the HTTP request.

### 2. The Request Handler
Each worker reads all incoming bytes using `BufReader` when the stream comes to its turn in the queue. The very first line of request (such as `GET / HTTP/1.1`) will determine what static file it will use according to pattern matching. Then, worker builds an appropriate HTTP Response with correct headers (`Content-Type` and `Content-Length`) for the response body.

### 3. The Shared Visitor Counter
To count visitors, the server uses a shared integer: `Arc<Mutex<usize>>`. 
Whenever the main page (`/`) is hit, the worker thread locks the mutex, safely increments the counter, and unlocks it. This guarantees that simultaneous requests do not overwrite each other's counts.

### 4. Frontend Architecture
The server natively hosts a dark-themed "Developer Hub" UI to demonstrate successful file routing and MIME-type handling:
* **HTML (`index.html`):** Provides the semantic structure and links to the requested assets.
* **CSS (`style.css`):** Proves the server can handle `text/css` content types. It utilizes a modern dark gradient background, CSS variables, and hover transitions for a polished UI.
* **JavaScript (`script.js`):** Proves the server can handle `application/javascript` execution. It features an interactive "terminal" output screen that responds to user clicks, verifying that client-side scripts are successfully delivered and executed by the browser.

---

## Why Rust is considered the best choice for web server? (Relatively Unique Safety and Concurrency features)

Building a multithreaded server in traditional systems languages (like C or C++) often leads to dangerous bugs such as data races, use-after-free errors, or null pointer dereferences. Rust eliminates these at compile time.

### The Borrow Checker & Data Races
Rust's borrow checker enforces a strict rule: you can have either multiple immutable references to data, or exactly one mutable reference, but never both simultaneously. 
* **Advantage:** In this server, multiple threads need to mutate the `visitor_count`. The borrow checker strictly prevents plain sharing. It forces the use of a `Mutex` to guarantee exclusive access, and an `Arc` (Atomic Reference Counted pointer) to safely manage the memory lifecycle across threads. If the `Mutex` is forgotten, the code simply will not compile, making data races impossible.

### Type Safety & Error Handling
* **No Null Pointers:** Rust does not have `null`. Instead, it uses the `Option` enum. In our `Worker` struct, the thread `handle` is stored as an `Option`. This forces the developer to explicitly handle the case where a thread might not exist, preventing null dereference crashes.
* **No Hidden Exceptions:** Operations that can fail return a `Result<T, E>`. In this code, `match` statements and `if let` constructs are used to explicitly handle errors. If a file is missing, the server catches the `Err` and serves a 404 page instead of crashing.

### Concurrency Traits (`Send` and `Sync`)
Rust's type system is aware of concurrency. The `ThreadPool` accepts jobs of type `Box<dyn FnOnce() + Send + 'static>`. The `Send` trait is a compile-time guarantee that the closure is safe to be transferred from the main thread to a worker thread. 

---

## Cargo & The Build System

This project is managed by **Cargo**, Rust's official package manager and build system. Cargo handles compiling the code, downloading dependencies (though this project strictly uses the standard library), and linking the final executable.

### The `Cargo.toml` File
This file sits at the root of the project and acts as the manifest. It defines the project's metadata (name, version) and ensures the compiler knows exactly how to build the application.

### Getting Started & Running the Server

**1. Prerequisites**
Ensure you have the Rust toolchain installed. You can verify this by running:
cargo --version

**Running the Server**
1.Clone the repository and navigate to the project directory.
2.Run the application:
cargo run
3.Open the browser and navigate to this localhost http://127.0.0.1:8000
