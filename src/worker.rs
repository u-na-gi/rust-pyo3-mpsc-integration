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

                // Call the Python function
                // let result: f64 = activators
                //     .getattr("my_numpy_function")
                //     .unwrap()
                //     .call1((5.0,))
                //     .unwrap()
                //     .extract()
                //     .unwrap();
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
        // PyWorkerの初期化
        let worker = PyWorker::new().expect("Failed to create PyWorker");

        // Pythonのタスクを実行
        let task = Box::new(|_: Python, module: &Bound<'_, PyModule>| {
            // "heavy_computation" Python関数を呼び出し、size = 3 の行列を作成
            let result: &PyArray2<f64> = module
                .getattr("heavy_computation")
                .unwrap()
                .call1((10000,))
                .unwrap()
                .extract()
                .unwrap();

            println!("result: &PyArray2<f64> =  -> {:?}", result);

            // 結果をRustの配列に変換して返す
            result.to_owned_array()
        });

        // タスクの送信と結果の受信
        let receiver = worker.run_task(task);
        let result = receiver
            .recv()
            .expect("Failed to receive result from worker");

        // 結果が行列であるか確認し、少なくとも1つの値が存在するか確認
        assert!(!result.is_empty());

        // 結果を確認する（サイズや範囲チェックなど）
        println!("Received result from Python: {:?}", result);
    }

    #[test]
    fn test_worker_auto_stop_on_drop() {
        {
            // PyWorkerの初期化
            let worker = PyWorker::new().expect("Failed to create PyWorker");

            // Pythonのタスクを実行
            let task = Box::new(|_: Python, module: &Bound<'_, PyModule>| {
                // Python関数 heavy_computation を呼び出す
                let result: &PyArray2<f64> = module
                    .getattr("heavy_computation")
                    .unwrap()
                    .call1((3,))
                    .unwrap()
                    .extract()
                    .unwrap();
                result.to_owned_array()
            });

            // タスクの送信と結果の受信
            let receiver = worker.run_task(task);
            let result = receiver
                .recv()
                .expect("Failed to receive result from worker");

            // 結果を確認
            assert!(!result.is_empty());
        } // PyWorkerがスコープを抜けるとDropが呼ばれ、スレッドが停止する

        // PyWorkerがスコープを抜けた後、スレッドが終了しているか確認
        println!("Worker thread should have stopped upon exiting the scope.");
    }
}
