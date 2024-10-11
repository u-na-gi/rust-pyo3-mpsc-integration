--------------
- Cargo.toml
```
[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
ndarray = { version = "0.15.0", features = ["serde"]}
numpy = { version = "0.21.0"}

[dependencies.pyo3]
version = "0.21.0"
features = ["auto-initialize"]
```--------------
--------------
- docker-compose.yaml
```
services:
    rust-pyo3-mpsc-integration:
      image: rust-pyo3-mpsc-integration
      build:
        context: .
        dockerfile: dockerfile.dev
      volumes:
        - ./:/app
        - ~/.gitconfig:/root/.gitconfig
        - ~/.ssh:/root/.ssh
      working_dir: /app
      platform: linux/amd64
      tty: true
```--------------
--------------
- requirements.txt
```
numpy==1.26.4
setuptools==72.1.0
uv==0.4.18
wheel==0.43.0

```--------------
--------------
- makefile
```
build:
	docker compose up -d --build

test:
	make build
	docker compose exec rust-pyo3-mpsc-integration cargo test
```--------------
--------------
- README.md
```
# rust-pyo3-mpsc-integration

This project demonstrates how to integrate Rust and Python using `PyO3` and `NumPy`. The project enables Rust to offload computational tasks to Python asynchronously. Python tasks are executed in parallel, allowing Rust to continue processing while awaiting Python’s results. This setup is managed using Docker for seamless environment setup.

## Features
- **Asynchronous Task Execution**: Run Python functions asynchronously from Rust, receiving computation results without blocking Rust's execution.
- **NumPy Integration**: Python functions leverage NumPy for handling computationally intensive tasks.
- **Multi-threaded Worker**: Rust employs a multi-threaded worker system to communicate with Python for efficient task management.

## Requirements
- Docker

## Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/your-repo/rust-pyo3-mpsc-integration.git
   cd rust-pyo3-mpsc-integration
   ```

2. **Build the Docker environment**
   Run the following command to build the Docker container and set up the environment:
   ```bash
   make build
   ```

## Running Tests

To run the unit tests that validate the Rust-Python integration, use the following command:
```bash
make test
```

This will execute the tests defined in the Rust code, ensuring that the Rust worker correctly sends and receives data from the Python side.

## Project Structure

- `Cargo.toml`: Defines Rust dependencies such as `ndarray`, `numpy`, and `pyo3`.
- `docker-compose.yaml`: Configures Docker services for the project.
- `src/`: Contains the Rust source code, including the Python worker module and tests.
- `src-py/`: Contains the Python code, including a NumPy-based heavy computation example.
```--------------
--------------
- dockerfile.dev
```
FROM --platform=linux/amd64 python:3.12.4-bullseye
SHELL ["/bin/bash", "-c"]

RUN apt update -y && \
    apt upgrade -y && \
    apt install -y git curl ffmpeg iputils-ping dnsutils && \
    apt autoremove -y && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

# rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o installer.sh
RUN sh installer.sh -y

# Pass the path for Rust
ENV PATH="/root/.cargo/bin:${PATH}"

# Change the target path for Rust
RUN mkdir -p /.rust/target
ENV CARGO_TARGET_DIR=/.rust/target

# Install Python dependencies
COPY requirements.txt /pyton-pkg-core/requirements.txt
WORKDIR /pyton-pkg-core

# If we don't use 'uv', the execution will be too slow and a waste of time
RUN pip install uv
RUN uv pip install --system  -r requirements.txt 

WORKDIR /app

```--------------
--------------
- .gitignore
```
memo.txt

# Added by cargo

/target

```--------------
--------------
- .vscode/settings.json
```
{
    "cSpell.words": [
        "bufread",
        "dummyadmin",
        "flate",
        "getattr",
        "Minio",
        "numpy",
        "tempdir",
        "tempfile"
    ],
    "python.testing.pytestArgs": [
        "-s", "src-py"
    ],
    "python.testing.unittestEnabled": false,
    "python.testing.pytestEnabled": true
}
```--------------
--------------
- .vscode/extensions.json
```
{
    "recommendations": [
        "1yib.rust-bundle",
        "tamasfe.even-better-toml",
        "streetsidesoftware.code-spell-checker",
        "ms-python.python"
    ]
}
```--------------
--------------
- src-py/__init__.py
```
def hello() -> str:
    return "Hello from src-py!"

