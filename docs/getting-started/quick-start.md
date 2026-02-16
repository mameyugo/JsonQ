# Quick Start

In this guide, you will learn how to create a store, save data, and perform queries in less than 5 minutes.

## 1. Create a Store

The `JsonQ\Store` class is the main entry point. It maps to a JSON file on disk.

```php
<?php
// Initialize the store
// If 'data.json' doesn't exist, it will be created automatically.
$store = new \JsonQ\Store(__DIR__ . '/data.json');

// Clear any existing data for this example
$store->clear();
```

## 2. Insert Data

You can write data using dot-notation paths.

```php
// Set a single value
$store->set('app_name', 'My Awesome App');

// Set an array of objects (e.g., users)
$users = [
    ['id' => 1, 'name' => 'Alice', 'role' => 'admin', 'age' => 30],
    ['id' => 2, 'name' => 'Bob', 'role' => 'editor', 'age' => 25],
    ['id' => 3, 'name' => 'Charlie', 'role' => 'user', 'age' => 35],
];

$store->set('users', $users);

echo "Data saved!\n";
```

## 3. Read Data

Retrieve data using the same dot-notation paths.

```php
$appName = $store->get('app_name');
echo "App: $appName\n"; // Output: App: My Awesome App

// Get a specific user's name
$firstUser = $store->get('users.0.name');
echo "First User: $firstUser\n"; // Output: First User: Alice
```

## 4. Query Data

Use MongoDB-style queries to find specific records.

```php
// Find all admins
$admins = $store->find('users', [
    'role' => 'admin'
]);

print_r($admins);
/*
Array
(
    [0] => Array
        (
            [id] => 1
            [name] => Alice
            [role] => admin
            [age] => 30
        )
)
*/

// Find users older than 28
$olderUsers = $store->find('users', [
    'age' => ['$gt' => 28]
]);
```

## 5. Aggregation

Calculate statistics instantly.

```php
// Calculate average age
$avgAge = $store->aggregate('users', 'age', 'avg');
echo "Average Age: $avgAge\n"; // Output: 30

// Count users
$count = $store->count('users');
echo "Total Users: $count\n"; // Output: 3
```

## Next Steps

Now that you know the basics, explore the advanced features:

- [Detailed Query Guide](../guides/queries.md)
- [Schema Validation](../guides/schema-validation.md)
- [Transaction Management](../guides/transactions.md)
