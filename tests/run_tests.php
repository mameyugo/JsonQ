<?php
/**
 * JsonQ Test Suite
 *
 * Comprehensive integration tests for the JsonQ PHP extension.
 * Run: php -d "extension=path/to/libjsonq.so" tests/run_tests.php
 */

require_once __DIR__ . '/integration/helpers.php';

// ═══════════════════════════════════════════
echo "\n🧪 JsonQ Test Suite v" . jsonq_version() . "\n";
echo str_repeat('═', 50) . "\n";

// ── Module ──
echo "\n📦 Module\n";

test('jsonq_version returns string', function() {
    assert_eq('0.6.0', jsonq_version());
});

test('JsonQ\\Store class exists', function() {
    assert_true(class_exists('JsonQ\\Store'));
});

test('Constructor creates file', function() {
    $path = tempnam(sys_get_temp_dir(), 'JsonQ_') . '.json';
    new \JsonQ\Store($path);
    assert_true(file_exists($path));
    assert_eq('{}', trim(file_get_contents($path)));
    unlink($path);
});

// ── CRUD ──
echo "\n📝 CRUD Operations\n";

test('set and get string', function() {
    $s = fresh_store();
    $s->set('name', 'Alice');
    assert_eq('Alice', $s->get('name'));
});

test('set and get integer', function() {
    $s = fresh_store();
    $s->set('count', 42);
    assert_eq(42, $s->get('count'));
});

test('set and get float', function() {
    $s = fresh_store();
    $s->set('pi', 3.14);
    assert_eq(3.14, $s->get('pi'));
});

test('set and get boolean', function() {
    $s = fresh_store();
    $s->set('active', true);
    assert_true($s->get('active'));
    $s->set('deleted', false);
    assert_false($s->get('deleted'));
});

test('set and get null', function() {
    $s = fresh_store();
    $s->set('empty', null);
    assert_null($s->get('empty'));
});

test('set and get array', function() {
    $s = fresh_store();
    $s->set('items', [1, 2, 3]);
    assert_eq([1, 2, 3], $s->get('items'));
});

test('set and get nested object', function() {
    $s = fresh_store();
    $s->set('user', ['name' => 'Bob', 'age' => 30]);
    $user = $s->get('user');
    assert_eq('Bob', $user['name']);
    assert_eq(30, $user['age']);
});

test('dot-notation get', function() {
    $s = fresh_store();
    $s->set('config', ['db' => ['host' => 'localhost', 'port' => 3306]]);
    assert_eq('localhost', $s->get('config.db.host'));
    assert_eq(3306, $s->get('config.db.port'));
});

test('dot-notation set creates intermediates', function() {
    $s = fresh_store();
    $s->set('a.b.c', 'deep');
    assert_eq('deep', $s->get('a.b.c'));
});

test('has returns correct values', function() {
    $s = fresh_store();
    $s->set('exists', 'yes');
    assert_true($s->has('exists'));
    assert_false($s->has('nope'));
});

test('count for arrays and objects', function() {
    $s = fresh_store();
    $s->set('arr', [1, 2, 3, 4, 5]);
    $s->set('obj', ['a' => 1, 'b' => 2]);
    assert_eq(5, $s->count('arr'));
    assert_eq(2, $s->count('obj'));
});

test('keys returns object keys', function() {
    $s = fresh_store();
    $s->set('x', 1);
    $s->set('y', 2);
    $keys = $s->keys('');
    assert_true(in_array('x', $keys));
    assert_true(in_array('y', $keys));
});

test('remove deletes value', function() {
    $s = fresh_store();
    $s->set('temp', 'data');
    assert_true($s->has('temp'));
    $s->remove('temp');
    assert_false($s->has('temp'));
});

test('push appends to array', function() {
    $s = fresh_store();
    $s->set('list', [1, 2]);
    $s->push('list', 3);
    assert_eq([1, 2, 3], $s->get('list'));
});

test('merge deep merges objects', function() {
    $s = fresh_store();
    $s->set('conf', ['a' => 1, 'b' => ['x' => 10]]);
    $s->merge('conf', ['b' => ['y' => 20], 'c' => 3]);
    assert_eq(1, $s->get('conf.a'));
    assert_eq(10, $s->get('conf.b.x'));
    assert_eq(20, $s->get('conf.b.y'));
    assert_eq(3, $s->get('conf.c'));
});

