<?php
/**
 * JsonQ — High-performance JSON file storage engine for PHP
 *
 * IDE autocompletion stubs for the JsonQ native extension.
 * This file is loaded automatically via Composer when the extension
 * is not present, providing type hints and autocompletion for IDEs.
 *
 * DO NOT include this file manually. DO NOT use these stubs at runtime.
 * In production, the native Rust extension provides the real implementations.
 *
 * @package  JsonQ
 * @version  0.7.0
 * @license  The PHP License, version 3.01
 * @link     https://github.com/mameyugo/JsonQ
 */

// Guard: only define stubs if the real extension is not loaded.
// In production (with ext-jsonq), PHP uses the native Rust classes.
// In development/CI without the extension, IDEs use these stubs.
namespace {
    if (!extension_loaded('jsonq')) {

        // ──────────────────────────────────────────────────────────────
        // Global Functions
        // ──────────────────────────────────────────────────────────────

    /**
     * Get the JsonQ extension version.
     *
     * @return string Semantic version string (e.g., "0.7.0")
     */
    function jsonq_version(): string
    {
        throw new \RuntimeException(
            'The JsonQ extension (ext-jsonq) is not loaded. '
            . 'Install it following the instructions at https://github.com/mameyugo/JsonQ'
        );
    }

    /**
     * Get current global configuration.
     *
     * Returns an array with:
     * - max_file_size:        int     (bytes)
     * - max_file_size_mb:     float
     * - max_validation_depth: int
     * - max_path_depth:       int
     * - allowed_extensions:   string[]
     * - base_path:            string|null
     *
     * @return array<string, mixed>
     */
    function jsonq_get_config(): array
    {
        throw new \RuntimeException('The JsonQ extension (ext-jsonq) is not loaded.');
    }

    /**
     * Set the maximum file size for all Store instances.
     *
     * Supports human-readable units: "100M", "1G", "500K".
     *
     * @param  string $size Format: [number][unit]  e.g. "50M"
     * @return bool
     */
    function jsonq_set_max_file_size(string $size): bool
    {
        throw new \RuntimeException('The JsonQ extension (ext-jsonq) is not loaded.');
    }

    /**
     * Set the allowed file extensions for Store instances.
     *
     * @param  string $extensions Comma-separated list (e.g., "json,db,data")
     * @return bool
     */
    function jsonq_set_allowed_extensions(string $extensions): bool
    {
        throw new \RuntimeException('The JsonQ extension (ext-jsonq) is not loaded.');
    }

    /**
     * Set a base path restriction for all file operations (security sandbox).
     *
     * All Store paths must reside within this base path.
     *
     * @param  string $path Absolute directory path
     * @return bool
     */
    function jsonq_set_base_path(string $path): bool
    {
        throw new \RuntimeException('The JsonQ extension (ext-jsonq) is not loaded.');
    }

    /**
     * Clear the base path restriction.
     *
     * @return bool
     */
    function jsonq_clear_base_path(): bool
    {
        throw new \RuntimeException('The JsonQ extension (ext-jsonq) is not loaded.');
    }

    /**
     * Write a Store's data to a file using memory-efficient streaming.
     *
     * @param  string    $path        Source JSON file path
     * @param  string    $outputPath  Target output file path
     * @param  bool|null $pretty      Whether to pretty-print (default: false)
     * @return bool
     */
    function jsonq_write_to_file(string $path, string $outputPath, ?bool $pretty = null): bool
    {
        throw new \RuntimeException('The JsonQ extension (ext-jsonq) is not loaded.');
    }

    /**
     * Append a record to a JSONL (newline-delimited JSON) file.
     *
     * Validates that the record is valid JSON before appending.
     *
     * @param  string $path   JSONL file path
     * @param  string $record JSON string
     * @return bool
     */
    function jsonq_append_jsonl(string $path, string $record): bool
    {
        throw new \RuntimeException('The JsonQ extension (ext-jsonq) is not loaded.');
    }

    /**
     * Read all records from a JSONL file.
     *
     * @param  string $path JSONL file path
     * @return string[]     Array of raw JSON strings, one per line
     */
    function jsonq_read_jsonl(string $path): array
    {
        throw new \RuntimeException('The JsonQ extension (ext-jsonq) is not loaded.');
    }

    /**
     * Get memory optimization and key-deduplication statistics for a file.
     *
     * Returns an array with:
     * - unique_keys:          int
     * - total_references:     int
     * - memory_saved_percent: int
     *
     * @param  string $path JSON file path
     * @return array<string, int>
     */
    function jsonq_memory_stats(string $path): array
    {
        throw new \RuntimeException('The JsonQ extension (ext-jsonq) is not loaded.');
    }

    /**
     * Query a JSON file directly using JSONPath without loading it fully into PHP.
     *
     * @param  string   $path       JSON file path
     * @param  string   $queryPath  JSONPath expression (e.g., "$.users[*].name")
     * @return string[]             Array of serialized JSON results
     */
    function jsonq_query_node(string $path, string $queryPath): array
    {
        throw new \RuntimeException('The JsonQ extension (ext-jsonq) is not loaded.');
    }

    /**
     * Alias for {@see jsonq_query_node()}.
     *
     * @param  string   $path  JSON file path
     * @param  string   $query JSONPath expression
     * @return string[]
     */
    function jsonq_query(string $path, string $query): array
    {
        throw new \RuntimeException('The JsonQ extension (ext-jsonq) is not loaded.');
    }

    // ──────────────────────────────────────────────────────────────
    // JsonQ\Store Class
    // ──────────────────────────────────────────────────────────────

    } // end if 
} // end global namespace

