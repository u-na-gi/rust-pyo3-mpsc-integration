build:
	docker compose up -d --build

test:
	make build
	docker compose exec rust-pyo3-mpsc-integration cargo test