test('increment and decrement', function() {
    $s = fresh_store();
    $s->set('counter', 10);
    $s->increment('counter');
    assert_eq(11.0, $s->get('counter'));
    $s->increment('counter', 5.0);
    assert_eq(16.0, $s->get('counter'));
    $s->decrement('counter', 3.0);
    assert_eq(13.0, $s->get('counter'));
});

// ── MongoDB Queries ──
echo "\n🔍 MongoDB-Style Queries\n";

function store_with_users(): \JsonQ\Store {
    $s = fresh_store();
    $s->set('users', [
        ['name' => 'Alice',   'age' => 30, 'city' => 'NYC',    'role' => 'admin',  'score' => 95],
        ['name' => 'Bob',     'age' => 25, 'city' => 'LA',     'role' => 'user',   'score' => 82],
        ['name' => 'Charlie', 'age' => 35, 'city' => 'NYC',    'role' => 'admin',  'score' => 88],
        ['name' => 'Diana',   'age' => 28, 'city' => 'Chicago','role' => 'user',   'score' => 91],
        ['name' => 'Eve',     'age' => 22, 'city' => 'LA',     'role' => 'viewer', 'score' => 76],
    ]);
    return $s;
}

test('find equality', function() {
    $s = store_with_users();
    $r = $s->find('users', ['city' => 'NYC']);
    assert_count(2, $r);
});

test('find $gt', function() {
    $s = store_with_users();
    $r = $s->find('users', ['age' => ['$gt' => 30]]);
    assert_count(1, $r);
    assert_eq('Charlie', $r[0]['name']);
});

test('find $gte', function() {
    $s = store_with_users();
    $r = $s->find('users', ['age' => ['$gte' => 30]]);
    assert_count(2, $r);
});

test('find $lt', function() {
    $s = store_with_users();
    $r = $s->find('users', ['age' => ['$lt' => 25]]);
    assert_count(1, $r);
});

test('find $lte', function() {
    $s = store_with_users();
    $r = $s->find('users', ['age' => ['$lte' => 25]]);
    assert_count(2, $r);
});

test('find $ne', function() {
    $s = store_with_users();
    $r = $s->find('users', ['role' => ['$ne' => 'admin']]);
    assert_count(3, $r);
});

test('find $in', function() {
    $s = store_with_users();
    $r = $s->find('users', ['role' => ['$in' => ['admin', 'viewer']]]);
    assert_count(3, $r);
});

test('find $nin', function() {
    $s = store_with_users();
    $r = $s->find('users', ['city' => ['$nin' => ['NYC', 'LA']]]);
    assert_count(1, $r);
    assert_eq('Diana', $r[0]['name']);
});

test('find $contains', function() {
    $s = store_with_users();
    $r = $s->find('users', ['name' => ['$contains' => 'li']]);
    assert_count(2, $r); // Alice, Charlie
});

test('find $startsWith', function() {
    $s = store_with_users();
    $r = $s->find('users', ['name' => ['$startsWith' => 'A']]);
    assert_count(1, $r);
});

test('find $endsWith', function() {
    $s = store_with_users();
    $r = $s->find('users', ['name' => ['$endsWith' => 'e']]);
    assert_count(3, $r); // Alice, Charlie, Eve
});

test('find $or', function() {
    $s = store_with_users();
    $r = $s->find('users', ['$or' => [['city' => 'Chicago'], ['role' => 'viewer']]]);
    assert_count(2, $r);
});

test('find $and implicit', function() {
    $s = store_with_users();
    $r = $s->find('users', ['city' => 'NYC', 'role' => 'admin']);
    assert_count(2, $r);
});

test('find $not', function() {
    $s = store_with_users();
    $r = $s->find('users', ['$not' => ['role' => 'admin']]);
    assert_count(3, $r);
});

test('find complex combined', function() {
    $s = store_with_users();
    $r = $s->find('users', ['age' => ['$gte' => 25], 'score' => ['$gt' => 85]]);
    assert_count(3, $r); // Alice(30,95), Charlie(35,88), Diana(28,91)
});

test('findOne returns first match', function() {
    $s = store_with_users();
    $r = $s->findOne('users', ['city' => 'NYC']);
    assert_eq('Alice', $r['name']);
});

test('findOne returns null for no match', function() {
    $s = store_with_users();
    $r = $s->findOne('users', ['city' => 'Mars']);
    assert_null($r);
});

// ── Fluent Queries ──
echo "\n🔗 Fluent Queries\n";

