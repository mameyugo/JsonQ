<?php
/**
 * JsonQ v0.8.0 — Streaming Tests
 *
 * Este script prueba las funcionalidades de stream memory-efficient:
 * Requiere: JsonQ extension >= 0.8.0
 * Usage: php -d "extension=/path/to/jsonq.so" tests/integration/test_streaming.php
 */

// ══════════════════════════════════════════════════════════
// HELPERS
// ══════════════════════════════════════════════════════════

function create_large_json_file(string $path, int $count = 10000): void
{
    $users = [];
    for ($i = 1; $i <= $count; $i++) {
        $users[] = [
            'id'     => $i,
            'name'   => "User_{$i}",
            'age'    => rand(18, 80),
            'score'  => rand(0, 100),
            'active' => ($i % 3 !== 0),
            'role'   => ['admin', 'editor', 'viewer'][$i % 3],
            'city'   => ['NYC', 'LA', 'Chicago', 'Miami'][$i % 4],
        ];
    }
    file_put_contents($path, json_encode(['users' => $users, 'meta' => ['total' => $count]]));
}

function create_nested_json(string $path): void
{
    $data = [
        'company' => [
            'name' => 'Acme Corp',
            'departments' => [
                [
                    'id'    => 1,
                    'name'  => 'Engineering',
                    'staff' => [
                        ['id' => 101, 'name' => 'Alice', 'level' => 'senior'],
                        ['id' => 102, 'name' => 'Bob', 'level' => 'junior'],
                    ]
                ],
                [
                    'id'    => 2,
                    'name'  => 'Marketing',
                    'staff' => [
                        ['id' => 201, 'name' => 'Carol', 'level' => 'manager'],
                    ]
                ],
            ]
        ]
    ];
    file_put_contents($path, json_encode($data, JSON_PRETTY_PRINT));
}

function create_root_array_json(string $path, int $count = 100): void
{
    $items = [];
    for ($i = 1; $i <= $count; $i++) {
        $items[] = ['id' => $i, 'value' => "item_{$i}"];
    }
    file_put_contents($path, json_encode($items));
}

$tmpDir = sys_get_temp_dir() . '/jsonq_stream_tests_' . getmypid();
if (!is_dir($tmpDir)) {
    mkdir($tmpDir, 0755, true);
}

$passCount = 0;
$failCount = 0;

function stream_test(string $name, callable $fn): void
{
    global $passCount, $failCount;
    try {
        $fn();
        echo "  ✅ {$name}\n";
        $passCount++;
    } catch (\Throwable $e) {
        echo "  ❌ {$name}\n     {$e->getMessage()}\n";
        $failCount++;
    }
}

function stream_assert(bool $cond, string $msg = ''): void
{
    if (!$cond) throw new \RuntimeException($msg ?: 'Assertion failed');
}

function stream_assert_eq(mixed $expected, mixed $actual, string $msg = ''): void
{
    if ($expected !== $actual) {
        throw new \RuntimeException(
            ($msg ? "{$msg}: " : '') .
            "Expected " . json_encode($expected) . ", got " . json_encode($actual)
        );
    }
}


// ══════════════════════════════════════════════════════════
// SECTION 1: JSON Pointer Parsing
// ══════════════════════════════════════════════════════════
echo "\n📍 JSON Pointer RFC 6901\n";

stream_test('stream root pointer "" returns root items', function() use ($tmpDir) {
    $path = "{$tmpDir}/root_array.json";
    create_root_array_json($path, 50);
    $store = new JsonQ\Store($path);
    $items = $store->stream('');
    stream_assert_eq(50, count($items), 'root array count');
});

stream_test('stream /users pointer returns users array', function() use ($tmpDir) {
    $path = "{$tmpDir}/large.json";
    create_large_json_file($path, 1000);
    $store = new JsonQ\Store($path);
    $items = $store->stream('/users');
    stream_assert_eq(1000, count($items), 'users count');
    stream_assert(isset($items[0]['id']), 'first item has id');
    stream_assert(isset($items[0]['name']), 'first item has name');
});

stream_test('stream nested pointer /company/departments returns departments', function() use ($tmpDir) {
    $path = "{$tmpDir}/nested.json";
    create_nested_json($path);
    $store = new JsonQ\Store($path);
    $depts = $store->stream('/company/departments');
    stream_assert_eq(2, count($depts), 'departments count');
    stream_assert_eq('Engineering', $depts[0]['name'], 'first dept name');
});

stream_test('stream deep nested pointer /company/departments/0/staff', function() use ($tmpDir) {
    $path = "{$tmpDir}/nested.json";
    create_nested_json($path);
    $store = new JsonQ\Store($path);
    $staff = $store->stream('/company/departments/0/staff');
    stream_assert_eq(2, count($staff), 'staff count');
    stream_assert_eq('Alice', $staff[0]['name'], 'first staff name');
});

