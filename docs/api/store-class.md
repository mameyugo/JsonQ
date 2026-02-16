# JsonQ\Store Class

The `JsonQ\Store` class is the main entry point for interacting with the JsonQ storage engine.

## Constructor

### `__construct(string $path)`

Create a new Store instance.

- **$path** (string): Absolute or relative path to the JSON file. If the file does not exist, it is created with `{}`.

```php
$store = new \JsonQ\Store('data.json');
```

---

## Read Operations

### `get(string $path): mixed`

Get a value by dot-notation path.

- **$path**: Dot-notation path (e.g., "users.0.name").
- **Returns**: The value at the path, or `null` if not found.

```php
$name = $store->get('users.0.name');
```

### `has(string $path): bool`

Check if a path exists in the data.

### `count(string $path): int`

Count elements at a path. Returns `-1` if the path doesn't point to an array/object.

### `keys(string $path): array`

Get top-level keys at a path.

### `values(string $path): array`

Get values of an object at a path.

---

## Write Operations

### `set(string $path, mixed $value): bool`

Set a value at a dot-notation path. Creates intermediate objects as needed.

```php
$store->set('users.0.active', true);
```

### `remove(string $path): bool`

Remove a value at a path.

### `push(string $path, mixed $value): bool`

Push a value onto an array at a path.

### `merge(string $path, mixed $value): bool`

Deep merge data into a path.

### `increment(string $path, ?float $amount = 1.0): bool`
### `decrement(string $path, ?float $amount = 1.0): bool`

Atomically increment or decrement a numeric value.

---

## Query Operations

### `find(string $collection, array $conditions): array`

Find records matching MongoDB-style conditions.

- **$collection**: Path to the array.
- **$conditions**: Array of criteria.

```php
$users = $store->find('users', ['age' => ['$gt' => 18]]);
```

### `findOne(string $collection, array $conditions): mixed`

Find the first matching record.

### `executeQuery(string $collection, array $querySpec): array`

Execute a fluent query.

```php
$results = $store->executeQuery('users', [
    'where' => [['field' => 'active', 'op' => '=', 'value' => true]],
    'limit' => 5
]);
```

---

## Aggregation & Collection Methods

### `aggregate(string $coll, string $field, string $op): mixed`

Aggregate a numeric field.
- **$op**: `sum`, `avg`, `min`, `max`, `count`.

### `groupBy(string $coll, string $field): array`

Group records by a field value.

### `pluck(string $coll, array $fields): array`

Extract specific fields from all records.

### `column(string $coll, string $field): array`

Extract values from a single column.

### `chunk(string $coll, int $size): array`

Split collection into chunks.

### `implode(string $coll, string $field, string $separator): string`

Join column values into a string.

### `except(array $fields): array` (Via Query)

Exclude specific fields from results.

---

## Validation

### `validate(string $path, array $schema): array`

Validate data against a JSON schema.

### `validateCollection(string $path, array $itemSchema): array`

Validate all items in a collection against an item schema.

---

## Indexing

### `createIndex(string $coll, string $field): bool`

Create an O(1) index on a field.

### `createCompoundIndex(string $coll, array $fields): bool`

Create a multi-field index.

### `indexLookup(string $coll, string $field, mixed $value): mixed`

Perform a direct index lookup.

### `listIndexes(): array`
### `dropIndex(string $coll): bool`
### `dropAllIndexes(): int`

---

## Transactions

### `beginTransaction(): bool`
### `commit(): bool`
### `rollback(): bool`
### `inTransaction(): bool`

---

## Utilities

### `stats(): array`
Get file and data statistics.

### `backup(?string $path = null): string`
Create a backup.

### `restore(string $path): bool`
Restore from backup.

### `getMetrics(): array`
Get real-time operational metrics.

### `appendJsonl(mixed $record): bool`
Append to JSONL file.

### `readJsonl(): array`
Read JSONL file.

### `toJson(?bool $pretty): string`
Export data to JSON string.

### `loadJson(string $json): bool`
Import data from JSON string.

---

## Options

### `setOption(string $key, mixed $value): bool`

- `pretty`: Enable pretty-printing.
- `fsync`: Enable fsync for crash safety.
- `compression`: Set storage compression (`none`, `gzip`, `zstd`).

### `getOption(string $key): mixed`
