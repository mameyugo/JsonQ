<?php
/**
 * JsonQ — High-performance JSON file storage engine for PHP
 *
 * IDE autocompletion stubs. Do not include this file at runtime.
 *
 * @package JsonQ
 * @version 0.7.0
 * @license The PHP License, version 3.01
 */

namespace JsonQ;

/**
 * High-performance JSON file storage engine backed by Rust.
 *
 * Provides file-based JSON CRUD operations, MongoDB-style queries,
 * fluent query builder, aggregation, schema validation, and in-memory indexes.
 */
class Store
{
    /**
     * Create a new Store instance.
     *
     * The file is created with `{}` if it does not exist.
     *
     * @param string $path Absolute or relative path to the JSON file
     */
    public function __construct(string $path) {}

    // ── Read Operations ──

    /**
     * Get a value by dot-notation path.
     *
     * @param string $path Dot-notation path (e.g., "users.0.name")
     * @return mixed The value at the path, or null if not found
     */
    public function get(string $path): mixed {}

    /**
     * Check if a path exists in the data.
     *
     * @param string $path Dot-notation path
     * @return bool
     */
    public function has(string $path): bool {}

    /**
     * Count elements at a path.
     *
     * @param string $path Path to an array or object
     * @return int Number of elements, or -1 if path is invalid
     */
    public function count(string $path): int {}

    /**
     * Get top-level keys at a path.
     *
     * @param string $path Path to an object (empty string for root)
     * @return string[]
     */
    public function keys(string $path): array {}

    // ── Write Operations ──

    /**
     * Set a value at a dot-notation path.
     *
     * Creates intermediate objects/arrays as needed.
     *
     * @param string $path Dot-notation path
     * @param mixed $value Value to set
     * @return bool
     * @throws \Exception If the store is not initialized
     */
    public function set(string $path, mixed $value): bool {}

    /**
     * Remove a value at a path.
     *
     * @param string $path Dot-notation path
     * @return bool
     * @throws \Exception If the store is not initialized
     */
    public function remove(string $path): bool {}

    /**
     * Push a value onto an array at a path.
     *
     * @param string $path Path to the target array
     * @param mixed $value Value to append
     * @return bool True if pushed, false if target is not an array
     * @throws \Exception If the store is not initialized
     */
    public function push(string $path, mixed $value): bool {}

    /**
     * Deep merge data into a path.
     *
     * For objects, keys are merged recursively. For other types, the value is replaced.
     *
     * @param string $path Dot-notation path
     * @param mixed $value Data to merge
     * @return bool
     * @throws \Exception If the store is not initialized
     */
    public function merge(string $path, mixed $value): bool {}

    /**
     * Increment a numeric value.
     *
     * @param string $path Path to a numeric value
     * @param float|null $amount Amount to increment (default: 1.0)
     * @return bool
     * @throws \Exception If the store is not initialized
     */
    public function increment(string $path, ?float $amount = null): bool {}

    /**
     * Decrement a numeric value.
     *
     * @param string $path Path to a numeric value
     * @param float|null $amount Amount to decrement (default: 1.0)
     * @return bool
     * @throws \Exception If the store is not initialized
     */
    public function decrement(string $path, ?float $amount = null): bool {}

    // ── Query Operations ──

    /**
     * Find records matching MongoDB-style conditions.
     *
     * Automatically uses indexes when available for simple equality queries.
     *
     * Supported operators:
     * - Comparison: $eq, $ne, $gt, $gte, $lt, $lte
     * - Array: $in, $nin
     * - String: $regex, $contains, $startsWith, $endsWith
     * - Type: $exists, $size, $type
     * - Logical: $and, $or, $not
     *
     * @param string $collection Dot-notation path to the array
     * @param array $conditions MongoDB-style filter conditions
     * @return array Matching records
     */
    public function find(string $collection, array $conditions): array {}

    /**
     * Find the first record matching conditions.
     *
     * @param string $collection Dot-notation path to the array
     * @param array $conditions MongoDB-style filter conditions
     * @return array|null First matching record, or null
     */
    public function findOne(string $collection, array $conditions): mixed {}

    /**
     * Execute a fluent query specification.
     *
     * Query spec keys:
     * - where: array of conditions [{field, op, value}, ...]
     * - order_by: {field, direction: "asc"|"desc"}
     * - limit: int
     * - offset: int
     * - select: string[] (fields to return)
     *
     * Supported operators: =, !=, <>, >, >=, <, <=, in, not in,
     * contains, starts_with, ends_with, between
     *
     * @param string $collection Dot-notation path to the array
     * @param array $querySpec Fluent query specification
     * @return array Matching records
     */
    public function executeQuery(string $collection, array $querySpec): array {}

