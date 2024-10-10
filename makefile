build:
	docker compose up -d --build --force-recreate --remove-orphans

test:
	make build
	docker compose exec rust-pyo3-mpsc-integration cargo test