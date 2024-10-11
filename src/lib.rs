pub mod worker;

use pyo3::prelude::*;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Call a Python function that performs heavy initialization with NumPy
fn heavy_numpy_initialization(py: Python) -> PyResult<()> {
    // Use include_str! to embed the Python file's content at build time
    let py_code = include_str!("/app/src-py/from_rust.py");

    // Execute the Python code as a module
    let activators = PyModule::from_code_bound(py, py_code, "from_rust.py", "from_rust").unwrap();

    // Simulate heavy initialization or computation with a large random array
    let result: f64 = activators
        .getattr("heavy_computation")
        .unwrap()
        .call1((10000,))
        .unwrap()
        .extract()
        .unwrap();

    println!("{}", result);

    Ok(())
}

/// Function to manage heavy Python computations and run other tasks concurrently in Rust
pub fn process_data_in_parallel() {
    let (tx, rx) = mpsc::channel();

    // Spawn a thread to handle the heavy Python computation
    let tx_clone = tx.clone();
    let python_thread = thread::spawn(move || {
        Python::with_gil(|py| {
            // Simulate a heavy NumPy computation that takes time
            heavy_numpy_initialization(py).expect("Failed to execute heavy NumPy computation");

            // Notify Rust that the heavy computation is done
            tx_clone
                .send("Python computation complete")
                .expect("Failed to send data");
        });
    });

    // Main thread doing other Rust work while Python computation is running
    println!("Rust is performing other tasks while waiting for Python...");

    // Simulate Rust doing some other work
    for i in 0..5 {
        println!("Rust is processing task {}", i);
        thread::sleep(Duration::from_secs(1));
    }

    // Wait for the message from the Python computation
    let message = rx.recv().unwrap();
    println!("{}", message);

    // Ensure the Python thread has finished
    python_thread.join().unwrap();
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_process_data_in_parallel() {
//         process_data_in_parallel();
//         // The main test here is that the function completes without blocking Rust's work
//     }
// }
