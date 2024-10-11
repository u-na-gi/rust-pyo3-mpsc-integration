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
        // Pythonのタスクを実行
        let task = Box::new(move |_: Python, module: &Bound<'_, PyModule>| {
            // "heavy_computation" Python関数を呼び出し、size = 3 の行列を作成
            let result: &PyArray2<f64> = module
                .getattr("heavy_computation")
                .unwrap()
                .call1((sample_size,))
                .unwrap()
                .extract()
                .unwrap();

            println!("result: &PyArray2<f64> =  -> {:?}", result);

            // 結果をRustの配列に変換して返す
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
        let worker = PyWorker::new().expect("fail initialization.");
        let task = create_heavy_task(100000);

        println!("run heavy task");
        let start = Instant::now();
        let receiver = worker.run_task(task);
        // 経過時間を計算
        let duration = start.elapsed();
        // 処理結果と経過時間を表示
        // およそ41.422µsなので、ブロックしないで先に進んでいることがわかる
        println!("PyWorker Time elapsed: {:?}", duration);

        println!("rustで先に進めたい処理。");
        let _ = my_other_task();
        println!("my_other_task success.");

        let result = receiver
            .recv()
            .expect("Failed to receive result from worker");

        // 結果が行列であるか確認し、少なくとも1つの値が存在するか確認
        assert!(!result.is_empty());

        // 結果を確認する（サイズや範囲チェックなど）
        println!("Received result from Python: {:?}", result);
    }
}
