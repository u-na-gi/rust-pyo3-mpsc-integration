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

    // Simulates a heavy computational task in Rust by performing a large loop. This is for demonstration purposes.
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
        // Calculate and display the elapsed time to verify that the task proceeds without blocking
        let duration = start.elapsed();

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
