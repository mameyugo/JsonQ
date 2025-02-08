<?php
/**
 * rjson Quick Start Example
 *
 * Run: php -d "extension=path/to/librjson.so" examples/quickstart.php
 */

use Rjson\Store;

echo "rjson Quick Start — v" . rjson_version() . "\n\n";

// ── 1. Create a store ──
$store = new Store('/tmp/rjson_quickstart.json');

// ── 2. Set data (dot-notation creates intermediates) ──
$store->set('app.name', 'MyApp');
$store->set('app.version', '2.1.0');
$store->set('app.debug', false);

$store->set('users', [
    ['id' => 1, 'name' => 'Alice',   'email' => 'alice@example.com',   'role' => 'admin',  'age' => 30],
    ['id' => 2, 'name' => 'Bob',     'email' => 'bob@example.com',     'role' => 'user',   'age' => 25],
    ['id' => 3, 'name' => 'Charlie', 'email' => 'charlie@example.com', 'role' => 'admin',  'age' => 35],
    ['id' => 4, 'name' => 'Diana',   'email' => 'diana@example.com',   'role' => 'user',   'age' => 28],
    ['id' => 5, 'name' => 'Eve',     'email' => 'eve@example.com',     'role' => 'viewer', 'age' => 22],
]);

// ── 3. Read data ──
echo "App: {$store->get('app.name')} v{$store->get('app.version')}\n";
echo "Users: {$store->count('users')}\n";
echo "Top keys: " . implode(', ', $store->keys('')) . "\n\n";

// ── 4. MongoDB-style queries ──
echo "── MongoDB Queries ──\n";

$admins = $store->find('users', ['role' => 'admin']);
echo "Admins: " . count($admins) . " → " . implode(', ', array_column($admins, 'name')) . "\n";

$young = $store->find('users', ['age' => ['$lt' => 26]]);
echo "Under 26: " . implode(', ', array_column($young, 'name')) . "\n";

$gmailOrAdmin = $store->find('users', [
    '$or' => [
        ['email' => ['$endsWith' => '@gmail.com']],
        ['role' => 'admin'],
    ]
]);
echo "Gmail or Admin: " . count($gmailOrAdmin) . "\n";

$alice = $store->findOne('users', ['name' => 'Alice']);
echo "FindOne Alice: id={$alice['id']}, role={$alice['role']}\n\n";

// ── 5. Fluent queries ──
echo "── Fluent Queries ──\n";

$result = $store->executeQuery('users', [
    'where'    => [
        ['field' => 'age', 'op' => 'between', 'value' => [25, 35]],
        ['field' => 'role', 'op' => '!=', 'value' => 'viewer'],
    ],
    'order_by' => ['field' => 'age', 'direction' => 'desc'],
    'select'   => ['name', 'age', 'role'],
    'limit'    => 3,
]);
echo "Age 25-35 (non-viewer, desc, top 3):\n";
foreach ($result as $r) {
    echo "  {$r['name']} — age {$r['age']}, {$r['role']}\n";
}
echo "\n";

// ── 6. Aggregation ──
echo "── Aggregation ──\n";
echo "Sum ages: " . $store->aggregate('users', 'age', 'sum') . "\n";
echo "Avg age:  " . $store->aggregate('users', 'age', 'avg') . "\n";
echo "Min age:  " . $store->aggregate('users', 'age', 'min') . "\n";
echo "Max age:  " . $store->aggregate('users', 'age', 'max') . "\n";

$byRole = $store->groupBy('users', 'role');
echo "By role:\n";
foreach ($byRole as $role => $users) {
    echo "  {$role}: " . count($users) . " users\n";
}

$emails = $store->pluck('users', ['email']);
echo "Emails: " . implode(', ', $emails) . "\n\n";

// ── 7. Indexes ──
echo "── Indexes ──\n";

$store->createIndex('users', 'role');
$store->createIndex('users', 'email');

$admins = $store->indexLookup('users', 'role', 'admin');
echo "Index lookup 'admin': " . count($admins) . " results\n";

$indexes = $store->listIndexes();
foreach ($indexes as $idx) {
    echo "  Index on {$idx['collection']}.{$idx['field']}: {$idx['unique_values']} unique, {$idx['total_entries']} entries\n";
}
echo "\n";

// ── 8. Validation ──
echo "── Schema Validation ──\n";

$result = $store->validateCollection('users', [
    'type'       => 'object',
    'required'   => ['id', 'name', 'email', 'role'],
    'properties' => [
        'id'    => ['type' => 'integer', 'min' => 1],
        'name'  => ['type' => 'string', 'minLength' => 1],
        'email' => ['type' => 'string', 'format' => 'email'],
        'role'  => ['type' => 'string', 'enum' => ['admin', 'user', 'viewer']],
        'age'   => ['type' => 'integer', 'min' => 0, 'max' => 150],
    ],
]);
echo "Collection valid: " . ($result['valid'] ? 'YES' : 'NO') . "\n";
echo "Total: {$result['total_items']}, Valid: {$result['valid_items']}, Invalid: {$result['invalid_items']}\n\n";

// ── 9. Mutations ──
echo "── Mutations ──\n";

$store->push('users', ['id' => 6, 'name' => 'Frank', 'email' => 'frank@example.com', 'role' => 'user', 'age' => 31]);
echo "After push: {$store->count('users')} users\n";

$store->set('stats.views', 100);
$store->increment('stats.views', 15.0);
echo "Views after +15: " . $store->get('stats.views') . "\n";

$store->merge('app', ['debug' => true, 'env' => 'development']);
echo "Debug after merge: " . var_export($store->get('app.debug'), true) . "\n";
echo "Env: {$store->get('app.env')}\n\n";

// ── 10. Utilities ──
echo "── Stats ──\n";
$stats = $store->stats();
echo "File: {$stats['file_size_h']}, Keys: {$stats['key_count']}, Indexes: {$stats['active_indexes']}\n";

$backup = $store->backup();
echo "Backup: {$backup}\n";

echo "\n✅ Quick start complete!\n";

// Cleanup
unlink('/tmp/rjson_quickstart.json');
unlink($backup);