namespace JsonQ {

    if (!extension_loaded('jsonq')) {

        /**
         * Stream Filter for memory-efficient MongoDB-style query filtering.
         * Useful for filtering JSON datasets directly while streaming.
         */
        class StreamFilter
        {
            /**
             * @param array $conditions MongoDB-style query filters
             * @param array $projection Fields to extract/keep
             */
            public function __construct(array $conditions, array $projection = []) {}

            /**
             * Applies the configured conditions and projection to an item.
             * Returns the modified item, or null if it fails the condition.
             *
             * @param mixed $item The decoded JSON item to filter
             * @return mixed|null
             */
            public function apply(mixed $item): mixed {}
        }

        /**
         * High-performance JSON file storage engine backed by Rust.
         *
         * Provides file-based JSON CRUD, MongoDB-style queries, a fluent
         * query builder, aggregation, schema validation, in-memory indexes,
         * transactions, and observability metrics.
         *
         * @package JsonQ
         *
         * @example
         * ```php
         * $store = new \JsonQ\Store('/var/data/app.json');
         * $store->set('users', []);
         * $store->push('users', ['id' => 1, 'name' => 'Alice', 'role' => 'admin']);
         * $admins = $store->find('users', ['role' => 'admin']);
         * ```
         */
        class Store
        {
            // ── Constructor ───────────────────────────────────────

            /**
             * Create a new Store instance.
             *
             * If the file does not exist it is created and initialised with `{}`.
             *
             * @param string $path Absolute or relative path to the JSON file.
             * @throws \RuntimeException If the JsonQ extension is not loaded.
             */
            public function __construct(string $path)
            {
                throw new \RuntimeException(
                    'The JsonQ extension (ext-jsonq) is not loaded. '
                    . 'Install it following the instructions at https://github.com/mameyugo/JsonQ'
                );
            }

            // ── Read Operations ───────────────────────────────────

            /**
             * Get a value by dot-notation path.
             *
             * @param  string $path Dot-notation path (e.g., "users.0.name")
             * @return mixed        The value at the path, or null if not found
             */
            public function get(string $path): mixed {}

            /**
             * Check whether a path exists in the data.
             *
             * @param  string $path Dot-notation path
             * @return bool
             */
            public function has(string $path): bool {}

            /**
             * Count elements at a path.
             *
             * @param  string $path Path to an array or object
             * @return int          Number of elements, or -1 if the path is invalid
             */
            public function count(string $path): int {}

            /**
             * Get the top-level keys at a path.
             *
             * @param  string   $path Dot-notation path (empty string for root)
             * @return string[]
             */
            public function keys(string $path): array {}

            /**
             * Get the values of an object at a path.
             *
             * @param  string  $path Dot-notation path
             * @return array
             */
            public function values(string $path): array {}

            // ── Write Operations ──────────────────────────────────

            /**
             * Set a value at a dot-notation path.
             *
             * Creates intermediate objects/arrays as needed.
             *
             * @param  string $path  Dot-notation path
             * @param  mixed  $value Value to set
             * @return bool
             * @throws \Exception If the store is not initialised
             */
            public function set(string $path, mixed $value): bool {}

            /**
             * Remove a value at a path.
             *
             * @param  string $path Dot-notation path
             * @return bool
             * @throws \Exception If the store is not initialised
             */
            public function remove(string $path): bool {}

            /**
             * Push a value onto an array at a path.
             *
             * @param  string $path  Path to the target array
             * @param  mixed  $value Value to append
             * @return bool          True if pushed, false if target is not an array
             * @throws \Exception If the store is not initialised
             */
            public function push(string $path, mixed $value): bool {}

            /**
             * Deep-merge data into a path.
             *
             * Object keys are merged recursively. Non-object values are replaced.
             *
             * @param  string $path  Dot-notation path
             * @param  mixed  $value Data to merge
             * @return bool
             * @throws \Exception If the store is not initialised
             */
            public function merge(string $path, mixed $value): bool {}

            /**
             * Atomically increment a numeric value.
             *
             * @param  string     $path   Path to a numeric value
             * @param  float|null $amount Amount to add (default: 1.0)
             * @return bool
             * @throws \Exception If the store is not initialised
             */
            public function increment(string $path, ?float $amount = null): bool {}

            /**
             * Atomically decrement a numeric value.
             *
             * @param  string     $path   Path to a numeric value
             * @param  float|null $amount Amount to subtract (default: 1.0)
             * @return bool
             * @throws \Exception If the store is not initialised
             */
            public function decrement(string $path, ?float $amount = null): bool {}

            // ── Query Operations ──────────────────────────────────

            /**
             * Find records matching MongoDB-style conditions.
             *
             * Automatically uses indexes for simple equality queries when available.
             *
             * **Supported operators:**
             * - Comparison:  `$eq`, `$ne`, `$gt`, `$gte`, `$lt`, `$lte`
             * - Array:       `$in`, `$nin`
             * - String:      `$regex`, `$contains`, `$startsWith`, `$endsWith`
             * - Type/exists: `$exists`, `$size`, `$type`
             * - Logical:     `$and`, `$or`, `$not`
             *
             * @param  string               $collection Dot-notation path to the array
             * @param  array<string, mixed> $conditions MongoDB-style filter conditions
             * @return array                            Matching records
             *
             * @example
             * ```php
             * // Simple equality
             * $store->find('users', ['role' => 'admin']);
             *
             * // Comparison + logical
             * $store->find('users', ['age' => ['$gte' => 18, '$lt' => 65]]);
             * $store->find('users', ['$or' => [['city' => 'NYC'], ['city' => 'LA']]]);
             *
             * // String operators
             * $store->find('users', ['email' => ['$endsWith' => '@example.com']]);
             * ```
             */
            public function find(string $collection, array $conditions): array {}

            /**
             * Find the first record matching conditions.
             *
             * @param  string               $collection Dot-notation path to the array
             * @param  array<string, mixed> $conditions MongoDB-style filter conditions
             * @return array|null                       First matching record, or null
             */
            public function findOne(string $collection, array $conditions): mixed {}

            /**
             * Execute a fluent query specification.
             *
             * **Query spec keys:**
             * - `where`    — `array` of `{field, op, value}` conditions
             * - `order_by` — `{field: string, direction: "asc"|"desc"}`
             * - `limit`    — `int`
             * - `offset`   — `int`
             * - `select`   — `string[]` fields to return
             *
             * **Supported `op` values:**
             * `=`, `!=`, `<>`, `>`, `>=`, `<`, `<=`,
             * `in`, `not in`, `contains`, `starts_with`, `ends_with`, `between`
             *
             * @param  string              $collection Dot-notation path to the array
             * @param  array<string, mixed> $querySpec  Fluent query specification
             * @return array                            Matching records
             *
             * @example
             * ```php
             * $store->executeQuery('users', [
             *     'where'    => [['field' => 'active', 'op' => '=', 'value' => true]],
             *     'order_by' => ['field' => 'name', 'direction' => 'asc'],
             *     'limit'    => 10,
             *     'offset'   => 0,
             *     'select'   => ['id', 'name', 'email'],
             * ]);
             * ```
             */
            public function executeQuery(string $collection, array $querySpec): array {}

            // ── Aggregation & Collection Methods ──────────────────

            /**
             * Aggregate a numeric field in a collection.
             *
             * @param  string        $collection Dot-notation path to the array
             * @param  string        $field      Field name to aggregate
             * @param  string        $operation  One of: `sum`, `avg`, `min`, `max`, `count`
             * @return float|int|null            Aggregated value, or null if the collection is empty
             *
             * @example
             * ```php
             * $store->aggregate('orders', 'total', 'sum');  // total revenue
             * $store->aggregate('users',  'age',   'avg');  // average age
             * ```
             */
            public function aggregate(string $collection, string $field, string $operation): mixed {}

            /**
             * Group records by a field value.
             *
             * @param  string $collection Dot-notation path to the array
             * @param  string $field      Field to group by
             * @return array<string, array> Associative array of field_value => records[]
             */
            public function groupBy(string $collection, string $field): array {}

            /**
             * Extract specific fields from all records.
             *
             * @param  string   $collection Dot-notation path to the array
             * @param  string[] $fields     Fields to extract
             * @return array
             */
            public function pluck(string $collection, array $fields): array {}

            /**
             * Extract values from a single column.
             *
             * @param  string $collection Dot-notation path to the array
             * @param  string $field      Field name to extract
             * @return array
             */
            public function column(string $collection, string $field): array {}

            /**
             * Split a collection into chunks of a given size.
             *
             * @param  string $collection Dot-notation path to the array
             * @param  int    $size       Chunk size (must be > 0)
             * @return array              Array of arrays (chunks)
             * @throws \Exception If size <= 0
             */
            public function chunk(string $collection, int $size): array {}

            /**
             * Join a column's values into a single string.
             *
             * @param  string $collection Dot-notation path to the array
             * @param  string $field      Field name
             * @param  string $separator  Separator string
             * @return string
             */
            public function implode(string $collection, string $field, string $separator): string {}

            // ── Schema Validation ─────────────────────────────────

            /**
             * Validate data at a path against a JSON Schema subset.
             *
             * **Supported schema keywords:**
             * `type`, `required`, `properties`, `minLength`, `maxLength`,
             * `format` (email, url, ipv4, date, uuid), `min`, `max`,
             * `minItems`, `maxItems`, `uniqueItems`, `items`, `enum`,
             * `if/then/else`, `oneOf`, `anyOf`
             *
             * @param  string               $path   Dot-notation path
             * @param  array<string, mixed> $schema Validation schema
             * @return array                        `{valid: bool, error_count: int, errors: array}`
             */
            public function validate(string $path, array $schema): array {}

            /**
             * Validate all items in a collection against an item schema.
             *
             * @param  string               $path       Path to the array
             * @param  array<string, mixed> $itemSchema Schema applied to each item
             * @return array                            `{valid: bool, total_items: int, valid_items: int, invalid_items: int, details: array}`
             */
            public function validateCollection(string $path, array $itemSchema): array {}

            // ── In-Memory Indexes ─────────────────────────────────

            /**
             * Create an in-memory hash index on a field for O(1) equality lookups.
             *
             * @param  string $collection Dot-notation path to the array
             * @param  string $field      Field to index
             * @return bool
             * @throws \Exception If the collection is not an array
             */
            public function createIndex(string $collection, string $field): bool {}

            /**
             * Create a compound index on multiple fields.
             *
             * @param  string   $collection Dot-notation path to the array
             * @param  string[] $fields     Fields to include in the compound index
             * @return bool
             * @throws \Exception If the collection is not an array
             */
            public function createCompoundIndex(string $collection, array $fields): bool {}

            /**
             * Perform a direct O(1) index lookup.
             *
             * @param  string $collection Dot-notation path to the array
             * @param  string $field      Indexed field
             * @param  mixed  $value      Value to look up
             * @return array|null         Matching records, or null if no index exists
             */
            public function indexLookup(string $collection, string $field, mixed $value): mixed {}

            /**
             * Create a native vector index on a field.
             *
             * @param string $collection Path to the array
             * @param string $field Field to index
             * @param array $options Options: ['dimension' => int, 'metric' => 'cosine'|'l2'|'dot']
             * @return bool
             */
            public function createVectorIndex(string $collection, string $field, array $options = []): bool {}

            /**
             * Search for items in a collection based on vector similarity.
             *
             * @param string $collection Path to the array
             * @param string $field Field containing the vector embeddings
             * @param array $queryVector Float array representation of the query embedding
             * @param int $limit Max number of matches to return
             * @param string|null $metric Optional metric override: 'cosine', 'l2', 'dot'
             * @return array Array of matching items wrapped with scores: [['score' => float, 'item' => array], ...]
             */
            public function vectorSearch(string $collection, string $field, array $queryVector, int $limit = 5, ?string $metric = null): array {}

            /**
             * List all active indexes with statistics.
             *
             * Each entry contains: `collection`, `type`, `field(s)`,
             * `unique_values`, `total_entries`.
             *
             * @return array
             */
            public function listIndexes(): array {}

            /**
             * Drop all indexes for a collection.
             *
             * @param  string $collection Dot-notation path to the array
             * @return bool               True if indexes existed and were dropped
             */
            public function dropIndex(string $collection): bool {}

            /**
             * Drop all indexes across all collections.
             *
             * @return int Number of collections whose indexes were dropped
             */
            public function dropAllIndexes(): int {}

            // ── Transactions ──────────────────────────────────────

            /**
             * Begin a transaction.
             *
             * All subsequent writes are buffered until {@see commit()} or
             * rolled back via {@see rollback()}.
             *
             * @return bool
             */
            public function beginTransaction(): bool {}

            /**
             * Commit the current transaction, persisting all buffered writes.
             *
             * @return bool
             */
            public function commit(): bool {}

            /**
             * Roll back the current transaction, discarding all buffered writes.
             *
             * @return bool
             */
            public function rollback(): bool {}

            /**
             * Check whether a transaction is currently active.
             *
             * @return bool
             */
            public function inTransaction(): bool {}

            // ── Options ───────────────────────────────────────────

            /**
             * Set a store option.
             *
             * **Available options:**
             * - `pretty`       (bool)   — Enable pretty-printing for writes.
             * - `fsync`        (bool)   — Enable fsync for crash-safe writes.
             * - `compression`  (string) — Storage compression: `"none"`, `"gzip"`, `"zstd"`.
             *
             * @param  string $key   Option name
             * @param  mixed  $value Option value
             * @return bool
             */
            public function setOption(string $key, mixed $value): bool {}

            /**
             * Get a store option value.
             *
             * @param  string $key Option name
             * @return mixed
             */
            public function getOption(string $key): mixed {}

            // ── Metrics & Observability ───────────────────────────

            /**
             * Get real-time operational metrics.
             *
             * Returns an array with:
             * `reads`, `writes`, `cache_hits`, `cache_misses`, `avg_latency_ms`
             *
             * @return array<string, mixed>
             */
            public function getMetrics(): array {}

            // ── JSONL Support ─────────────────────────────────────

            /**
             * Append a record to the JSONL file associated with this Store.
             *
             * @param  mixed $record Any JSON-serialisable value
             * @return bool
             */
            public function appendJsonl(mixed $record): bool {}

            /**
             * Read all records from the JSONL file associated with this Store.
             *
             * @return array
             */
            public function readJsonl(): array {}

            // ── Import / Export ───────────────────────────────────

            /**
             * Export the store's data as a JSON string.
             *
             * @param  bool|null $pretty Whether to pretty-print (default: false)
             * @return string
             */
            public function toJson(?bool $pretty = null): string {}

            /**
             * Import data from a JSON string, replacing the current store contents.
             *
             * @param  string $json Valid JSON string
             * @return bool
             */
            public function loadJson(string $json): bool {}

            // ── Utilities ─────────────────────────────────────────

            /**
             * Get file and data statistics.
             *
             * Returns an array with:
             * `file_path`, `file_size`, `file_size_h`, `top_level_keys`,
             * `key_count`, `active_indexes`
             *
             * @return array<string, mixed>
             */
            public function stats(): array {}

            /**
             * Create a backup of the JSON file.
             *
             * @param  string|null $backupPath Custom path (auto-timestamped if null)
             * @return string                  Path to the created backup file
             * @throws \Exception On I/O error
             */
            public function backup(?string $backupPath = null): string {}

            /**
             * Restore data from a backup file.
             *
             * @param  string $path Path to the backup file
             * @return bool
             * @throws \Exception On I/O error
             */
            public function restore(string $path): bool {}

            /**
             * Creates an isolated branch of the database.
             *
             * @param string $name Branch name.
             * @return bool True if created successfully, false if the branch already exists.
             */
            public function createBranch(string $name): bool {}

            /**
             * Switches the active connection to another branch.
             *
             * Use "main" or "master" or empty string to switch back to the original database.
             *
             * @param string $name Branch name.
             * @return bool True if switched successfully, false if the branch does not exist.
             */
            public function switchBranch(string $name): bool {}

            /**
             * Lists all available branches for this database.
             *
             * @return string[] Array of branch names.
             */
            public function listBranches(): array {}

            /**
             * Deletes a database branch and all its associated index files.
             *
             * @param string $name Branch name.
             * @return bool True if deleted successfully, false if the branch does not exist.
             */
            public function deleteBranch(string $name): bool {}

            /**
             * Merges the changes from a branch into the currently active branch using a deep merge.
             *
             * @param string $name Branch name to merge.
             * @return bool True if merged successfully.
             */
            public function mergeBranch(string $name): bool {}
        } // end class Store

    } // end if (!extension_loaded('jsonq'))

} // end namespace JsonQ