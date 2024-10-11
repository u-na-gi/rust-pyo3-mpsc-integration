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