stream_test('stream on object (not array) at pointer', function() use ($tmpDir) {
    $path = "{$tmpDir}/nested.json";
    create_nested_json($path);
    $store = new JsonQ\Store($path);
    $items = $store->stream('/company'); // company is an object
    stream_assert(is_array($items), 'always returns array');
    stream_assert_eq(0, count($items), 'object returns empty stream');
});

stream_test('stream pointer with tilde escaping (~0 ~1)', function() use ($tmpDir) {
    $path = "{$tmpDir}/tilde.json";
    // JSON key "a/b" and "a~b"
    $data = ['a/b' => [['id' => 1]], 'a~b' => [['id' => 2]]];
    file_put_contents($path, json_encode($data));
    $store = new JsonQ\Store($path);

    $items1 = $store->stream('/a~1b'); // ~1 = /
    stream_assert_eq(1, count($items1), 'tilde-1 escaped pointer');
    stream_assert_eq(1, $items1[0]['id']);

    $items2 = $store->stream('/a~0b'); // ~0 = ~
    stream_assert_eq(1, count($items2), 'tilde-0 escaped pointer');
    stream_assert_eq(2, $items2[0]['id']);
});


// ══════════════════════════════════════════════════════════
// SECTION 2: Basic Stream Operations
// ══════════════════════════════════════════════════════════
echo "\n🌊 Basic Stream Operations\n";

stream_test('stream returns all items without filter', function() use ($tmpDir) {
    $path = "{$tmpDir}/large.json";
    $store = new JsonQ\Store($path);
    $items = $store->stream('/users');
    stream_assert_eq(1000, count($items));
});

stream_test('stream items have correct structure', function() use ($tmpDir) {
    $path = "{$tmpDir}/large.json";
    $store = new JsonQ\Store($path);
    $items = $store->stream('/users');
    $first = $items[0];
    stream_assert(isset($first['id']), 'id present');
    stream_assert(isset($first['name']), 'name present');
    stream_assert(isset($first['age']), 'age present');
    stream_assert(isset($first['score']), 'score present');
    stream_assert(isset($first['active']), 'active present');
    stream_assert(isset($first['role']), 'role present');
});

stream_test('stream items are in correct order', function() use ($tmpDir) {
    $path = "{$tmpDir}/large.json";
    $store = new JsonQ\Store($path);
    $items = $store->stream('/users');
    stream_assert_eq(1, $items[0]['id'], 'first item id');
    stream_assert_eq(2, $items[1]['id'], 'second item id');
    stream_assert_eq(1000, $items[999]['id'], 'last item id');
});

stream_test('stream on empty file works', function() use ($tmpDir) {
    $path = "{$tmpDir}/empty_stream.json";
    file_put_contents($path, json_encode(['items' => []]));
    $store = new JsonQ\Store($path);
    $items = $store->stream('/items');
    stream_assert_eq([], $items, 'empty array stream');
});


// ══════════════════════════════════════════════════════════
// SECTION 3: Stream with Filters
// ══════════════════════════════════════════════════════════
echo "\n🔍 Stream with Filters\n";

stream_test('stream filter with $eq', function() use ($tmpDir) {
    $path = "{$tmpDir}/large.json";
    $store = new JsonQ\Store($path);
    $items = $store->stream('/users', ['role' => ['$eq' => 'admin']]);
    foreach ($items as $item) {
        stream_assert_eq('admin', $item['role'], 'all items must be admin');
    }
    stream_assert(count($items) > 0, 'must return some admins');
});

stream_test('stream filter with $gt', function() use ($tmpDir) {
    $path = "{$tmpDir}/large.json";
    $store = new JsonQ\Store($path);
    $items = $store->stream('/users', ['age' => ['$gt' => 60]]);
    foreach ($items as $item) {
        stream_assert($item['age'] > 60, "age {$item['age']} must be > 60");
    }
});

stream_test('stream filter with $gte $lte (range)', function() use ($tmpDir) {
    $path = "{$tmpDir}/large.json";
    $store = new JsonQ\Store($path);
    $items = $store->stream('/users', ['score' => ['$gte' => 90, '$lte' => 100]]);
    foreach ($items as $item) {
        stream_assert($item['score'] >= 90 && $item['score'] <= 100);
    }
});

stream_test('stream filter with $in', function() use ($tmpDir) {
    $path = "{$tmpDir}/large.json";
    $store = new JsonQ\Store($path);
    $items = $store->stream('/users', ['city' => ['$in' => ['NYC', 'LA']]]);
    foreach ($items as $item) {
        stream_assert(in_array($item['city'], ['NYC', 'LA']));
    }
    stream_assert(count($items) > 0);
});

// ══════════════════════════════════════════════════════════
// SECTION 4: Stream Options (limit, skip, select)
// ══════════════════════════════════════════════════════════
echo "\n⚙️ Stream Options\n";

