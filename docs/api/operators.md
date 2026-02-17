# Query Operators Reference

Complete reference for all MongoDB-style query operators supported by JsonQ.

## Comparison Operators

### `$eq` - Equal To

Matches values that are equal to a specified value.

```php
// Find users with age exactly 30
$users = $store->find('users', ['age' => ['$eq' => 30]]);

// Shorthand (implicit $eq)
$users = $store->find('users', ['age' => 30]);
```

### `$ne` - Not Equal To

Matches all values that are not equal to a specified value.

```php
// Find users who are not banned
$users = $store->find('users', ['status' => ['$ne' => 'banned']]);

// Find active users
$users = $store->find('users', ['role' => ['$ne' => 'guest']]);
```

### `$gt` - Greater Than

Matches values that are greater than a specified value.

```php
// Find adults (age > 18)
$adults = $store->find('users', ['age' => ['$gt' => 18]]);

// Find expensive products
$products = $store->find('products', ['price' => ['$gt' => 100]]);
```

### `$gte` - Greater Than or Equal To

Matches values that are greater than or equal to a specified value.

```php
// Find users 18 or older
$adults = $store->find('users', ['age' => ['$gte' => 18]]);

// Find high scores
$scores = $store->find('scores', ['points' => ['$gte' => 90]]);
```

### `$lt` - Less Than

Matches values that are less than a specified value.

```php
// Find young users (age < 30)
$young = $store->find('users', ['age' => ['$lt' => 30]]);

// Find budget items
$budget = $store->find('products', ['price' => ['$lt' => 50]]);
```

### `$lte` - Less Than or Equal To

Matches values that are less than or equal to a specified value.

```php
// Find users 65 or younger
$users = $store->find('users', ['age' => ['$lte' => 65]]);

// Find affordable products
$products = $store->find('products', ['price' => ['$lte' => 99.99]]);
```

### Range Queries

Combine `$gte` and `$lte` for range queries:

```php
// Find users between 25 and 35 years old
$midAge = $store->find('users', [
    'age' => ['$gte' => 25, '$lte' => 35]
]);

// Find products in price range
$products = $store->find('products', [
    'price' => ['$gte' => 50, '$lte' => 200]
]);
```

---

## Array Operators

### `$in` - In Array

Matches any of the values specified in an array.

```php
// Find admins and moderators
$staff = $store->find('users', [
    'role' => ['$in' => ['admin', 'moderator']]
]);

// Find users from specific cities
$users = $store->find('users', [
    'city' => ['$in' => ['NYC', 'LA', 'Chicago']]
]);
```

### `$nin` - Not In Array

Matches none of the values specified in an array.

```php
// Exclude banned and deleted users
$active = $store->find('users', [
    'status' => ['$nin' => ['banned', 'deleted']]
]);

// Exclude specific roles
$regular = $store->find('users', [
    'role' => ['$nin' => ['admin', 'guest']]
]);
```

### `$size` - Array Size

Matches arrays with a specific number of elements.

```php
// Find users with exactly 3 tags
$users = $store->find('users', [
    'tags' => ['$size' => 3]
]);

// Find teams with 5 members
$teams = $store->find('projects', [
    'members' => ['$size' => 5]
]);
```

### `$all` - Contains All

Matches arrays that contain all specified elements.

```php
// Find users with all required skills
$qualified = $store->find('users', [
    'skills' => ['$all' => ['PHP', 'JavaScript', 'SQL']]
]);
```

### `$elemMatch` - Element Match

Matches documents that contain an array field with at least one element matching all criteria.

```php
// Find orders with items over $100
$orders = $store->find('orders', [
    'items' => ['$elemMatch' => ['price' => ['$gt' => 100]]]
]);
```

---

## String Operators

### `$contains` - String Contains

Matches strings that contain a specified substring.

```php
// Find Gmail users
$gmailUsers = $store->find('users', [
    'email' => ['$contains' => '@gmail.com']
]);

// Find posts mentioning a topic
$posts = $store->find('posts', [
    'content' => ['$contains' => 'JsonQ']
]);
```

### `$startsWith` - Starts With

Matches strings that start with a specified prefix.

```php
// Find admin usernames
$admins = $store->find('users', [
    'username' => ['$startsWith' => 'admin_']
]);

// Find files in a directory
$files = $store->find('files', [
    'path' => ['$startsWith' => '/uploads/']
]);
```

### `$endsWith` - Ends With

Matches strings that end with a specified suffix.

```php
// Find text files
$txtFiles = $store->find('files', [
    'name' => ['$endsWith' => '.txt']
]);

// Find .com domains
$domains = $store->find('websites', [
    'url' => ['$endsWith' => '.com']
]);
```

### `$regex` - Regular Expression

Matches strings against a regular expression pattern. Includes ReDoS protection with backtracking limits.

```php
// Find US phone numbers
$contacts = $store->find('contacts', [
    'phone' => ['$regex' => '^\+1-\d{3}-\d{3}-\d{4}$']
]);

// Find email addresses
$users = $store->find('users', [
    'email' => ['$regex' => '^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$']
]);
```

**Note:** Regex patterns are compiled and cached for performance. Backtracking is limited to prevent ReDoS attacks.

---

## Type and Existence Operators

### `$exists` - Field Exists

Matches documents where the field exists and is not null.

```php
// Find users with phone numbers
$withPhone = $store->find('users', [
    'phone' => ['$exists' => true]
]);

// Find users without phone numbers
$withoutPhone = $store->find('users', [
    'phone' => ['$exists' => false]
]);
```

### `$type` - Type Check

Matches documents where the field is of a specific type.

