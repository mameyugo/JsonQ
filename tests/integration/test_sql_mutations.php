<?php
/**
 * JsonQ v0.9.0 — SQL Mutation Integration Tests
 * Usage: php -d "extension=/path/to/jsonq.so" tests/integration/test_sql_mutations.php
 */

require_once __DIR__ . '/../../vendor/autoload.php';

use JsonQ\Store;

$pass = 0; $fail = 0;
$tmp = sys_get_temp_dir() . '/jsonq_sql_mut_' . getmypid();
if (!is_dir($tmp)) {
    mkdir($tmp, 0755, true);
}

function sql_mut_test(string $name, callable $fn): void {
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

echo "\n🧪 SQL Mutation Integration Tests\n";

$dbPath = "{$tmp}/mutations_test.json";
if (file_exists($dbPath)) {
    unlink($dbPath);
}

$store = new Store($dbPath);
// Seed initial data
$store->set('users', [
    ['id' => 1, 'name' => 'Alice', 'role' => 'admin',  'age' => 25, 'profile' => ['status' => 'active']],
    ['id' => 2, 'name' => 'Bob',   'role' => 'user',   'age' => 30, 'profile' => ['status' => 'inactive']],
    ['id' => 3, 'name' => 'Charlie','role' => 'user',  'age' => 35, 'profile' => ['status' => 'active']],
]);

sql_mut_test('INSERT INTO creates and appends new JSON object', function() use ($store) {
    $res = $store->query("INSERT INTO users (id, name, role, age) VALUES (4, 'Diana', 'guest', 28)");
    assert($res === 1, 'Should return 1 affected row');

    $users = $store->get('users');
    assert(count($users) === 4);
    assert($users[3]['name'] === 'Diana');
    assert($users[3]['role'] === 'guest');
    assert($users[3]['age'] === 28);
});

sql_mut_test('UPDATE SET updates matching records (single and multiple)', function() use ($store) {
    // Single update
    $res = $store->query("UPDATE users SET role = 'superadmin' WHERE id = 1");
    assert($res === 1, 'Should affect 1 row');
    
    $users = $store->get('users');
    assert($users[0]['role'] === 'superadmin');

    // Multiple updates
    $res = $store->query("UPDATE users SET role = 'staff' WHERE role = 'user'");
    assert($res === 2, 'Should affect 2 rows (Bob, Charlie)');
    
    $users = $store->get('users');
    assert($users[1]['role'] === 'staff');
    assert($users[2]['role'] === 'staff');
});

sql_mut_test('UPDATE SET supports dot-notation for nested fields', function() use ($store) {
    $res = $store->query("UPDATE users SET profile.status = 'suspended' WHERE id = 2");
    assert($res === 1);

    $users = $store->get('users');
    assert($users[1]['profile']['status'] === 'suspended');
});

sql_mut_test('DELETE FROM deletes matching records', function() use ($store) {
    $res = $store->query("DELETE FROM users WHERE id = 3");
    assert($res === 1, 'Should delete 1 row');

    $users = $store->get('users');
    assert(count($users) === 3); // Alice (1), Bob (2), Diana (4)
    assert($users[2]['name'] === 'Diana');
});

sql_mut_test('PITR history records and rolls back SQL mutations successfully', function() use ($tmp) {
    $db = "{$tmp}/pitr_sql.json";
    if (file_exists($db)) {
        unlink($db);
    }
    if (file_exists("{$db}.journal")) {
        unlink("{$db}.journal");
    }

    $s = new Store($db);
    $s->set('users', [
        ['id' => 1, 'name' => 'Alice'],
    ]);

    // Check history count
    assert(count($s->history()) === 1, 'Initial set of users collection');

    // SQL Insert
    $s->query("INSERT INTO users (id, name) VALUES (2, 'Bob')");
    $h = $s->history();
    assert(count($h) === 2, 'History should record the SQL insert');
    
    $last_rev = $h[1];
    assert($last_rev['op'] === 'set');
    assert($last_rev['path'] === 'users');

    // SQL Update
    $s->query("UPDATE users SET name = 'Alice Updated' WHERE id = 1");
    assert(count($s->history()) === 3, 'History should record SQL update');

    // SQL Delete
    $s->query("DELETE FROM users WHERE id = 2");
    assert(count($s->history()) === 4, 'History should record SQL delete');

    // Verify current state before rollback
    $users = $s->get('users');
    assert(count($users) === 1);
    assert($users[0]['name'] === 'Alice Updated');

    // Rollback to revision 1 (after initial seed, before SQL modifications)
    $s->rollbackTo(1);
    
    $users_reverted = $s->get('users');
    assert(count($users_reverted) === 1, 'Should only have 1 user');
    assert($users_reverted[0]['name'] === 'Alice', 'Should revert to original name Alice');
});

echo "\n══════════════════════════════════\n";
echo "  ✅ Passed: {$pass}  ❌ Failed: {$fail}\n";
echo "══════════════════════════════════\n";

// Cleanup
array_map('unlink', glob("{$tmp}/*"));
rmdir($tmp);

exit($fail > 0 ? 1 : 0);
