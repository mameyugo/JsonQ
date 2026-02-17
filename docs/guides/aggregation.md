# Aggregation Guide

Learn how to perform aggregations and data analysis with JsonQ.

## Overview

JsonQ provides powerful aggregation functions to analyze and summarize your data. You can perform operations like sum, average, min, max, count, grouping, and field extraction.

---

## Basic Aggregation Functions

### Single Aggregation

Perform a single aggregation operation on a numeric field:

```php
use JsonQ\Store;

$store = new Store('data.json');

// Sum all order totals
$total = $store->aggregate('orders', ['sum' => 'amount']);
// Returns: ['sum' => 15750.50]

// Calculate average user age
$avgAge = $store->aggregate('users', ['avg' => 'age']);
// Returns: ['avg' => 28.5]

// Find minimum product price
$minPrice = $store->aggregate('products', ['min' => 'price']);
// Returns: ['min' => 9.99]

// Find maximum score
$maxScore = $store->aggregate('scores', ['max' => 'points']);
// Returns: ['max' => 985]

// Count total users
$userCount = $store->aggregate('users', ['count' => 'id']);
// Returns: ['count' => 150]
```

### Multiple Aggregations

Perform multiple aggregations in a single call:

```php
// Get comprehensive product statistics
$stats = $store->aggregate('products', [
    'sum' => 'price',
    'avg' => 'price',
    'min' => 'price',
    'max' => 'price',
    'count' => 'id'
]);

// Returns:
// [
//     'sum' => 5000.00,
//     'avg' => 250.00,
//     'min' => 50.00,
//     'max' => 1000.00,
//     'count' => 20
// ]
```

---

## Aggregation Functions

### `sum` - Sum Values

Calculate the sum of numeric values.

```php
// Total revenue
$revenue = $store->aggregate('orders', ['sum' => 'total']);

// Total hours worked
$hours = $store->aggregate('timesheets', ['sum' => 'hours']);
```

### `avg` - Average Values

Calculate the arithmetic mean.

```php
// Average product rating
$avgRating = $store->aggregate('products', ['avg' => 'rating']);
// Returns: ['avg' => 4.2]

// Average order value
$avgOrderValue = $store->aggregate('orders', ['avg' => 'total']);
// Returns: ['avg' => 87.50]
```

### `min` - Minimum Value

Find the smallest value.

```php
// Cheapest product
$cheapest = $store->aggregate('products', ['min' => 'price']);

// Youngest user
$youngest = $store->aggregate('users', ['min' => 'age']);
```

### `max` - Maximum Value

Find the largest value.

```php
// Most expensive product
$mostExpensive = $store->aggregate('products', ['max' => 'price']);

// Highest score
$topScore = $store->aggregate('scores', ['max' => 'points']);
```

### `count` - Count Items

Count the number of items.

```php
// Total number of users
$totalUsers = $store->aggregate('users', ['count' => 'id']);

// Total products in stock
$inStock = $store->aggregate('products', ['count' => 'sku']);
```

---

## Grouping Data

### Group By Field

Group records by a field value:

```php
// Group users by city
$byCity = $store->groupBy('users', 'city');

// Returns:
// [
//     'NYC' => [
//         ['id' => 1, 'name' => 'Alice', 'city' => 'NYC'],
//         ['id' => 3, 'name' => 'Charlie', 'city' => 'NYC']
//     ],
//     'LA' => [
//         ['id' => 2, 'name' => 'Bob', 'city' => 'LA']
//     ],
//     'Chicago' => [
//         ['id' => 4, 'name' => 'Dave', 'city' => 'Chicago']
//     ]
// ]

// Group products by category
$byCategory = $store->groupBy('products', 'category');

// Group orders by status
$byStatus = $store->groupBy('orders', 'status');
```

### Analyzing Grouped Data

Combine `groupBy` with other operations:

```php
// Group by category and count items in each
$byCategory = $store->groupBy('products', 'category');

foreach ($byCategory as $category => $products) {
    $count = count($products);
    echo "$category: $count products\n";
}

// Calculate average price per category
foreach ($byCategory as $category => $products) {
    $total = array_sum(array_column($products, 'price'));
    $avg = $total / count($products);
    echo "$category: $" . number_format($avg, 2) . "\n";
}
```

---

## Field Extraction

### Pluck - Extract Field Values

Extract values from a specific field across all records:

```php
// Extract all user names
$names = $store->pluck('users', ['name']);
// Returns: ['Alice', 'Bob', 'Charlie', 'Dave']

// Extract all email addresses
$emails = $store->pluck('users', ['email']);
// Returns: ['alice@example.com', 'bob@example.com', ...]

// Extract multiple fields
$info = $store->pluck('users', ['name', 'email']);
// Returns:
// [
//     ['name' => 'Alice', 'email' => 'alice@example.com'],
//     ['name' => 'Bob', 'email' => 'bob@example.com'],
//     ...
// ]
```

---

## Collection Methods

### `column` - Extract Single Column

Extract values from a single column (similar to `array_column`):

```php
// Get all user emails
$emails = $store->column('users', 'email');
// Returns: ['alice@example.com', 'bob@example.com', ...]

// Get all product IDs
$ids = $store->column('products', 'id');
// Returns: [1, 2, 3, 4, 5]
```

### `implode` - Join Column Values

Join column values into a string:

```php
// Create comma-separated list of names
$nameList = $store->implode('users', 'name', ', ');
// Returns: "Alice, Bob, Charlie, Dave"

// Create tag list
$tagList = $store->implode('products', 'category', ' | ');
// Returns: "Electronics | Books | Clothing"
```

