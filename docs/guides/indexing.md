# Indexing & Performance

JsonQ is fast by default, but appropriate indexing can make it orders of magnitude faster (up to 125x for lookups).

## How Indexes Work

Without an index, JsonQ must scan every record in a collection to find matches (O(n)).
With an index, JsonQ creates a hash map in memory that points directly to the record's location (O(1)).

## Creating Indexes

Use `createIndex()` to index a specific field.

```php
// Index the 'email' field in the 'users' collection
$store->createIndex('users', 'email');
```

This operation:
1. Scans the 'users' collection.
2. Builds an in-memory hash map of `email -> index`.
3. Speeds up subsequent queries.

> **Note:** Indexes are currently **in-memory only** and must be recreated when the `Store` is re-initialized (e.g., in a new PHP request). For long-running processes (workers, daemons), create them once at startup.

## Compound Indexes

You can index multiple fields together for queries that filter on both.

```php
// Index 'role' and 'status' together
$store->createCompoundIndex('users', ['role', 'status']);
```

This speeds up queries like:
```php
$store->find('users', [
    'role' => 'admin',
    'status' => 'active'
]);
```

## Using Indexes

You don't need to change your queries. The query optimizer automatically detects if an index exists for the fields you are querying and uses it.

```php
// If 'email' is indexed, this becomes O(1) instead of O(n)
$user = $store->find('users', ['email' => 'alice@example.com']);
```

### Direct Index Lookup

For maximum performance, you can perform a direct lookup if you know the index exists:

```php
$user = $store->indexLookup('users', 'email', 'alice@example.com');
```

## Managing Indexes

### List Indexes

Check which indexes are currently active.

```php
$indexes = $store->listIndexes();
print_r($indexes);
```

### Drop Indexes

Remove an index to free up memory.

```php
// Drop index on 'email'
$store->dropIndex('users.email'); // Note dot notation or collection/field args depending on implementation

// Drop all indexes
$store->dropAllIndexes();
```

## Performance Tips

1. **Index High-Cardinally Fields**: Fields with many unique values (IDs, emails, usernames) benefit most from indexing.
2. **Index Low-Cardinality Fields for Grouping**: Improve `groupBy` performance by indexing the grouping field.
3. **Avoid Over-Indexing**: Indexes consume memory. Only index fields that are frequently queried.
4. **Use `findOne`**: If you only expect one result, us `findOne()` to stop scanning after the first match (unless indexed, where it's instant anyway).