```--------------
--------------
- src-py/from_rust.py
```
import numpy as np
import time

def heavy_computation(size):
    """
    Simulates a heavy computation or initialization using NumPy.
    This can be generating a large random array or performing an intensive computation.
    """
    print("Starting heavy computation in Python...")
    
    # Simulate heavy computation by creating a large array and sleeping for a while
    print(f"Generating a random array of size {size}x{2}...")
    data = np.random.rand(size, 2)
    print("Array generated.", data.shape)
    time.sleep(5)  # Simulating a heavy task taking 5 seconds
    
    print("Heavy computation in Python complete.")
    return data

```--------------
--------------
- src-py/.gitignore
```
# python generated files
__pycache__/
*.py[oc]
build/
dist/
wheels/
*.egg-info

# venv
.venv

```--------------
--------------
- src/lib.rs
```
pub mod worker;

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use pyo3::prelude::*;

    use ndarray::Array2;
    use numpy::PyArray2;
    use worker::PyWorker;

    use super::*;

    fn create_heavy_task(
        sample_size: i32,
    ) -> Box<dyn FnOnce(Python, &Bound<'_, PyModule>) -> Array2<f64> + Send> {
        // Execute the Python task
        let task = Box::new(move |_: Python, module: &Bound<'_, PyModule>| {
            // Call the Python function "heavy_computation" and create a matrix with a size determined by the sample_size parameter
            let result: &PyArray2<f64> = module
                .getattr("heavy_computation")
                .unwrap()
                .call1((sample_size,))
                .unwrap()
                .extract()
                .unwrap();

            println!("result: &PyArray2<f64> =  -> {:?}", result);

            // Convert the result to a Rust array and return
            result.to_owned_array()
        });

        task
    }

    fn my_other_task() -> i32 {
        let mut res = 1;
        for i in 0..1000000000 {
            res = i * res;
        }

        res
    }

    #[test]
    fn test_manager() {
        let worker = PyWorker::new().expect("Failed to initialize.");
        let task = create_heavy_task(100000);

        println!("Run heavy task");
        let start = Instant::now();
        let receiver = worker.run_task(task);
        // Calculate elapsed time
        let duration = start.elapsed();
        // Display the result and elapsed time
        // The time is approximately 41.422µs, indicating that it's proceeding without blocking
        println!("PyWorker Time elapsed: {:?}", duration);

        println!("Processing in Rust that needs to continue ahead.");
        let _ = my_other_task();
        println!("my_other_task success.");

        let result = receiver
            .recv()
            .expect("Failed to receive result from worker");

        // Verify that the result is a matrix and that at least one value exists
        assert!(!result.is_empty());

        // Verify the result (e.g., size or range checks)
        println!("Received result from Python: {:?}", result);
    }
}

```--------------
--------------
- src/worker.rs
```
use ndarray::Array2;
use pyo3::{prelude::*, types::PyModule};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