### `keys` - Get Object Keys

Get the keys of an object:

```php
// Get field names from a user object
$fields = $store->keys('users.0');
// Returns: ['id', 'name', 'email', 'age', 'city']

// Get top-level keys
$topLevel = $store->keys('');
// Returns: ['users', 'products', 'orders', 'config']
```

### `values` - Get Object Values

Get the values of an object:

```php
// Get all values from a user profile
$values = $store->values('users.0');
// Returns: [1, 'Alice', 'alice@example.com', 30, 'NYC']
```

---

## Advanced Examples

### Calculate Revenue by Category

```php
$store->set('orders', [
    ['id' => 1, 'category' => 'electronics', 'total' => 299.99],
    ['id' => 2, 'category' => 'books', 'total' => 49.99],
    ['id' => 3, 'category' => 'electronics', 'total' => 599.99],
    ['id' => 4, 'category' => 'books', 'total' => 29.99],
]);

// Group by category
$byCategory = $store->groupBy('orders', 'category');

// Calculate revenue per category
$revenue = [];
foreach ($byCategory as $category => $orders) {
    $total = array_sum(array_column($orders, 'total'));
    $revenue[$category] = $total;
}

print_r($revenue);
// ['electronics' => 899.98, 'books' => 79.98]
```

### User Demographics Analysis

```php
// Get all user ages
$ages = $store->column('users', 'age');

// Calculate statistics
$stats = [
    'total_users' => count($ages),
    'average_age' => array_sum($ages) / count($ages),
    'min_age' => min($ages),
    'max_age' => max($ages)
];

// Group by age ranges
$byAge = [];
foreach ($ages as $age) {
    if ($age < 18) $byAge['under_18'][] = $age;
    elseif ($age < 30) $byAge['18_29'][] = $age;
    elseif ($age < 50) $byAge['30_49'][] = $age;
    else $byAge['50_plus'][] = $age;
}

$distribution = array_map('count', $byAge);
print_r($distribution);
```

### Sales Performance

```php
// Get sales data
$store->set('sales', [
    ['month' => 'Jan', 'revenue' => 50000, 'orders' => 120],
    ['month' => 'Feb', 'revenue' => 65000, 'orders' => 150],
    ['month' => 'Mar', 'revenue' => 72000, 'orders' => 165],
]);

// Calculate totals and averages
$stats = $store->aggregate('sales', [
    'sum' => 'revenue',
    'avg' => 'revenue',
    'count' => 'month'
]);

echo "Total Revenue: $" . number_format($stats['sum'], 2) . "\n";
echo "Average Monthly Revenue: $" . number_format($stats['avg'], 2) . "\n";
echo "Number of Months: " . $stats['count'] . "\n";

// Calculate average order value per month
$months = $store->get('sales');
foreach ($months as $month) {
    $aov = $month['revenue'] / $month['orders'];
    echo "{$month['month']}: $" . number_format($aov, 2) . " per order\n";
}
```

---

## Performance Tips

### 1. Use Appropriate Aggregations

```php
// Good: Use count for counting
$total = $store->aggregate('users', ['count' => 'id']);

// Avoid: Don't fetch all data just to count
// $users = $store->get('users');
// $count = count($users); // Inefficient
```

### 2. Combine Aggregations

```php
// Good: Single call for multiple stats
$stats = $store->aggregate('products', [
    'sum' => 'price',
    'avg' => 'price',
    'min' => 'price',
    'max' => 'price'
]);

// Avoid: Multiple separate calls
// $sum = $store->aggregate('products', ['sum' => 'price']);
// $avg = $store->aggregate('products', ['avg' => 'price']);
```

### 3. Filter Before Aggregating

```php
// Get active users only, then aggregate
$activeUsers = $store->find('users', ['status' => 'active']);
$avgAge = array_sum(array_column($activeUsers, 'age')) / count($activeUsers);
```

### 4. Use Field Projection

```php
// Extract only needed fields
$names = $store->pluck('users', ['name']);

// More efficient than:
// $users = $store->get('users');
// $names = array_column($users, 'name');
```

---

## Common Patterns

### Dashboard Statistics

```php
function getDashboardStats($store) {
    return [
        'total_users' => $store->aggregate('users', ['count' => 'id'])['count'],
        'total_products' => $store->aggregate('products', ['count' => 'id'])['count'],
        'total_orders' => $store->aggregate('orders', ['count' => 'id'])['count'],
        'total_revenue' => $store->aggregate('orders', ['sum' => 'total'])['sum'],
        'avg_order_value' => $store->aggregate('orders', ['avg' => 'total'])['avg'],
        'users_by_role' => $store->groupBy('users', 'role'),
        'orders_by_status' => $store->groupBy('orders', 'status')
    ];
}
```

### Top Products Report

```php
// Get products with their order counts
$products = $store->get('products');
$orders = $store->get('orders');

$productSales = [];
foreach ($orders as $order) {
    $productId = $order['product_id'];
    if (!isset($productSales[$productId])) {
        $productSales[$productId] = 0;
    }
    $productSales[$productId]++;
}

// Sort by sales count
arsort($productSales);

// Get top 10
$top10 = array_slice($productSales, 0, 10, true);
```

---

## See Also

- [Querying Data](queries.md) - Filter data before aggregating
- [API Reference](../api/store-class.md) - Complete API documentation
- [Collection Methods](../api/store-class.md#collection-methods) - Additional data manipulation methods
