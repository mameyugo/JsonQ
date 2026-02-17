# Best Practices Guide

Learn best practices for using JsonQ in production applications.

## Data Organization

### 1. Structure Your Data Logically

```php
// Good: Organized by entity
{
  "users": [...],
  "products": [...],
  "orders": [...]
}

// Avoid: Flat unstructured data
{
  "data": [...],
  "stuff": [...],
  "misc": [...]
}
```

### 2. Use Meaningful Field Names

```php
// Good
['created_at' => '2026-02-17 10:30:00', 'user_id' => 123]

// Avoid
['ts' => 1708168200, 'uid' => 123]
```

### 3. Keep Collections Focused

```php
// Good: Separate collections
$users = $store->get('users');
$orders = $store->get('orders');

// Avoid: Mixed data in one collection
$everything = $store->get('data');
```

---

## Performance Optimization

### 1. Create Indexes on Frequently Queried Fields

```php
// Index email for user lookups
$store->createIndex('users', 'email');

// Compound index for multi-field queries
$store->createCompoundIndex('orders', ['user_id', 'status']);
```

### 2. Use Field Projection

```php
// Good: Select only needed fields
$users = $store->executeQuery('users', [
    'select' => ['name', 'email']
]);

// Avoid: Fetching all fields when not needed
$users = $store->get('users');
```

### 3. Limit Query Results

```php
// Good: Paginate large result sets
$results = $store->executeQuery('products', [
    'limit' => 20,
    'offset' => 0
]);

// Avoid: Loading thousands of records at once
$all = $store->get('products'); // Could be huge!
```

### 4. Filter Before Aggregating

```php
// Good: Filter first, then aggregate
$activeUsers = $store->find('users', ['status' => 'active']);
$avgAge = array_sum(array_column($activeUsers, 'age')) / count($activeUsers);

// Less efficient: Aggregate all then filter
```

---

## Data Integrity

### 1. Use Schema Validation

```php
$userSchema = [
    'type' => 'object',
    'required' => ['name', 'email'],
    'properties' => [
        'name' => ['type' => 'string', 'minLength' => 2],
        'email' => ['type' => 'string', 'format' => 'email'],
        'age' => ['type' => 'integer', 'minimum' => 0, 'maximum' => 150]
    ]
];

// Validate before saving
$result = $store->validate($userData, $userSchema);
if (!$result['valid']) {
    throw new Exception('Invalid data');
}
```

### 2. Use Transactions for Related Operations

```php
try {
    $store->begin();
    
    // Deduct from account A
    $store->set('accounts.A.balance', 900);
    
    // Add to account B
    $store->set('accounts.B.balance', 1100);
    
    // Log transaction
    $store->push('transactions', [...]);
    
    $store->commit();
} catch (Exception $e) {
    $store->rollback();
    throw $e;
}
```

### 3. Validate Input Data

```php
function createUser(array $data): array {
    // Sanitize input
    $data['name'] = trim($data['name']);
    $data['email'] = filter_var($data['email'], FILTER_VALIDATE_EMAIL);
    
    // Validate
    if (!$data['email']) {
        throw new InvalidArgumentException('Invalid email');
    }
    
    // Check uniqueness
    $existing = $store->findOne('users', ['email' => $data['email']]);
    if ($existing) {
        throw new Exception('Email already exists');
    }
    
    return $data;
}
```

---

## Security

### 1. Sanitize User Input

```php
// Always sanitize before using in queries
$search = htmlspecialchars($_GET['q'], ENT_QUOTES, 'UTF-8');
$results = $store->find('products', [
    'name' => ['$contains' => $search]
]);
```

### 2. Use Base Path Restriction

```php
// Restrict file access to specific directory
jsonq_set_base_path('/var/www/data');

// Now only files under /var/www/data can be accessed
$store = new Store('data.json'); // OK: /var/www/data/data.json
$store = new Store('/etc/passwd'); // ERROR: Outside base path
```

### 3. Set File Size Limits

```php
// Limit maximum file size
jsonq_set_max_file_size('100M');

// Prevent large file attacks
```

### 4. Validate File Extensions

```php
jsonq_set_allowed_extensions('json,db');

// Only .json and .db files allowed
```

---

## Error Handling

