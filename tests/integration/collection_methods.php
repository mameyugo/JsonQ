<?php
/**
 * Collection Methods Tests
 */

require_once __DIR__ . '/helpers.php';

echo "\n🧪 Collection Methods Tests\n";
echo str_repeat('═', 50) . "\n";

function setup_test_data() {
    $s = fresh_store();
    $s->set('users', [
        ['id' => 1, 'name' => 'Alice', 'email' => 'alice@test.com', 'age' => 30, 'city' => 'NYC'],
        ['id' => 2, 'name' => 'Bob', 'email' => 'bob@test.com', 'age' => 25, 'city' => 'LA'],
        ['id' => 3, 'name' => 'Charlie', 'email' => 'charlie@test.com', 'age' => 35, 'city' => 'NYC'],
    ]);
    return $s;
}

// ── except() ──
echo "\n🚫 except()\n";

test('except excludes specified fields', function() {
    $s = setup_test_data();
    $results = $s->executeQuery('users', [
        'except' => ['email', 'city']
    ]);
    
    assert_true(count($results) === 3, 'Count should be 3');
    assert_true(isset($results[0]['name']), 'Name should strictly exist');
    assert_true(isset($results[0]['age']), 'Age should strictly exist');
    assert_false(isset($results[0]['email']), 'Email should be excluded');
    assert_false(isset($results[0]['city']), 'City should be excluded');
});

test('except with single field', function() {
    $s = setup_test_data();
    $results = $s->executeQuery('users', [
        'except' => ['email']
    ]);
    
    assert_true(isset($results[0]['name']), 'Name should exist');
    assert_false(isset($results[0]['email']), 'Email should be excluded');
});

// ── column() ──
echo "\n📊 column()\n";

test('column extracts field values', function() {
    $s = setup_test_data();
    $names = $s->column('users', 'name');
    
    // Sort to ensure order doesn't fail test if implementation changes
    // But implementation preserves order, so we can check directly
    assert_true($names === ['Alice', 'Bob', 'Charlie'], 'Names match');
});

test('column with missing field returns empty', function() {
    $s = setup_test_data();
    $missing = $s->column('users', 'nonexistent');
    
    assert_eq([], $missing);
});

test('column with numbers', function() {
    $s = setup_test_data();
    $ages = $s->column('users', 'age');
    
    assert_true($ages === [30, 25, 35], 'Ages match');
});

// ── chunk() ──
echo "\n📦 chunk()\n";

test('chunk splits into equal groups', function() {
    $s = setup_test_data();
    $chunks = $s->chunk('users', 2);
    
    assert_true(count($chunks) === 2, 'Should have 2 chunks');
    assert_true(count($chunks[0]) === 2, 'First chunk size 2');
    assert_true(count($chunks[1]) === 1, 'Second chunk size 1');
});

test('chunk with size 1', function() {
    $s = setup_test_data();
    $chunks = $s->chunk('users', 1);
    
    assert_true(count($chunks) === 3, 'Should have 3 chunks');
    assert_true(count($chunks[0]) === 1, 'Chunk size 1');
});

test('chunk larger than collection', function() {
    $s = setup_test_data();
    $chunks = $s->chunk('users', 10);
    
    assert_true(count($chunks) === 1, 'Should have 1 chunk');
    assert_true(count($chunks[0]) === 3, 'Chunk contains all items');
});

test('chunk with zero throws error', function() {
    $s = setup_test_data();
    try {
        $s->chunk('users', 0);
        assert_true(false, 'Should have thrown exception');
    } catch (Exception $e) {
        assert_true(str_contains($e->getMessage(), 'greater than 0'), 'Exception message check');
    }
});

// ── implode() ──
echo "\n🔗 implode()\n";

test('implode joins string values', function() {
    $s = setup_test_data();
    $result = $s->implode('users', 'name', ', ');
    
    assert_eq('Alice, Bob, Charlie', $result);
});

test('implode with different separator', function() {
    $s = setup_test_data();
    $result = $s->implode('users', 'name', ' | ');
    
    assert_eq('Alice | Bob | Charlie', $result);
});

test('implode with numbers', function() {
    $s = setup_test_data();
    $result = $s->implode('users', 'age', ',');
    
    assert_eq('30,25,35', $result);
});

// ── keys() ──
echo "\n🔑 keys()\n";

test('keys returns object keys', function() {
    $s = setup_test_data();
    $keys = $s->keys('users.0');
    
    assert_true(in_array('id', $keys), 'Has id');
    assert_true(in_array('name', $keys), 'Has name');
    assert_true(in_array('email', $keys), 'Has email');
    assert_true(in_array('age', $keys), 'Has age');
    assert_true(in_array('city', $keys), 'Has city');
    assert_true(count($keys) === 5, 'Count is 5');
});

test('keys on non-object returns empty', function() {
    $s = fresh_store();
    $s->set('value', 'string');
    $keys = $s->keys('value');
    
    assert_eq([], $keys);
});

// ── values() ──
echo "\n💎 values()\n";

test('values returns object values', function() {
    $s = setup_test_data();
    $values = $s->values('users.0');
    
    assert_true(in_array('Alice', $values), 'Has Alice');
    assert_true(in_array(30, $values), 'Has 30');
    assert_true(in_array('NYC', $values), 'Has NYC');
    assert_true(count($values) === 5, 'Count is 5');
});

test('values on non-object returns empty', function() {
    $s = fresh_store();
    $s->set('value', 'string');
    $values = $s->values('value');
    
    assert_eq([], $values);
});

// ── Integration Test ──
echo "\n🔄 Integration Test\n";

test('combine multiple collection methods', function() {
    // Setup fresh data first
    $s = setup_test_data();
    
    // 1. Get only certain fields
    $results = $s->executeQuery('users', [
        'where' => [['field' => 'age', 'op' => '>', 'value' => 25]],
        'except' => ['email']
    ]);
    
    // Results should be Alice (30) and Charlie (35)
    assert_true(count($results) === 2, 'Alice and Charlie'); 
    assert_false(isset($results[0]['email']), 'Email excluded');
    
    // 2. Extract names
    $names = $s->column('users', 'name');
    assert_true(count($names) === 3, 'All names');
    
    // 3. Join them
    $nameList = $s->implode('users', 'name', ' & ');
    assert_eq('Alice & Bob & Charlie', $nameList);
    
    // 4. Chunk the results
    $chunks = $s->chunk('users', 2);
    assert_true(count($chunks) === 2, 'Chunk count');
});

print_summary();