test('fluent where + order + limit', function() {
    $s = store_with_users();
    $r = $s->executeQuery('users', [
        'where' => [['field' => 'age', 'op' => '>=', 'value' => 25]],
        'order_by' => ['field' => 'age', 'direction' => 'desc'],
        'limit' => 2,
    ]);
    assert_count(2, $r);
    assert_eq('Charlie', $r[0]['name']);
    assert_eq('Alice', $r[1]['name']);
});

test('fluent between', function() {
    $s = store_with_users();
    $r = $s->executeQuery('users', [
        'where' => [['field' => 'age', 'op' => 'between', 'value' => [25, 30]]],
    ]);
    assert_count(3, $r); // Bob(25), Alice(30), Diana(28)
});

test('fluent select projection', function() {
    $s = store_with_users();
    $r = $s->executeQuery('users', [
        'where' => [['field' => 'city', 'op' => '=', 'value' => 'NYC']],
        'select' => ['name', 'age'],
    ]);
    assert_count(2, $r);
    assert_true(isset($r[0]['name']));
    assert_true(isset($r[0]['age']));
    assert_false(isset($r[0]['city']));
});

test('fluent offset pagination', function() {
    $s = store_with_users();
    $all = $s->executeQuery('users', ['order_by' => ['field' => 'name', 'direction' => 'asc']]);
    $page2 = $s->executeQuery('users', [
        'order_by' => ['field' => 'name', 'direction' => 'asc'],
        'offset' => 2,
        'limit' => 2,
    ]);
    assert_count(2, $page2);
    assert_eq($all[2]['name'], $page2[0]['name']);
});

test('fluent contains', function() {
    $s = store_with_users();
    $r = $s->executeQuery('users', [
        'where' => [['field' => 'name', 'op' => 'contains', 'value' => 'ob']],
    ]);
    assert_count(1, $r);
    assert_eq('Bob', $r[0]['name']);
});

// ── Aggregation ──
echo "\n📊 Aggregation\n";

test('aggregate sum', function() {
    $s = store_with_users();
    assert_eq(140.0, $s->aggregate('users', 'age', 'sum'));
});

test('aggregate avg', function() {
    $s = store_with_users();
    assert_eq(28.0, $s->aggregate('users', 'age', 'avg'));
});

test('aggregate min', function() {
    $s = store_with_users();
    assert_eq(22.0, $s->aggregate('users', 'age', 'min'));
});

test('aggregate max', function() {
    $s = store_with_users();
    assert_eq(35.0, $s->aggregate('users', 'age', 'max'));
});

test('aggregate count', function() {
    $s = store_with_users();
    assert_eq(5, $s->aggregate('users', 'age', 'count'));
});

test('groupBy', function() {
    $s = store_with_users();
    $g = $s->groupBy('users', 'city');
    assert_count(2, $g['NYC']);
    assert_count(2, $g['LA']);
    assert_count(1, $g['Chicago']);
});

test('pluck single field', function() {
    $s = store_with_users();
    $names = $s->pluck('users', ['name']);
    assert_count(5, $names);
    assert_eq('Alice', $names[0]);
});

test('pluck multiple fields', function() {
    $s = store_with_users();
    $r = $s->pluck('users', ['name', 'city']);
    assert_count(5, $r);
    assert_eq('Alice', $r[0]['name']);
    assert_eq('NYC', $r[0]['city']);
});

// ── Indexes ──
echo "\n⚡ Indexes\n";

test('createIndex and indexLookup', function() {
    $s = store_with_users();
    $s->createIndex('users', 'city');
    $r = $s->indexLookup('users', 'city', 'NYC');
    assert_count(2, $r);
    assert_eq('Alice', $r[0]['name']);
});

test('find uses index automatically', function() {
    $s = store_with_users();
    $s->createIndex('users', 'role');
    $r = $s->find('users', ['role' => 'admin']);
    assert_count(2, $r);
});

test('listIndexes', function() {
    $s = store_with_users();
    $s->createIndex('users', 'city');
    $s->createIndex('users', 'role');
    $list = $s->listIndexes();
    assert_true(count($list) >= 2);
});

test('dropIndex', function() {
    $s = store_with_users();
    $s->createIndex('users', 'city');
    assert_true($s->dropIndex('users'));
    assert_false($s->dropIndex('nonexistent'));
});

test('dropAllIndexes', function() {
    $s = store_with_users();
    $s->createIndex('users', 'city');
    $s->createIndex('users', 'role');
    $count = $s->dropAllIndexes();
    assert_eq(1, $count); // Both indexes are under 'users' collection
});

