<?php
/**
 * JsonQ — High-performance JSON file storage engine for PHP
 *
 * IDE autocompletion stubs. Do not include this file at runtime.
 *
 * @package JsonQ
 * @version 0.3.1
 * @license The PHP License, version 3.01
 */

/**
 * Get the JsonQ extension version.
 *
 * @return string Semantic version string (e.g., "0.3.1")
 */
function jsonq_version(): string {}

/**
 * Get current global configuration.
 *
 * Returns an array with:
 * - max_file_size: int (bytes)
 * - max_file_size_mb: float
 * - max_validation_depth: int
 * - max_path_depth: int
 * - allowed_extensions: string[]
 * - base_path: ?string
 *
 * @return array
 */
function jsonq_get_config(): array {}

/**
 * Set maximum file size for all Store instances.
 *
 * Supports units like "100M", "1G", "500K".
 *
 * @param string $size Format: [number][unit]
 * @return bool
 */
function jsonq_set_max_file_size(string $size): bool {}

/**
 * Set allowed file extensions for Store instances.
 *
 * @param string $extensions Comma-separated list (e.g., "json,db,data")
 * @return bool
 */
function jsonq_set_allowed_extensions(string $extensions): bool {}

/**
 * Set base path restriction for all file operations.
 *
 * All Store paths must be within this base path (security sandbox).
 *
 * @param string $path Absolute base directory path
 * @return bool
 */
function jsonq_set_base_path(string $path): bool {}

/**
 * Clear the base path restriction.
 *
 * @return bool
 */
function jsonq_clear_base_path(): bool {}

/**
 * Write a Store's data directly to a file using memory-efficient streaming.
 *
 * @param string $path Source JSON file path
 * @param string $output_path Target output file path
 * @param bool|null $pretty Whether to pretty-print (default: false)
 * @return bool
 */
function jsonq_write_to_file(string $path, string $output_path, ?bool $pretty = null): bool {}

/**
 * Append a record to a JSONL (Line-delimited JSON) file.
 *
 * Validates that the record is valid JSON before appending.
 *
 * @param string $path JSONL file path
 * @param string $record JSON string record
 * @return bool
 */
function jsonq_append_jsonl(string $path, string $record): bool {}

/**
 * Read all records from a JSONL file.
 *
 * @param string $path JSONL file path
 * @return string[] Array of JSON string records
 */
function jsonq_read_jsonl(string $path): array {}

/**
 * Get memory optimization and key deduplication statistics.
 *
 * Returns an array with:
 * - unique_keys: int
 * - total_references: int
 * - memory_saved_percent: int
 *
 * @param string $path JSON file path
 * @return array
 */
function jsonq_memory_stats(string $path): array {}

/**
 * Query a JSON file directly using JSONPath without fully loading it into PHP.
 *
 * @param string $path JSON file path
 * @param string $query_path JSONPath query (e.g., "$.users[*].name")
 * @return string[] Array of serialized JSON results
 */
function jsonq_query_node(string $path, string $query_path): array {}

/**
 * Alias for jsonq_query_node.
 *
 * @param string $path JSON file path
 * @param string $query JSONPath query
 * @return string[]
 */
function jsonq_query(string $path, string $query): array {}