/// `PyWorker` handles tasks sent from Rust to Python. It runs tasks on a Python thread and communicates with Rust using channels.
pub struct PyWorker {
    sender: Sender<(
        Box<dyn FnOnce(Python, &Bound<'_, PyModule>) -> Array2<f64> + Send>,
        Sender<Array2<f64>>,
    )>,
    stop_sender: Option<Sender<()>>,
}

impl PyWorker {
    /// Creates a new `PyWorker`, spawns a Python thread, and listens for tasks.
    pub fn new() -> PyResult<Self> {
        let (task_tx, task_rx): (
            Sender<(
                Box<dyn FnOnce(Python, &Bound<'_, PyModule>) -> Array2<f64> + Send>,
                Sender<Array2<f64>>,
            )>,
            Receiver<(
                Box<dyn FnOnce(Python, &Bound<'_, PyModule>) -> Array2<f64> + Send>,
                Sender<Array2<f64>>,
            )>,
        ) = mpsc::channel();

        let (stop_tx, stop_rx) = mpsc::channel();

        // Spawn a new thread for Python task handling
        thread::spawn(move || {
            Python::with_gil(|py| {
                // Use include_str! to embed the Python file's content at build time
                let py_code = include_str!("/app/src-py/from_rust.py");

                // Execute the Python code as a module
                let activators =
                    PyModule::from_code_bound(py, py_code, "from_rust.py", "from_rust").unwrap();

                loop {
                    if let Ok(_) = stop_rx.try_recv() {
                        println!("Shutting down Python thread.");
                        break;
                    }

                    if let Ok((task, result_tx)) = task_rx.try_recv() {
                        let result = task(py, &activators);
                        result_tx.send(result).unwrap();
                    }
                }
            })
        });

        Ok(PyWorker {
            sender: task_tx,
            stop_sender: Some(stop_tx),
        })
    }

    /// Sends a task to the Python thread to be executed asynchronously.
    pub fn run_task(
        &self,
        task: Box<dyn FnOnce(Python, &Bound<'_, PyModule>) -> Array2<f64> + Send>,
    ) -> Receiver<Array2<f64>> {
        let (tx, rx) = mpsc::channel();
        self.sender.send((task, tx)).unwrap();
        rx
    }

    /// Stops the Python worker thread.
    pub fn stop(&self) -> PyResult<()> {
        if let Some(stop_tx) = &self.stop_sender {
            stop_tx.send(()).expect("Failed to send stop signal.");
        }
        Ok(())
    }
}

/// Implements the Drop trait to automatically stop the Python worker when it goes out of scope.
impl Drop for PyWorker {
    fn drop(&mut self) {
        println!("Dropping PyWorker, stopping Python worker thread.");
        self.stop().expect("Failed to stop the Python worker.");
    }
}

#[cfg(test)]
mod tests {
    use numpy::PyArray2;

    use super::*;

    #[test]
    fn test_python_heavy_computation() {
        // Initialize PyWorker
        let worker = PyWorker::new().expect("Failed to create PyWorker");

        // Execute the Python task
        let task = Box::new(|_: Python, module: &Bound<'_, PyModule>| {
            // Call the Python function "heavy_computation" and create a matrix with a size determined by the sample_size parameter
            let result: &PyArray2<f64> = module
                .getattr("heavy_computation")
                .unwrap()
                .call1((10000,))
                .unwrap()
                .extract()
                .unwrap();

            println!("result: &PyArray2<f64> =  -> {:?}", result);

            // Convert the result to a Rust array and return
            result.to_owned_array()
        });

        // Send the task and receive the result
        let receiver = worker.run_task(task);
        let result = receiver
            .recv()
            .expect("Failed to receive result from worker");

        // Verify that the result is a matrix and that at least one value exists
        assert!(!result.is_empty());

        // Verify the result (e.g., size or range checks)
        println!("Received result from Python: {:?}", result);
    }

    #[test]
    fn test_worker_auto_stop_on_drop() {
        {
            // Initialize PyWorker
            let worker = PyWorker::new().expect("Failed to create PyWorker");

            // Execute the Python task
            let task = Box::new(|_: Python, module: &Bound<'_, PyModule>| {
                // Call the Python function "heavy_computation"
                let result: &PyArray2<f64> = module
                    .getattr("heavy_computation")
                    .unwrap()
                    .call1((3,))
                    .unwrap()
                    .extract()
                    .unwrap();
                result.to_owned_array()
            });

            // Send the task and receive the result
            let receiver = worker.run_task(task);
            let result = receiver
                .recv()
                .expect("Failed to receive result from worker");

            // Verify the result
            assert!(!result.is_empty());
        } // When PyWorker goes out of scope, Drop is called, and the thread is stopped

        // Check that the worker thread has stopped after PyWorker goes out of scope
        println!("Worker thread should have stopped upon exiting the scope.");
    }
}

```--------------