```php
// Find records where age is an integer
$users = $store->find('users', [
    'age' => ['$type' => 'integer']
]);

// Find records where tags is an array
$posts = $store->find('posts', [
    'tags' => ['$type' => 'array']
]);
```

**Supported types:** `string`, `integer`, `number`, `boolean`, `array`, `object`, `null`

---

## Logical Operators

### `$and` - Logical AND

Performs a logical AND operation on an array of conditions. All conditions must match.

```php
// Find senior admins (implicit AND)
$seniorAdmins = $store->find('users', [
    'role' => 'admin',
    'age' => ['$gte' => 30]
]);

// Explicit AND
$results = $store->find('users', [
    '$and' => [
        ['age' => ['$gte' => 18]],
        ['verified' => true]
    ]
]);
```

### `$or` - Logical OR

Performs a logical OR operation on an array of conditions. At least one condition must match.

```php
// Find staff members (admin or moderator)
$staff = $store->find('users', [
    '$or' => [
        ['role' => 'admin'],
        ['role' => 'moderator']
    ]
]);

// Find users from NYC or LA
$users = $store->find('users', [
    '$or' => [
        ['city' => 'NYC'],
        ['city' => 'LA']
    ]
]);
```

### `$not` - Logical NOT

Performs a logical NOT operation, inverting the effect of a query expression.

```php
// Find non-guest users
$members = $store->find('users', [
    '$not' => ['role' => 'guest']
]);

// Find users not from banned list
$active = $store->find('users', [
    '$not' => ['status' => 'banned']
]);
```

### `$nor` - Logical NOR

Performs a logical NOR operation. Returns documents that fail all conditions.

```php
// Find users who are neither banned nor deleted
$active = $store->find('users', [
    '$nor' => [
        ['status' => 'banned'],
        ['status' => 'deleted']
    ]
]);
```

---

## Complex Queries

### Nested Logical Operators

Combine logical operators for complex queries:

```php
// Find verified adults who are either admins or have premium status
$results = $store->find('users', [
    '$and' => [
        ['age' => ['$gte' => 18]],
        ['verified' => true],
        ['$or' => [
            ['role' => 'admin'],
            ['premium' => true]
        ]]
    ]
]);
```

### Multiple Conditions

```php
// Find products in price range with specific categories
$products = $store->find('products', [
    'price' => ['$gte' => 50, '$lte' => 200],
    'category' => ['$in' => ['electronics', 'computers']],
    'inStock' => true,
    'rating' => ['$gte' => 4.0]
]);
```

---

## Performance Tips

1. **Use Indexes**: Create indexes on frequently queried fields for O(1) lookups
   ```php
   $store->createIndex('users', 'email');
   ```

2. **Simple Equality First**: Simple equality queries automatically use indexes
   ```php
   // This uses index if available
   $user = $store->find('users', ['email' => 'alice@example.com']);
   ```

3. **Limit Results**: Use `executeQuery` with `limit` for pagination
   ```php
   $results = $store->executeQuery('users', [
       'where' => [['field' => 'age', 'op' => '>=', 'value' => 18]],
       'limit' => 20
   ]);
   ```

4. **Field Projection**: Select only needed fields to reduce memory usage
   ```php
   $results = $store->executeQuery('users', [
       'select' => ['name', 'email']
   ]);
   ```

---

## Operator Summary Table

| Operator | Type | Description | Example |
|----------|------|-------------|---------|
| `$eq` | Comparison | Equal to | `['age' => ['$eq' => 30]]` |
| `$ne` | Comparison | Not equal to | `['role' => ['$ne' => 'guest']]` |
| `$gt` | Comparison | Greater than | `['age' => ['$gt' => 18]]` |
| `$gte` | Comparison | Greater than or equal | `['score' => ['$gte' => 90]]` |
| `$lt` | Comparison | Less than | `['age' => ['$lt' => 30]]` |
| `$lte` | Comparison | Less than or equal | `['price' => ['$lte' => 99.99]]` |
| `$in` | Array | In array | `['role' => ['$in' => ['admin', 'mod']]]` |
| `$nin` | Array | Not in array | `['status' => ['$nin' => ['banned']]]` |
| `$size` | Array | Array length equals | `['tags' => ['$size' => 5]]` |
| `$all` | Array | Contains all elements | `['skills' => ['$all' => ['PHP', 'JS']]]` |
| `$elemMatch` | Array | Element matches criteria | `['items' => ['$elemMatch' => [...]]]` |
| `$contains` | String | String contains | `['email' => ['$contains' => '@gmail']]` |
| `$startsWith` | String | String starts with | `['name' => ['$startsWith' => 'A']]` |
| `$endsWith` | String | String ends with | `['file' => ['$endsWith' => '.txt']]` |
| `$regex` | String | Regex pattern match | `['phone' => ['$regex' => '^\+1-']]` |
| `$exists` | Type | Field exists | `['phone' => ['$exists' => true]]` |
| `$type` | Type | Value type matches | `['age' => ['$type' => 'integer']]` |
| `$and` | Logical | All conditions match | `['$and' => [{...}, {...}]]` |
| `$or` | Logical | Any condition matches | `['$or' => [{...}, {...}]]` |
| `$not` | Logical | Negate condition | `['$not' => ['role' => 'guest']]` |
| `$nor` | Logical | No conditions match | `['$nor' => [{...}, {...}]]` |

---

## See Also

- [Querying Data Guide](../guides/queries.md) - Full guide to querying
- [Indexing Guide](../guides/indexing.md) - Performance optimization
- [API Reference](store-class.md) - Complete API documentation