### 1. Check Operation Results

```php
// Check if operation succeeded
if (!$store->set('key', 'value')) {
    throw new Exception('Failed to save data');
}

// Check if item exists before accessing
if ($store->has('users.0')) {
    $user = $store->get('users.0');
}
```

### 2. Wrap Critical Operations in Try-Catch

```php
try {
    $store->begin();
    // ... operations ...
    $store->commit();
} catch (Exception $e) {
    $store->rollback();
    error_log("Transaction failed: " . $e->getMessage());
    throw $e;
}
```

### 3. Log Errors

```php
function safeQuery($store, $collection, $query) {
    try {
        return $store->find($collection, $query);
    } catch (Exception $e) {
        error_log("Query error: " . $e->getMessage());
        return [];
    }
}
```

---

## Code Organization

### 1. Create Repository Classes

```php
class UserRepository {
    private Store $store;
    
    public function __construct(Store $store) {
        $this->store = $store;
        $this->store->createIndex('users', 'email');
    }
    
    public function findByEmail(string $email): ?array {
        return $this->store->findOne('users', ['email' => $email]);
    }
    
    public function create(array $data): array {
        // Validation, sanitization, business logic
        // ...
        $this->store->push('users', $data);
        return $data;
    }
}
```

### 2. Use Dependency Injection

```php
class OrderService {
    public function __construct(
        private Store $store,
        private UserRepository $users,
        private ProductRepository $products
    ) {}
    
    public function createOrder(int $userId, array $items): array {
        // Business logic using injected dependencies
    }
}
```

### 3. Separate Business Logic from Data Access

```php
// Data Access Layer
class DataAccess {
    public function getUsers() { /* ... */ }
    public function saveUser($user) { /* ... */ }
}

// Business Logic Layer
class UserService {
    public function registerUser($data) {
        // Validation, password hashing, etc.
        $this->dataAccess->saveUser($user);
    }
}
```

---

## Testing

### 1. Use Separate Test Database

```php
class UserServiceTest extends TestCase {
    private Store $store;
    
    protected function setUp(): void {
        // Use test database
        $this->store = new Store('/tmp/test-' . uniqid() . '.json');
    }
    
    protected function tearDown(): void {
        // Clean up
        unlink($this->store->stats()['file_path']);
    }
}
```

### 2. Test with Real Data

```php
public function testUserCreation() {
    $user = ['name' => 'Test User', 'email' => 'test@example.com'];
    $this->store->push('users', $user);
    
    $found = $this->store->findOne('users', ['email' => 'test@example.com']);
    $this->assertEquals('Test User', $found['name']);
}
```

---

## Monitoring

### 1. Track Performance Metrics

```php
$metrics = $store->getMetrics();

// Log metrics periodically
error_log(sprintf(
    "JsonQ Metrics - Reads: %d, Writes: %d, Cache Hit Rate: %.2f%%",
    $metrics['reads'],
    $metrics['writes'],
    $metrics['cache_hit_rate']
));
```

### 2. Monitor File Size

```php
$stats = $store->stats();
if ($stats['file_size'] > 100 * 1024 * 1024) { // 100MB
    trigger_error("Database file is getting large: {$stats['file_size_h']}", E_USER_WARNING);
}
```

---

## Backup Strategy

### 1. Regular Backups

```php
// Daily backup
$backupPath = "/backups/data-" . date('Y-m-d') . ".json";
$store->backup($backupPath);
```

### 2. Backup Before Major Operations

```php
// Backup before bulk update
$store->backup();

try {
    // Perform bulk operation
    bulkUpdate();
} catch (Exception $e) {
    // Restore from backup if needed
    $store->restore($backupPath);
}
```

---

## Common Pitfalls to Avoid

❌ Not creating indexes on queried fields  
❌ Loading entire collections when you need filtered results  
❌ Not using transactions for related operations  
❌ Storing files in web-accessible directories  
❌ Not validating user input  
❌ Ignoring error returns  
❌ Not monitoring file growth  
❌ Missing backups before risky operations  

---

## See Also

- [Indexing Guide](indexing.md)
- [Schema Validation](schema-validation.md)
- [Transactions](transactions.md)
- [Performance Tuning](../deployment/performance-tuning.md)
