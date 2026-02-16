# Querying Data

JsonQ offers two powerful ways to query your data:
1. **MongoDB-Style Queries**: Using declarative arrays with operators.
2. **Fluent Query Builder**: Method chaining for complex criteria.

## 1. MongoDB-Style Queries

The `find($collection, $conditions)` method accepts an array of conditions.

### Comparison Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `$eq` | Equal to (implicit) | `['age' => 25]` |
| `$ne` | Not equal to | `['age' => ['$ne' => 25]]` |
| `$gt` | Greater than | `['age' => ['$gt' => 25]]` |
| `$gte` | Greater than or equal | `['age' => ['$gte' => 25]]` |
| `$lt` | Less than | `['age' => ['$lt' => 25]]` |
| `$lte` | Less than or equal | `['age' => ['$lte' => 25]]` |

**Example:**
```php
// Find users where age >= 18 AND age <= 30
$users = $store->find('users', [
    'age' => [
        '$gte' => 18,
        '$lte' => 30
    ]
]);
```

### Array Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `$in` | In array | `['role' => ['$in' => ['admin', 'mod']]]` |
| `$nin` | Not in array | `['role' => ['$nin' => ['guest']]]` |
| `$size` | Array size matches | `['tags' => ['$size' => 3]]` |
| `$all` | Contains all values | `['tags' => ['$all' => ['php', 'rust']]]` |

**Example:**
```php
// Find posts with specific tags
$posts = $store->find('posts', [
    'tags' => ['$in' => ['news', 'update']]
]);
```

### String Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `$contains` | Substring match | `['title' => ['$contains' => 'JsonQ']]` |
| `$startsWith` | Starts with | `['slug' => ['$startsWith' => '2023-']]` |
| `$endsWith` | Ends with | `['email' => ['$endsWith' => '@gmail.com']]` |
| `$regex` | Regex match | `['code' => ['$regex' => '^[A-Z]{3}-\d{3}$']]` |

> **Note:** `$regex` uses Rust's regex engine. It is safe and performant, with protection against ReDoS attacks.

### Logical Operators

Combine multiple conditions.

| Operator | Description | Example |
|----------|-------------|---------|
| `$and` | AND (implicit) | `['$and' => [['a'=>1], ['b'=>2]]]` |
| `$or` | OR | `['$or' => [['role'=>'admin'], ['id'=>1]]]` |
| `$not` | NOT | `['age' => ['$not' => ['$lt' => 18]]]` |

**Example:**
```php
// Find users who are admins OR (editors AND active)
$users = $store->find('users', [
    '$or' => [
        ['role' => 'admin'],
        [
            '$and' => [
                ['role' => 'editor'],
                ['active' => true]
            ]
        ]
    ]
]);
```

## 2. Fluent Query Builder

The `executeQuery($collection, $query)` method allows you to structure queries with explicit clauses for sorting, pagination, and projection.

### Structure

```php
$query = [
    'where' => [ ... ],
    'order_by' => [ ... ],
    'limit' => int,
    'offset' => int,
    'select' => [ ... ]
];
```

### Filtering (`where`)

A list of conditions. Each condition has `field`, `op` (operator), and `value`.

**Supported Operators:** `=`, `!=`, `>`, `>=`, `<`, `<=`, `in`, `not in`, `contains`, `starts_with`, `ends_with`, `between`.

```php
$results = $store->executeQuery('products', [
    'where' => [
        ['field' => 'price', 'op' => '>=', 'value' => 100],
        ['field' => 'category', 'op' => 'in', 'value' => ['electronics', 'computers']]
    ]
]);
```

### Sorting (`order_by`)

Sort results by a field.

```php
'order_by' => [
    'field' => 'created_at',
    'direction' => 'desc' // or 'asc'
]
```

### Pagination (`limit`, `offset`)

Efficiently page through results.

```php
'limit' => 20,
'offset' => 0 // Page 1
```

### Projection (`select` / `except`)

Choose which fields to return. You can use **either** `select` or `except`, but not both.

**Select (Whitelist):**
Only return specific fields.
```php
'select' => ['id', 'name', 'email']
```

**Except (Blacklist):**
Return all fields *except* specific ones.
```php
'except' => ['password_hash', 'audit_logs']
```

### Full Example

```php
$users = $store->executeQuery('users', [
    'where' => [
        ['field' => 'active', 'op' => '=', 'value' => true]
    ],
    'order_by' => ['field' => 'joined_at', 'direction' => 'desc'],
    'limit' => 10,
    'offset' => 0,
    'select' => ['id', 'username', 'avatar_url']
]);
```