    // ── Aggregation ──

    /**
     * Aggregate a numeric field.
     *
     * @param string $collection Path to the array
     * @param string $field Field name to aggregate
     * @param string $operation One of: sum, avg, min, max, count
     * @return float|int|null Aggregated value, or null if no data
     */
    public function aggregate(string $collection, string $field, string $operation): mixed {}

    /**
     * Group records by a field value.
     *
     * @param string $collection Path to the array
     * @param string $field Field to group by
     * @return array Associative array of field_value => records[]
     */
    public function groupBy(string $collection, string $field): array {}

    /**
     * Extract specific fields from all records.
     *
     * @param string $collection Path to the array
     * @param string[] $fields Fields to extract
     * @return array Array of extracted values
     */
    public function pluck(string $collection, array $fields): array {}

    /**
     * Extract values from a single column.
     *
     * @param string $collection Collection path
     * @param string $field Field name to extract
     * @return array Array of values from that field
     */
    public function column(string $collection, string $field): array {}

    /**
     * Split collection into chunks.
     *
     * @param string $collection Collection path
     * @param int $size Size of each chunk (must be > 0)
     * @return array Array of arrays (chunks)
     * @throws \Exception If size <= 0
     */
    public function chunk(string $collection, int $size): array {}

    /**
     * Join column values into a string.
     *
     * @param string $collection Collection path
     * @param string $field Field name
     * @param string $separator Separator string
     * @return string Joined string
     */
    public function implode(string $collection, string $field, string $separator): string {}

    /**
     * Get values of an object at path.
     *
     * @param string $path Dot-notation path
     * @return array Array of values
     */
    public function values(string $path): array {}

    // ── Validation ──

    /**
     * Validate data at a path against a JSON schema.
     *
     * @param string $path Dot-notation path
     * @param array $schema Validation schema
     * @return array {valid: bool, error_count: int, errors: array}
     */
    public function validate(string $path, array $schema): array {}

    /**
     * Validate all items in a collection against an item schema.
     *
     * @param string $path Path to the array
     * @param array $itemSchema Schema for each item
     * @return array {valid: bool, total_items: int, valid_items: int, invalid_items: int, details: array}
     */
    public function validateCollection(string $path, array $itemSchema): array {}

    // ── Indexes ──

    /**
     * Create an in-memory index on a field for O(1) equality lookups.
     *
     * @param string $collection Path to the array
     * @param string $field Field to index
     * @return bool
     * @throws \Exception If collection is not an array
     */
    public function createIndex(string $collection, string $field): bool {}

    /**
     * Create a compound index on multiple fields.
     *
     * @param string $collection Path to the array
     * @param string[] $fields Fields to include in the compound index
     * @return bool
     * @throws \Exception If collection is not an array
     */
    public function createCompoundIndex(string $collection, array $fields): bool {}

    /**
     * Perform a direct O(1) index lookup.
     *
     * @param string $collection Path to the array
     * @param string $field Indexed field
     * @param mixed $value Value to look up
     * @return array|null Matching records, or null if index not found
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
     * @return array Array of index info: [{collection, type, field(s), unique_values, total_entries}]
     */
    public function listIndexes(): array {}

    /**
     * Drop all indexes for a collection.
     *
     * @param string $collection Path to the array
     * @return bool True if indexes existed and were dropped
     */
    public function dropIndex(string $collection): bool {}

    /**
     * Drop all indexes across all collections.
     *
     * @return int Number of collections whose indexes were dropped
     */
    public function dropAllIndexes(): int {}

    // ── Metrics & Observability ──

    /**
     * Get real-time operational metrics and statistics.
     *
     * @return array {reads, writes, cache_hits, cache_misses, avg_latency_ms}
     */
    public function getMetrics(): array {}

    // ── Utilities ──

    /**
     * Get file and data statistics.
     *
     * @return array {file_path, file_size, file_size_h, top_level_keys, key_count, active_indexes}
     */
    public function stats(): array {}

    /**
     * Create a backup of the JSON file.
     *
     * @param string|null $backupPath Custom backup path (auto-timestamped if null)
     * @return string Path to the created backup file
     * @throws \Exception On I/O error
     */
    public function backup(?string $backupPath = null): string {}

    /**
     * Restore data from a backup file.
     *
     * @param string $backupPath Path to the backup file
     * @return bool
     * @throws \Exception On I/O error
     */
    public function restore(string $backupPath): bool {}

