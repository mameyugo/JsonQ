# JsonQ Project Configuration

## Development Environment

**Docker Container**: `lamp-php83ubuntu2404`

All development commands, tests, builds, and benchmarks should be executed inside this Docker container.

## Project Structure

- **Host Path**: `c:\xampp\htdocs\JsonQ`
- **Container Path**: `/var/www/html/JsonQ`

## Execution Pattern

Always use:
```bash
docker exec lamp-php83ubuntu2404 bash -c "cd /var/www/html/JsonQ && <your-command>"
```

## Environment Details

- PHP: 8.3
- Rust: Latest stable
- OS: Ubuntu 24.04
- Extension location: Built in `target/release/libjsonq.so`
