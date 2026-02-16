# Transactions

JsonQ supports ACID transactions, ensuring that multiple operations are either all applied or none at all. This is critical for data integrity when performing complex updates.

## ACID Guarantees

- **Atomic**: Operations within a transaction are all-or-nothing.
- **Consistent**: The data moves from one valid state to another.
- **Isolated**: Changes are not visible to other processes until committed.
- **Durable**: Committed changes are permanently written to disk (using `fsync` if enabled).

## Usage

### Basic Flow

```php
try {
    // 1. Start the transaction
    $store->beginTransaction();

    // 2. Perform operations
    $store->set('user.balance', 100);
    $store->set('user.status', 'active');

    // These changes are currently in memory only
    // and invisible to other readers.

    // 3. Commit changes to disk
    $store->commit();

} catch (Exception $e) {
    // 4. Rollback in case of error
    $store->rollback();
    echo "Transaction failed: " . $e->getMessage();
}
```

### Checking Transaction State

```php
if ($store->inTransaction()) {
    echo "Transaction is active";
}
```

## How It Works

1. **Begin**: JsonQ creates a lightweight in-memory snapshot layer.
2. **Write**: Updates are applied to this layer.
3. **Commit**: The snapshot is merged with the main data, and the file is atomically rewritten.
4. **Rollback**: The snapshot layer is discarded.

> **Note:** Nested transactions are not currently supported. Calling `beginTransaction()` while a transaction is already active will throw an exception.