    // ── Options ──

    /**
     * Set a store option.
     *
     * Options:
     * - "pretty" (bool): Enable pretty-printed JSON output (default: false)
     * - "fsync" (bool): Enable fsync on writes for crash safety (default: false)
     * - "compression" (string): Set storage compression ("none", "gzip", "zstd") (default: "none")
     *
     * @param string $key Option name
     * @param mixed $value Option value
     * @return bool True if option was recognized and set
     */
    public function setOption(string $key, mixed $value): bool {}

    /**
     * Get a store option value.
     *
     * @param string $key Option name
     * @return mixed Option value, or null if not recognized
     */
    public function getOption(string $key): mixed {}

    // ── Transactions ──

    /**
     * Begin a transaction. Changes are buffered in memory until commit.
     *
     * @return bool
     * @throws \Exception If the store is not initialized
     */
    public function beginTransaction(): bool {}

    /**
     * Commit the current transaction. Flushes all buffered changes to disk.
     *
     * @return bool
     * @throws \Exception If no active transaction
     */
    public function commit(): bool {}

    /**
     * Rollback the current transaction. Discards all buffered changes.
     *
     * @return bool
     */
    public function rollback(): bool {}

    /**
     * Check if a transaction is active.
     *
     * @return bool
     */
    public function inTransaction(): bool {}

    // ── Batch Operations ──

    /**
     * Set multiple key-value pairs in a single write operation.
     *
     * @param array $pairs Associative array of path => value pairs
     * @return int Number of keys set
     * @throws \Exception If the store is not initialized
     */
    public function setMany(array $pairs): int {}

    /**
     * Remove multiple paths in a single write operation.
     *
     * @param string[] $paths Array of dot-notation paths to remove
     * @return int Number of paths actually removed
     * @throws \Exception If the store is not initialized
     */
    public function removeMany(array $paths): int {}

    // ── Import/Export ──

    /**
     * Export all data as a JSON string.
     *
     * @param bool|null $pretty Whether to pretty-print (default: false)
     * @return string JSON string
     */
    public function toJson(?bool $pretty = null): string {}

    /**
     * Import data from a JSON string, replacing all existing data.
     *
     * @param string $jsonStr Valid JSON string
     * @return bool
     * @throws \Exception On invalid JSON
     */
    public function fromJson(string $jsonStr): bool {}

    // ── Extra ──

    /**
     * Get all data from the store.
     *
     * @return mixed The entire root data structure
     */
    public function getAll(): mixed {}

    /**
     * Clear all data, resetting to empty object {}.
     *
     * @return bool
     * @throws \Exception If the store is not initialized
     */
    public function clear(): bool {}

    /**
     * Full-text search across all string fields in a collection.
     *
     * Case-insensitive, searches recursively through nested objects and arrays.
     *
     * @param string $collection Path to the array to search
     * @param string $keyword Search keyword
     * @return array Matching records
     */
    public function search(string $collection, string $keyword): array {}

    /**
     * Append a record to a JSONL file.
     *
     * @param mixed $record Data to append
     * @return bool
     */
    public function appendJsonl(mixed $record): bool {}

    /**
     * Read all records from a JSONL file.
     *
     * @return string[] Array of JSON strings
     */
    public function readJsonl(): array {}
    /**
     * Stream items from a JSON array at the given JSON Pointer path.
     *
     * @param string $pointer    JSON Pointer (RFC 6901), e.g. "/users"
     * @param array|null  $conditions MongoDB-style conditions (optional)
     * @param array|null  $options    ['limit' => int, 'skip' => int, 'select' => string[], 'pretty' => bool]
     * @return array
     */
    public function stream(string $pointer, ?array $conditions = null, ?array $options = null): array {}

    /**
     * @param string $pointer
     * @param array|null  $conditions
     * @return int
     */
    public function streamCount(string $pointer, ?array $conditions = null): int {}

    /**
     * @param string $pointer
     * @param string $outputPath
     * @param array|null  $conditions
     * @param array|null  $options ['limit', 'skip', 'select', 'pretty']
     * @return int Items written
     */
    public function streamToFile(string $pointer, string $outputPath, ?array $conditions = null, ?array $options = null): int {}

    /**
     * @param string $pointer
     * @param string $operation 'sum'|'avg'|'min'|'max'|'count'
     * @param string $field
     * @param array|null  $conditions
     * @return float|int
     */
    public function streamAggregate(string $pointer, string $operation, string $field, ?array $conditions = null): float|int {}

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
}
