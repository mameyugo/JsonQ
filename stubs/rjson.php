<?php

namespace Rjson;

/**
 * High-performance JSON file storage engine for PHP, powered by Rust.
 */
class Store {
    /**
     * @param string $path Path to the JSON file.
     */
    public function __construct(string $path) {}

    /**
     * @param string $path Dot-notation path to the value.
     */
    public function get(string $path): mixed { return null; }

    /**
     * @param string $path Dot-notation path to the value.
     * @param mixed $value Value to set.
     */
    public function set(string $path, mixed $value): bool { return true; }

    /**
     * @param string $path Dot-notation path to the value.
     */
    public function has(string $path): bool { return false; }

    /**
     * @param string $collection Collection name (top-level key).
     * @param array $conditions MongoDB-style query.
     */
    public function find(string $collection, array $conditions): array { return []; }

    /**
     * @param string $collection Collection name.
     * @param string $field Field to aggregate.
     * @param string $operation sum, avg, min, max, count.
     */
    public function aggregate(string $collection, string $field, string $operation): mixed { return null; }

    /**
     * @param string $path Dot-notation path.
     * @param array $schema JSON Schema subset.
     */
    public function validate(string $path, array $schema): array { return []; }
}
