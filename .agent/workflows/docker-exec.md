---
description: Execute commands in the Docker container lamp-php83ubuntu2404
---

# Docker Execution Configuration

**IMPORTANT**: All commands for this project should be executed inside the Docker container `lamp-php83ubuntu2404`.

## How to Execute Commands

Use the following pattern to execute any command:

```bash
docker exec lamp-php83ubuntu2404 bash -c "cd /var/www/html/JsonQ && <command>"
```

## Common Commands

### Build the Rust extension
```bash
docker exec lamp-php83ubuntu2404 bash -c "cd /var/www/html/JsonQ && cargo build --release"
```

### Run PHP tests
```bash
docker exec lamp-php83ubuntu2404 bash -c "cd /var/www/html/JsonQ && php tests/run_tests.php"
```

### Run benchmarks
```bash
docker exec lamp-php83ubuntu2404 bash -c "cd /var/www/html/JsonQ && php examples/benchmark.php"
```

### Check PHP modules
```bash
docker exec lamp-php83ubuntu2404 bash -c "php -m | grep jsonq"
```

### Run Rust tests
```bash
docker exec lamp-php83ubuntu2404 bash -c "cd /var/www/html/JsonQ && cargo test"
```

## Notes

- The project is mounted at `/var/www/html/JsonQ` inside the container
- The container has PHP 8.3, Rust, and all necessary dependencies installed
- Always prefix commands with `cd /var/www/html/JsonQ &&` to ensure correct working directory
