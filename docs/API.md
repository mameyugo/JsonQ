# JsonQ API Reference (v0.6.0)

Complete reference for all `JsonQ\\Store` methods and the `jsonq_version()` function.

## Table of Contents

- [Constructor](#constructor)
- [Read Operations](#read-operations)
- [Streaming Operations](#streaming-operations)
- [Write Operations](#write-operations)
- [Query Operations](#query-operations)
- [Aggregation](#aggregation)
- [Schema Validation](#schema-validation)
- [Indexes](#indexes)
- [Utilities](#utilities)
- [Standalone Functions](#standalone-functions)

---

## Constructor

### `new JsonQ\\Store(string $path)`

Creates a new store instance bound to a JSON file. If the file does not exist, it is created with `{}`.

```php
$store = new JsonQ\\Store('/var/data/app.json');
```

**Parameters:**
- `$path` — Absolute or relative path to the JSON file

**Note:** Each `Store` instance maintains its own in-memory cache and index set. Multiple instances pointing to the same file will share data through the filesystem but have independent caches.

---

## Read Operations

### `get(string $path): mixed`

Retrieves a value using dot-notation path traversal.

```php
$store->get('users');           // Entire array
$store->get('users.0.name');    // First user's name
$store->get('config.db.host');  // Nested object value
```

Returns `null` if the path does not exist. Supports traversal into arrays by numeric index.

### `has(string $path): bool`

Checks whether a value exists at the given path.

```php
if ($store->has('config.debug')) {
    // ...
}
```

### `count(string $path): int`

Returns the number of elements in an array or object at the path. Returns `-1` if the path does not point to an array or object.

```php
$store->count('users');    // Number of users
$store->count('config');   // Number of config keys
```

### `keys(string $path): string[]`

Returns the top-level keys of an object at the given path. Pass an empty string for root keys.

```php
$store->keys('');        // ['users', 'config', ...]
$store->keys('config');  // ['debug', 'version', ...]
```

---

## Streaming Operations

### `stream(string $pointer, ?array $conditions = null, ?array $options = null): array`

Streams items from a JSON array at a specific pointer, applying optional filters. Extremely memory-efficient.

```php
// Stream all users
$users = $store->stream('/users');

// Stream with filter and options
$admins = $store->stream('/users',
    ['role' => 'admin'],
    ['limit' => 100, 'select' => ['id', 'email']]
);
```

### `streamAs(string $class, string $pointer, ?array $conditions = null, ?array $options = null): array`

*(HydratableStore only)* Similar to `stream()`, but returns an array of hydrated PHP objects.

```php
$admins = $hydratableStore->streamAs(User::class, '/users');
```

**Options:**
- `limit` (int): Max items to return.
- `skip` (int): Items to skip.
- `select` (string[]): Fields to include.

**Note (v0.4.1):** If the JSON Pointer targets an object instead of an array, `stream()` gracefully returns an empty stream instead of throwing a format error.

### `streamCount(string $pointer, ?array $conditions = null): int`

Counts items in a stream without loading them all into memory.

```php
$count = $store->streamCount('/logs', ['level' => 'error']);
```

### `streamToFile(string $pointer, string $outputPath, ?array $conditions = null, ?array $options = null): int`

Streams filtered items directly to a new file.

```php
$store->streamToFile('/users', '/tmp/admins.json', ['role' => 'admin']);
```

### `streamAggregate(string $pointer, string $op, string $field, ?array $conditions = null): number`

Performs aggregation on a stream.
**Ops:** `sum`, `avg`, `min`, `max`, `count`.

```php
$total = $store->streamAggregate('/orders', 'sum', 'amount');
```

### `\JsonQ\StreamFilter::apply(mixed $item): mixed`

If you are using the native stream filter wrapper directly, you can apply the condition/projection logic to a single decoded item.

```php
$filter = new \JsonQ\StreamFilter(['age' => ['$gt' => 18]], ['name', 'age']);
$result = $filter->apply(['name' => 'Alice', 'age' => 20, 'role' => 'admin']);
// Returns: ['name' => 'Alice', 'age' => 20]
```

---

## Write Operations

### `set(string $path, mixed $value): bool`

Sets a value at a dot-notation path. Automatically creates intermediate objects or arrays.

```php
$store->set('users', [...]);
$store->set('config.db.host', 'localhost');  // Creates config and config.db
$store->set('items.0.name', 'Widget');       // Creates items array if needed
```

**Atomic write:** Uses tmp file → fsync → rename to prevent corruption.

### `remove(string $path): bool`

Removes the value at the given path.

```php
$store->remove('config.debug');  // Remove a key
$store->remove('users.0');       // Remove first array element
```

### `push(string $path, mixed $value): bool`

Appends a value to an array. Returns `false` if the target is not an array.

```php
$store->push('users', ['name' => 'New User', 'age' => 25]);
$store->push('tags', 'new-tag');
```

### `setObject(string $path, object $obj, array $ignore = []): bool`

*(HydratableStore only)* Serializes a PHP object and sets it at the given dot-notation path.

```php
$hydratableStore->setObject('profile', new User(1, 'Alice'));
```

### `pushObject(string $path, object $obj, array $ignore = []): bool`

*(HydratableStore only)* Serializes a PHP object and pushes it to an array.

```php
$hydratableStore->pushObject('users', new User(1, 'Alice'));
```

### `merge(string $path, mixed $value): bool`

Deep merges an object into the existing value. For non-object values, replaces the value.

```php
$store->merge('config', [
    'cache' => ['ttl' => 3600],
    'debug' => true,
]);
// Existing config keys are preserved; matching keys are recursively merged
```

### `increment(string $path, ?float $amount = null): bool`

Increments a numeric value. Default amount is `1.0`.

```php
$store->increment('stats.views');       // +1
$store->increment('stats.views', 10);   // +10
```

### `decrement(string $path, ?float $amount = null): bool`

Decrements a numeric value. Default amount is `1.0`.

```php
$store->decrement('stock.quantity');     // -1
$store->decrement('stock.quantity', 5); // -5
```

---

## Query Operations

### `find(string $collection, array $conditions): array`

Finds all records in a collection matching MongoDB-style conditions. Automatically uses indexes for simple equality queries when available.

```php
// Simple equality
$store->find('users', ['role' => 'admin']);

// Comparison operators
$store->find('users', ['age' => ['$gte' => 18, '$lt' => 65]]);

// Logical operators
$store->find('users', [
    '$or' => [
        ['city' => 'NYC'],
        ['city' => 'LA'],
    ]
]);
$store->find('users', ['$not' => ['role' => 'viewer']]);

// String operators
$store->find('users', ['name' => ['$startsWith' => 'A']]);
$store->find('users', ['email' => ['$contains' => '@gmail']]);

// Type/existence operators
$store->find('users', ['phone' => ['$exists' => true]]);
$store->find('users', ['tags' => ['$size' => 3]]);
```

**Operator Reference:**

| Operator | Description | Example |
|----------|-------------|---------|
| `$eq` | Equal to | `['age' => ['$eq' => 30]]` |
| `$ne` | Not equal to | `['role' => ['$ne' => 'admin']]` |
| `$gt` | Greater than | `['age' => ['$gt' => 18]]` |
| `$gte` | Greater than or equal | `['score' => ['$gte' => 90]]` |
| `$lt` | Less than | `['age' => ['$lt' => 30]]` |
| `$lte` | Less than or equal | `['price' => ['$lte' => 99.99]]` |
| `$in` | In array | `['role' => ['$in' => ['admin', 'mod']]]` |
| `$nin` | Not in array | `['status' => ['$nin' => ['banned']]]` |
| `$contains` | String contains | `['bio' => ['$contains' => 'PHP']]` |
| `$startsWith` | String starts with | `['name' => ['$startsWith' => 'A']]` |
| `$endsWith` | String ends with | `['email' => ['$endsWith' => '.com']]` |
| `$exists` | Field exists (not null) | `['phone' => ['$exists' => true]]` |
| `$size` | Array length equals | `['tags' => ['$size' => 5]]` |
| `$type` | Value type matches | `['age' => ['$type' => 'integer']]` |
| `$and` | All conditions match | `['$and' => [{...}, {...}]]` |
| `$or` | Any condition matches | `['$or' => [{...}, {...}]]` |
| `$not` | Negate conditions | `['$not' => ['role' => 'admin']]` |

### `findOne(string $collection, array $conditions): mixed`

Returns the first matching record, or `null` if none match.

```php
$user = $store->findOne('users', ['email' => 'alice@example.com']);
```

### `findOneAs(string $class, string $collection, array $conditions): ?object`

*(HydratableStore only)* Works like `findOne` but returns a strongly-typed hydrated PHP object.

```php
$user = $hydratableStore->findOneAs(User::class, 'users', ['email' => 'alice@example.com']);
```

### `findInAs(string $class, string $collection, array $conditions): array`

*(HydratableStore only)* Works like `find` but returns an array of hydrated PHP objects.

```php
$admins = $hydratableStore->findInAs(User::class, 'users', ['role' => 'admin']);
```

### `executeQuery(string $collection, array $querySpec): array`

Executes a fluent query specification with filtering, sorting, pagination, and projection.

```php
$store->executeQuery('users', [
    'where'    => [
        ['field' => 'status', 'op' => '=', 'value' => 'active'],
        ['field' => 'age', 'op' => 'between', 'value' => [18, 65]],
    ],
    'order_by' => ['field' => 'created_at', 'direction' => 'desc'],
    'offset'   => 20,
    'limit'    => 10,
    'select'   => ['name', 'email', 'age'],
]);
```

**Query Spec Keys:**

| Key | Type | Description |
|-----|------|-------------|
| `where` | `array` | Array of `{field, op, value}` conditions (all must match) |
| `order_by` | `object` | `{field: string, direction: "asc"\|"desc"}` |
| `limit` | `int` | Maximum records to return |
| `offset` | `int` | Number of records to skip |
| `select` | `string[]` | Fields to include in results (projection) |

**Fluent Operators:** `=`, `==`, `!=`, `<>`, `>`, `>=`, `<`, `<=`, `in`, `not in`, `contains`, `starts_with`, `ends_with`, `between`

---

## Aggregation

### `aggregate(string $collection, string $field, string $operation): mixed`

Performs an aggregation operation on a numeric field.

```php
$store->aggregate('orders', 'total', 'sum');   // 15420.50
$store->aggregate('users', 'age', 'avg');      // 28.5
$store->aggregate('products', 'price', 'min'); // 9.99
$store->aggregate('products', 'price', 'max'); // 299.99
$store->aggregate('users', 'id', 'count');     // 150
```

**Operations:** `sum`, `avg`, `min`, `max`, `count`

### `groupBy(string $collection, string $field): array`

Groups records by a field value. Returns an associative array where keys are the distinct field values and values are arrays of matching records.

```php
$byCity = $store->groupBy('users', 'city');
// ['NYC' => [{...}, {...}], 'LA' => [{...}], ...]
```

### `pluck(string $collection, array $fields): array`

Extracts specific fields from all records.

```php
// Single field — returns flat array of values
$names = $store->pluck('users', ['name']);
// ['Alice', 'Bob', 'Charlie']

// Multiple fields — returns array of objects
$info = $store->pluck('users', ['name', 'email']);
// [['name' => 'Alice', 'email' => '...'], ...]
```

---

## Schema Validation

### `validate(string $path, array $schema): array`

Validates the data at a path against a JSON Schema specification.

```php
$result = $store->validate('user', [
    'type'       => 'object',
    'required'   => ['name', 'email'],
    'properties' => [
        'name'  => ['type' => 'string', 'minLength' => 1],
        'email' => ['type' => 'string', 'format' => 'email'],
        'age'   => ['type' => 'integer', 'min' => 0, 'max' => 150],
    ],
    'additionalProperties' => false,
]);
```

**Return format:**
```php
[
    'valid'       => true|false,
    'error_count' => 0,
    'errors'      => [
        ['path' => 'user.email', 'error' => "Invalid format: 'email'", 'code' => 'FORMAT_INVALID'],
    ]
]
```

**Schema Keywords:**

| Keyword | Applies to | Description |
|---------|-----------|-------------|
| `type` | All | `string`, `integer`, `number`, `boolean`, `array`, `object`, `null`, `any` |
| `nullable` | All | Allow null values |
| `required` | Object | Array of required field names |
| `properties` | Object | Schema for each property |
| `additionalProperties` | Object | `false` to disallow extra keys |
| `minLength` / `maxLength` | String | Length constraints |
| `format` | String | `email`, `url`, `uri`, `ipv4`, `date`, `uuid` |
| `min` / `max` | Number | Value range |
| `minItems` / `maxItems` | Array | Length constraints |
| `uniqueItems` | Array | Require unique elements |
| `items` | Array | Schema for array elements |
| `enum` | All | Allowed values list |
| `oneOf` | All | Exactly one schema must match |
| `anyOf` | All | At least one schema must match |
| `if` / `then` / `else` | All | Conditional validation |

### `validateCollection(string $path, array $itemSchema): array`

Validates every item in an array against an item schema.

```php
$result = $store->validateCollection('users', [
    'type'     => 'object',
    'required' => ['name', 'email'],
]);
```

**Return format:**
```php
[
    'valid'         => false,
    'total_items'   => 100,
    'valid_items'   => 97,
    'invalid_items' => 3,
    'details'       => [
        ['index' => 12, 'errors' => [...]],
        ['index' => 45, 'errors' => [...]],
        ['index' => 88, 'errors' => [...]],
    ]
]
```

---

## Indexes

### `createIndex(string $collection, string $field): bool`

Creates an in-memory hash index on a single field for O(1) equality lookups.

```php
$store->createIndex('users', 'email');
```

**Note:** Indexes are invalidated automatically when data is written. Rebuild after bulk writes for best performance.

### `createCompoundIndex(string $collection, array $fields): bool`

Creates a compound index on multiple fields.

```php
$store->createCompoundIndex('users', ['city', 'role']);
```

### `indexLookup(string $collection, string $field, mixed $value): mixed`

Performs a direct O(1) hash lookup against an existing index. Returns `null` if the index doesn't exist or is stale.

```php
$store->createIndex('users', 'email');
$user = $store->indexLookup('users', 'email', 'alice@example.com');
```

### `listIndexes(): array`

Returns metadata for all active indexes.

```php
$indexes = $store->listIndexes();
// [
//   ['collection' => 'users', 'type' => 'single', 'field' => 'email', 'unique_values' => 100, 'total_entries' => 100],
//   ['collection' => 'users', 'type' => 'compound', 'fields' => 'city+role', 'unique_values' => 15, 'total_entries' => 100],
// ]
```

### `dropIndex(string $collection): bool`

Drops all indexes for a collection. Returns `true` if indexes existed.

### `dropAllIndexes(): int`

Drops all indexes across all collections. Returns the number of collections affected.

---

## Utilities

### `stats(): array`

Returns file and data statistics.

```php
$stats = $store->stats();
// [
//   'file_path'      => '/path/to/data.json',
//   'file_size'      => 15420,
//   'file_size_h'    => '15.06 KB',
//   'top_level_keys' => ['users', 'config'],
//   'key_count'      => 2,
//   'active_indexes' => 3,
// ]
```

### `backup(?string $backupPath = null): string`

Creates a copy of the JSON file. If no path is provided, creates an auto-timestamped backup next to the original file.

```php
$path = $store->backup();                    // data.json.backup.1707350400
$path = $store->backup('/backups/snap.json'); // Custom path
```

### `restore(string $backupPath): bool`

Restores data from a backup file. Clears all caches and indexes.

```php
$store->restore('/backups/snap.json');
```

---

## Standalone Functions

### `jsonq_version(): string`

Returns the extension version string.

```php
echo jsonq_version(); // "0.6.0"
```
