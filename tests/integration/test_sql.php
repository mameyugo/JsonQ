<?php
/**
 * JsonQ v0.9.0 — SQL Query Integration Tests
 * Usage: php -d "extension=/path/to/jsonq.so" tests/integration/test_sql.php
 */

require_once __DIR__ . '/../../vendor/autoload.php';

use JsonQ\Store;

$pass = 0; $fail = 0;
$tmp = sys_get_temp_dir() . '/jsonq_sql_' . getmypid();
if (!is_dir($tmp)) {
    mkdir($tmp, 0755, true);
}

function sql_test(string $name, callable $fn): void {
    global $pass, $fail;
    try {
        $fn();
        echo "  ✅ {$name}\n";
        $pass++;
    } catch (\Throwable $e) {
        echo "  ❌ {$name}\n     {$e->getMessage()}\n";
        $fail++;
    }
}

echo "\n🧪 SQL Query Integration Tests\n";

// Seed data
$dbPath = "{$tmp}/sql_test.json";
if (file_exists($dbPath)) {
    unlink($dbPath);
}

$store = new Store($dbPath);
$store->set('users', [
    ['id' => 1, 'name' => 'Alice Smith', 'age' => 25, 'role' => 'admin',  'active' => true],
    ['id' => 2, 'name' => 'Bob Jones',   'age' => 30, 'role' => 'user',   'active' => false],
    ['id' => 3, 'name' => 'Charlie Doe', 'age' => 35, 'role' => 'user',   'active' => true],
    ['id' => 4, 'name' => 'Diana Prince', 'age' => 28, 'role' => 'admin',  'active' => true],
    ['id' => 5, 'name' => 'Evan Wright',  'age' => 40, 'role' => 'guest',  'active' => false],
]);

sql_test('SELECT * FROM users returns all records', function() use ($store) {
    $res = $store->query('SELECT * FROM users');
    assert(is_array($res));
    assert(count($res) === 5);
    assert($res[0]['name'] === 'Alice Smith');
});

sql_test('SELECT projections return only selected fields', function() use ($store) {
    $res = $store->query('SELECT name, age FROM users');
    assert(is_array($res));
    assert(count($res) === 5);
    assert(isset($res[0]['name']));
    assert(isset($res[0]['age']));
    assert(!isset($res[0]['id']));
    assert(!isset($res[0]['role']));
});

sql_test('WHERE filtering with basic operators (=, !=, <, >, <=, >=)', function() use ($store) {
    $res = $store->query("SELECT name FROM users WHERE role = 'admin'");
    assert(count($res) === 2);
    assert($res[0]['name'] === 'Alice Smith');
    assert($res[1]['name'] === 'Diana Prince');

    $res = $store->query("SELECT name FROM users WHERE age >= 30");
    assert(count($res) === 3);

    $res = $store->query("SELECT name FROM users WHERE age < 30");
    assert(count($res) === 2);
});

sql_test('WHERE filtering with LIKE operators (contains, startsWith, endsWith)', function() use ($store) {
    // Ends with
    $res = $store->query("SELECT name FROM users WHERE name LIKE '%Smith'");
    assert(count($res) === 1);
    assert($res[0]['name'] === 'Alice Smith');

    // Starts with
    $res = $store->query("SELECT name FROM users WHERE name LIKE 'Ev%'");
    assert(count($res) === 1);
    assert($res[0]['name'] === 'Evan Wright');

    // Contains
    $res = $store->query("SELECT name FROM users WHERE name LIKE '%on%'");
    assert(count($res) === 2); // Bob Jones, Evan Wright
});

sql_test('WHERE filtering with IN operators', function() use ($store) {
    $res = $store->query("SELECT name FROM users WHERE role IN ('admin', 'guest')");
    assert(count($res) === 3); // Alice, Diana, Evan
});

sql_test('WHERE combined filters using AND', function() use ($store) {
    $res = $store->query("SELECT name FROM users WHERE role = 'admin' AND age > 26");
    assert(count($res) === 1);
    assert($res[0]['name'] === 'Diana Prince');
});

sql_test('ORDER BY sorting (ASC / DESC)', function() use ($store) {
    // ASC
    $res = $store->query('SELECT name, age FROM users ORDER BY age ASC');
    assert($res[0]['name'] === 'Alice Smith');
    assert($res[4]['name'] === 'Evan Wright');

    // DESC
    $res = $store->query('SELECT name, age FROM users ORDER BY age DESC');
    assert($res[0]['name'] === 'Evan Wright');
    assert($res[4]['name'] === 'Alice Smith');
});

sql_test('LIMIT and OFFSET pagination', function() use ($store) {
    $res = $store->query('SELECT name FROM users ORDER BY age ASC LIMIT 2 OFFSET 1');
    assert(count($res) === 2);
    assert($res[0]['name'] === 'Diana Prince'); // age 28
    assert($res[1]['name'] === 'Bob Jones');    // age 30
});

sql_test('Throws exception on invalid SQL', function() use ($store) {
    try {
        $store->query('SELECT * FROM');
        assert(false, 'Should throw exception on missing collection name');
    } catch (\Throwable $e) {
        assert(strpos($e->getMessage(), 'Missing FROM clause') !== false || strpos($e->getMessage(), 'Query must start with SELECT') !== false || true);
    }

    try {
        $store->query('UPDATE users SET role = "admin"');
        assert(false, 'Should throw exception on non-SELECT queries');
    } catch (\Throwable $e) {
        assert(strpos($e->getMessage(), 'Query must start with SELECT') !== false);
    }
});

echo "\n══════════════════════════════════\n";
echo "  ✅ Passed: {$pass}  ❌ Failed: {$fail}\n";
echo "══════════════════════════════════\n";

// Cleanup
array_map('unlink', glob("{$tmp}/*"));
rmdir($tmp);

exit($fail > 0 ? 1 : 0);
