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
To keep track of visitors, the server relies on a shared integer: `Arc<Mutex<usize>>`. Each time someone hits the main page (`/`), the worker thread locks the mutex, increments the counter safely, and then unlocks it. This approach ensures that multiple requests don’t mess up each other’s counts.

### 4. Frontend Architecture
The server comes with a built-in dark-themed "Developer Hub" UI that showcases how file routing and MIME-type handling work:
* **HTML (`index.html`):** This file lays out the semantic structure and links to the necessary assets.
* **CSS (`style.css`):** This demonstrates the server's ability to manage `text/css` content types. It features a sleek dark gradient background, CSS variables, and smooth hover transitions for a refined user experience.
* **JavaScript (`script.js`):** This shows that the server can execute `application/javascript`. It includes an interactive terminal output screen that reacts to user clicks, confirming that client-side scripts are being delivered and executed properly by the browser.

---

 ## Why Rust is considered the best choice for web server? (Relatively Unique Safety and Concurrency features)

When it comes to building a multithreaded server using traditional systems languages like C or C++, developers often run into tricky bugs like data races, use-after-free errors, or null pointer dereferences. Rust steps in to eliminate these issues right at compile time.

### The Borrow Checker & Data Races
Rust's borrow checker has a clear rule: you can either have multiple immutable references to data or just one mutable reference, but never both at the same time.
* **Advantage:** In our server, multiple threads need to update the `visitor_count`. The borrow checker prevents simple sharing, compelling us to use a `Mutex` for exclusive access and an `Arc` (Atomic Reference Counted pointer) to manage memory safely across threads. If we forget the `Mutex`, the code won’t compile, effectively making data races impossible.

### Type Safety & Error Handling
* **No Null Pointers:** Rust doesn’t deal with `null`. Instead, it uses the `Option` enum. In our `Worker` struct, the thread `handle` is stored as an `Option`, which means developers must explicitly handle the scenario where a thread might not exist, thus avoiding those nasty null dereference crashes.
* **No Hidden Exceptions:** Any operation that might fail returns a `Result<T, E>`. In this code, we use `match` statements and `if let` constructs to handle errors directly. If a file is missing, the server catches the `Err` and serves a 404 page instead of crashing.

### Concurrency Traits (`Send` and `Sync`)
Rust’s type system is built with concurrency in mind. The `ThreadPool` takes jobs of type `Box<dyn FnOnce() + Send + 'static>`. The `Send` trait acts as a compile-time assurance that the closure can be safely moved from the main thread to a worker thread.


# Cargo & The Build System
This project is powered by **Cargo**, which is Rust's official package manager and build system. Cargo takes care of compiling the code, downloading dependencies (though for this project, we’re sticking to the standard library), and linking everything together to create the final executable.

### The `Cargo.toml` File
The `Cargo.toml` file lies at the center of our project, which serves as the package manager and a definitive guide. It outlines the project's metadata, like its name and version, and makes sure the compiler knows exactly how to build the application.

### Getting Started & Running the Server

**1. Prerequisites**
First things first, make sure you have the Rust toolchain installed. You can check this by running:\
cargo --version

**Running the Server**\
1. Clone the repository and head over to the project directory.
2. Start the application by running:
cargo run
3. Open your browser and go to this localhost: http://127.0.0.1:8000\

## 🐳Docker Deployment
This application is packaged using a multi-stage Docker build to maximize efficiency.

**Stage 1 (Build):** We use the official Rust image to compile the application. This image comes with all the necessary build tools and toolchains. The project is built with cargo build --release for optimal performance.

**Stage 2 (Runtime):** Here, we switch to a minimal debian-slim image. We only copy over the final compiled binary and the static assets (HTML, CSS, JS) from the build stage.

This approach eliminates the bulky Rust compiler and source code from the final image, resulting in a lightweight, secure, and production-ready container that can be easily deployed to services like AWS ECS, Google Cloud Run, or a VPS.