stream_test('stream with limit', function() use ($tmpDir) {
    $path = "{$tmpDir}/large.json";
    $store = new JsonQ\Store($path);
    $items = $store->stream('/users', [], ['limit' => 10]);
    stream_assert_eq(10, count($items), 'limit 10');
});

stream_test('stream with skip', function() use ($tmpDir) {
    $path = "{$tmpDir}/large.json";
    $store = new JsonQ\Store($path);
    $items = $store->stream('/users', [], ['skip' => 990]);
    stream_assert_eq(10, count($items), 'skip 990 of 1000 → 10 remaining');
    stream_assert_eq(991, $items[0]['id'], 'first item after skip');
});

stream_test('stream with select (field projection)', function() use ($tmpDir) {
    $path = "{$tmpDir}/large.json";
    $store = new JsonQ\Store($path);
    $items = $store->stream('/users', [], ['select' => ['id', 'name']]);
    stream_assert_eq(1000, count($items));
    stream_assert(isset($items[0]['id']), 'id present');
    stream_assert(isset($items[0]['name']), 'name present');
    stream_assert(!isset($items[0]['age']), 'age excluded');
});


// ══════════════════════════════════════════════════════════
// SECTION 5: streamCount
// ══════════════════════════════════════════════════════════
echo "\n🔢 streamCount\n";

stream_test('streamCount returns total without filter', function() use ($tmpDir) {
    $path = "{$tmpDir}/large.json";
    $store = new JsonQ\Store($path);
    $count = $store->streamCount('/users');
    stream_assert_eq(1000, $count);
});

stream_test('streamCount with filter', function() use ($tmpDir) {
    $path = "{$tmpDir}/large.json";
    $store = new JsonQ\Store($path);
    $findCount  = count($store->find('users', ['role' => ['$eq' => 'admin']]));
    $streamCount = $store->streamCount('/users', ['role' => ['$eq' => 'admin']]);
    stream_assert_eq($findCount, $streamCount, 'streamCount must match find count');
});


// ══════════════════════════════════════════════════════════
// SECTION 6: streamToFile
// ══════════════════════════════════════════════════════════
echo "\n💾 streamToFile\n";

stream_test('streamToFile writes all items without filter', function() use ($tmpDir) {
    $path = "{$tmpDir}/large.json";
    $output = "{$tmpDir}/stream_out.json";
    $store = new JsonQ\Store($path);
    $written = $store->streamToFile('/users', $output);
    stream_assert_eq(1000, $written, 'should write 1000 items');
    stream_assert(file_exists($output), 'output file exists');
    $result = json_decode(file_get_contents($output), true);
    stream_assert(is_array($result), 'output is valid JSON array');
    stream_assert_eq(1000, count($result), 'output has 1000 items');
});


// ══════════════════════════════════════════════════════════
// SECTION 7: streamAggregate
// ══════════════════════════════════════════════════════════
echo "\n📊 streamAggregate\n";

stream_test('streamAggregate count without filter', function() use ($tmpDir) {
    $path = "{$tmpDir}/large.json";
    $store = new JsonQ\Store($path);
    $count = $store->streamAggregate('/users', 'count', 'id');
    stream_assert_eq(1000, $count);
});

stream_test('streamAggregate sum matches PHP calculation', function() use ($tmpDir) {
    $path = "{$tmpDir}/large.json";
    $store = new JsonQ\Store($path);
    $streamSum  = $store->streamAggregate('/users', 'sum', 'score');
    
    // Manual calculation
    $data = json_decode(file_get_contents($path), true);
    $manualSum = array_sum(array_column($data['users'], 'score'));
    
    stream_assert($manualSum == $streamSum, "stream sum ({$streamSum}) must match PHP array_sum ({$manualSum})");
});

stream_test('streamAggregate avg specific edge case (missing fields)', function() use ($tmpDir) {
    $path = "{$tmpDir}/avg_edge.json";
    $data = [
        ['id' => 1, 'score' => 10],
        ['id' => 2, 'name' => 'Bob'], // No score
        ['id' => 3, 'score' => 20],
        ['id' => 4, 'score' => 'invalid'], // Not a number
    ];
    file_put_contents($path, json_encode(['points' => $data]));
    
    $store = new JsonQ\Store($path);
    $avg = $store->streamAggregate('/points', 'avg', 'score');
    
    // Should be (10 + 20) / 2 = 15. The missing and invalid ones are skipped.
    stream_assert_eq(15, (int)$avg, 'average of valid scores');
});


// ══════════════════════════════════════════════════════════
// SUMMARY
// ══════════════════════════════════════════════════════════
echo "\n══════════════════════════════════════════\n";
echo "  Streaming Tests v0.8.0\n";
echo "  ✅ Passed: {$passCount}\n";
echo "  ❌ Failed: {$failCount}\n";
echo "══════════════════════════════════════════\n";

// Cleanup
array_map('unlink', glob("{$tmpDir}/*"));
rmdir($tmpDir);

exit($failCount > 0 ? 1 : 0);