test('createCompoundIndex', function() {
    $s = store_with_users();
    $s->createCompoundIndex('users', ['city', 'role']);
    $list = $s->listIndexes();
    $compound = array_filter($list, fn($i) => $i['type'] === 'compound');
    assert_true(count($compound) > 0);
});

// ── Validation ──
echo "\n✅ Schema Validation\n";

test('validate type check pass', function() {
    $s = fresh_store();
    $s->set('name', 'Alice');
    $r = $s->validate('name', ['type' => 'string']);
    assert_true($r['valid']);
    assert_eq(0, $r['error_count']);
});

test('validate type check fail', function() {
    $s = fresh_store();
    $s->set('name', 42);
    $r = $s->validate('name', ['type' => 'string']);
    assert_false($r['valid']);
    assert_eq(1, $r['error_count']);
});

test('validate required fields', function() {
    $s = fresh_store();
    $s->set('user', ['name' => 'Alice']);
    $r = $s->validate('user', [
        'type' => 'object',
        'required' => ['name', 'email'],
    ]);
    assert_false($r['valid']);
    assert_eq(1, $r['error_count']);
});

test('validate string constraints', function() {
    $s = fresh_store();
    $s->set('short', 'ab');
    $r = $s->validate('short', ['type' => 'string', 'minLength' => 5]);
    assert_false($r['valid']);
});

test('validate number range', function() {
    $s = fresh_store();
    $s->set('age', 200);
    $r = $s->validate('age', ['type' => 'integer', 'min' => 0, 'max' => 150]);
    assert_false($r['valid']);
});

test('validate email format', function() {
    $s = fresh_store();
    $s->set('email', 'invalid');
    $r = $s->validate('email', ['type' => 'string', 'format' => 'email']);
    assert_false($r['valid']);
    $s->set('email', 'user@example.com');
    $r = $s->validate('email', ['type' => 'string', 'format' => 'email']);
    assert_true($r['valid']);
});

test('validate enum', function() {
    $s = fresh_store();
    $s->set('status', 'pending');
    $r = $s->validate('status', ['type' => 'string', 'enum' => ['active', 'pending', 'closed']]);
    assert_true($r['valid']);
    $s->set('status', 'unknown');
    $r = $s->validate('status', ['type' => 'string', 'enum' => ['active', 'pending', 'closed']]);
    assert_false($r['valid']);
});

test('validate nested properties', function() {
    $s = fresh_store();
    $s->set('user', ['name' => 'Alice', 'age' => 30]);
    $r = $s->validate('user', [
        'type' => 'object',
        'properties' => [
            'name' => ['type' => 'string'],
            'age' => ['type' => 'integer', 'min' => 0],
        ],
    ]);
    assert_true($r['valid']);
});

test('validateCollection', function() {
    $s = store_with_users();
    $r = $s->validateCollection('users', [
        'type' => 'object',
        'required' => ['name', 'age'],
        'properties' => [
            'name' => ['type' => 'string'],
            'age' => ['type' => 'integer'],
        ],
    ]);
    assert_true($r['valid']);
    assert_eq(5, $r['total_items']);
    assert_eq(0, $r['invalid_items']);
});

// ── Utilities ──
echo "\n🔧 Utilities\n";

test('stats returns metadata', function() {
    $s = store_with_users();
    $st = $s->stats();
    assert_true(isset($st['file_path']));
    assert_true(isset($st['file_size']));
    assert_true(isset($st['file_size_h']));
    assert_true($st['key_count'] > 0);
});

test('backup and restore', function() {
    $s = fresh_store();
    $s->set('data', 'original');
    $backup = $s->backup();
    assert_true(file_exists($backup));
    $s->set('data', 'modified');
    assert_eq('modified', $s->get('data'));
    $s->restore($backup);
    assert_eq('original', $s->get('data'));
    unlink($backup);
});

test('data persists across instances', function() {
    $path = tempnam(sys_get_temp_dir(), 'JsonQ_persist_') . '.json';
    $s1 = new \JsonQ\Store($path);
    $s1->set('persistent', true);
    unset($s1);
    $s2 = new \JsonQ\Store($path);
    assert_true($s2->get('persistent'));
    unlink($path);
});

// ── Options ──
echo "\n⚙️  Options\n";

test('setOption pretty_print', function() {
    $s = fresh_store();
    $s->setOption('pretty', true);
    assert_true($s->getOption('pretty'));
    $s->set('test', 'value');
    // Pretty format should have newlines
    $s->setOption('pretty', false);
    assert_false($s->getOption('pretty'));
});

