# Rust Multithreaded Web Server

A lightweight, multithreaded HTTP web server built entirely from scratch using Rust's standard library (`std`). 

This project demonstrates core systems programming concepts, including raw TCP socket management, HTTP protocol parsing, concurrency, and thread pool implementation, without relying on external crates like `tokio` or `actix-web`.

## Features
* **Custom Thread Pool:** Manages a fixed number of worker threads (4) to handle multiple incoming TCP connections simultaneously without the overhead of spawning a new thread for every request.
* **Static File Routing:** Parses raw HTTP `GET` requests to serve HTML, CSS, and JavaScript files safely, including a 404 fallback.
* **Shared State Management:** Safely tracks a global visitor count across multiple concurrent threads.
* **Panic-Free Error Handling:** Uses pattern matching and `Result` types to gracefully handle missing files and malformed requests without crashing the server.
* **Zero-Dependency Frontend:** Serves a modern, responsive developer hub using pure HTML, CSS, and Vanilla JavaScript.

---

## How the Code Works

### 1. The Thread Pool Architecture
Creating a new thread for every single web request can overwhelm a system. Instead, this server uses a **Thread Pool** pattern:
* **The Channel (`mpsc`):** A Multiple-Producer, Single-Consumer channel acts as a queue. The main thread (Producer) accepts incoming TCP connections and sends them down the channel as `Job`s (closures).
* **The Workers:** The server spawns 4 worker threads upon startup. These threads loop infinitely, waiting for jobs to appear in the channel.
* **Locking the Queue:** Because `mpsc` only allows one consumer, the receiving end of the channel is wrapped in an `Arc<Mutex<Receiver>>`. Workers compete to lock the mutex, grab a job, unlock the mutex, and execute the HTTP request.

### 2. The Request Handler
When a worker receives a `TcpStream`, it reads the incoming bytes using a `BufReader`. It extracts the first line of the HTTP request (e.g., `GET / HTTP/1.1`) and uses pattern matching to route the request to the correct static file. It then formats a valid HTTP response containing the appropriate headers (like `Content-Type` and `Content-Length`) and streams it back to the client.

### 3. The Shared Visitor Counter
To count visitors, the server uses a shared integer: `Arc<Mutex<usize>>`. 
Whenever the main page (`/`) is hit, the worker thread locks the mutex, safely increments the counter, and unlocks it. This guarantees that simultaneous requests do not overwrite each other's counts.

### 4. Frontend Architecture
The server natively hosts a dark-themed "Developer Hub" UI to demonstrate successful file routing and MIME-type handling:
* **HTML (`index.html`):** Provides the semantic structure and links to the requested assets.
* **CSS (`style.css`):** Proves the server can handle `text/css` content types. It utilizes a modern dark gradient background, CSS variables, and hover transitions for a polished UI.
* **JavaScript (`script.js`):** Proves the server can handle `application/javascript` execution. It features an interactive "terminal" output screen that responds to user clicks, verifying that client-side scripts are successfully delivered and executed by the browser.

---

## Why Rust? (Safety and Concurrency Guarantees)

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
```bash
cargo --version