test('setOption fsync', function() {
    $s = fresh_store();
    assert_false($s->getOption('fsync'));
    $s->setOption('fsync', true);
    assert_true($s->getOption('fsync'));
    $s->set('key', 'val'); // Should still work with fsync
    assert_eq('val', $s->get('key'));
});

// ── Transactions ──
echo "\n🔄 Transactions\n";

test('transaction commit', function() {
    $s = fresh_store();
    $s->set('value', 'original');
    $s->beginTransaction();
    assert_true($s->inTransaction());
    $s->set('value', 'modified');
    $s->set('new_key', 'added');
    assert_eq('modified', $s->get('value'));
    $s->commit();
    assert_false($s->inTransaction());
    assert_eq('modified', $s->get('value'));
    assert_eq('added', $s->get('new_key'));
});

test('transaction rollback', function() {
    $s = fresh_store();
    $s->set('value', 'original');
    $s->beginTransaction();
    $s->set('value', 'modified');
    $s->set('added', true);
    $s->rollback();
    assert_false($s->inTransaction());
    assert_eq('original', $s->get('value'));
    assert_false($s->has('added'));
});

test('transaction multiple writes are atomic', function() {
    $s = fresh_store();
    $s->set('counter', 0);
    $s->beginTransaction();
    for ($i = 0; $i < 10; $i++) {
        $s->increment('counter');
    }
    $s->set('bulk', 'data');
    $s->commit();
    assert_eq(10.0, $s->get('counter'));
    assert_eq('data', $s->get('bulk'));
});

// ── Batch Operations ──
echo "\n📦 Batch Operations\n";

test('setMany writes multiple keys at once', function() {
    $s = fresh_store();
    $count = $s->setMany([
        'name' => 'Alice',
        'age' => 30,
        'config.debug' => true,
    ]);
    assert_eq(3, $count);
    assert_eq('Alice', $s->get('name'));
    assert_eq(30, $s->get('age'));
    assert_true($s->get('config.debug'));
});

test('removeMany deletes multiple paths', function() {
    $s = fresh_store();
    $s->setMany(['a' => 1, 'b' => 2, 'c' => 3, 'd' => 4]);
    $count = $s->removeMany(['a', 'c', 'nonexistent']);
    assert_eq(2, $count);
    assert_false($s->has('a'));
    assert_true($s->has('b'));
    assert_false($s->has('c'));
    assert_true($s->has('d'));
});

// ── Import/Export ──
echo "\n📥 Import/Export\n";

test('toJson export', function() {
    $s = fresh_store();
    $s->set('key', 'value');
    $json = $s->toJson();
    $decoded = json_decode($json, true);
    assert_eq('value', $decoded['key']);
});

test('toJson pretty export', function() {
    $s = fresh_store();
    $s->set('key', 'value');
    $json = $s->toJson(true);
    assert_true(str_contains($json, "\n"));
});

test('fromJson import', function() {
    $s = fresh_store();
    $s->fromJson('{"imported":"data","count":42}');
    assert_eq('data', $s->get('imported'));
    assert_eq(42, $s->get('count'));
});

// ── Extra Methods ──
echo "\n🔧 Extra Methods\n";

test('getAll returns everything', function() {
    $s = fresh_store();
    $s->setMany(['a' => 1, 'b' => 'two', 'c' => true]);
    $all = $s->getAll();
    assert_eq(1, $all['a']);
    assert_eq('two', $all['b']);
    assert_true($all['c']);
});

test('clear removes all data', function() {
    $s = fresh_store();
    $s->setMany(['a' => 1, 'b' => 2, 'c' => 3]);
    assert_eq(3, count($s->keys('')));
    $s->clear();
    assert_eq(0, count($s->keys('')));
});

test('search finds keyword across all fields', function() {
    $s = store_with_users();
    $r = $s->search('users', 'alice');
    assert_count(1, $r);
    assert_eq('Alice', $r[0]['name']);
});

test('search is case-insensitive', function() {
    $s = store_with_users();
    $r = $s->search('users', 'NYC');
    assert_count(2, $r);
});

test('search finds in nested fields', function() {
    $s = fresh_store();
    $s->set('items', [
        ['title' => 'PHP Book', 'meta' => ['tags' => 'programming, web']],
        ['title' => 'Cooking', 'meta' => ['tags' => 'food, recipes']],
    ]);
    $r = $s->search('items', 'programming');
    assert_count(1, $r);
    assert_eq('PHP Book', $r[0]['title']);
});

// ═══════════════════════════════════════════
// ═══════════════════════════════════════════
print_summary();
