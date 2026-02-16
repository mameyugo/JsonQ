<?php
/**
 * Advanced Features Test Suite for JsonQ
 */

$passed = 0;
$failed = 0;
$errors = [];

function test(string $name, callable $fn): void {
    global $passed, $failed, $errors;
    try {
        $fn();
        $passed++;
        echo "  ✓ {$name}\n";
    } catch (\Throwable $e) {
        $failed++;
        $errors[] = "{$name}: {$e->getMessage()} (line {$e->getLine()})";
        echo "  ✗ {$name}: {$e->getMessage()}\n";
        if ($e instanceof \RuntimeException && str_contains($e->getMessage(), '^')) {
            echo "    [Error Context Detected]\n";
        }
    }
}

function assert_eq($expected, $actual, string $msg = ''): void {
    if ($expected !== $actual) {
        $e = var_export($expected, true);
        $a = var_export($actual, true);
        throw new \RuntimeException($msg ?: "Expected {$e}, got {$a}");
    }
}

function assert_true($val, string $msg = ''): void {
    if ($val !== true) throw new \RuntimeException($msg ?: "Expected true");
}

function assert_false($val, string $msg = ''): void {
    if ($val !== false) throw new \RuntimeException($msg ?: "Expected false");
}

function assert_count(int $expected, $arr, string $msg = ''): void {
    $c = is_array($arr) ? count($arr) : -1;
    if ($c !== $expected) throw new \RuntimeException($msg ?: "Expected count {$expected}, got {$c}");
}

function fresh_store(): \JsonQ\Store {
    $path = tempnam(sys_get_temp_dir(), 'jsonq_adv_') . '.json';
    return new \JsonQ\Store($path);
}

echo "\n🧪 JsonQ Advanced Features Test Suite\n";
echo str_repeat('═', 50) . "\n";

// ── 1. Advanced JSONPath ──
echo "\n🔍 Advanced JSONPath\n";

test('recursive descent (..)', function() {
    $s = fresh_store();
    $s->set('data', [
        'users' => [
            ['name' => 'Alice', 'meta' => ['id' => 1]],
            ['name' => 'Bob', 'meta' => ['id' => 2]]
        ],
        'meta' => ['version' => 1.0]
    ]);
    
    // Find all 'id' fields regardless of depth
    $results = $s->get('data..id');
    assert_count(2, $results);
    assert_eq(1, $results[0]);
    assert_eq(2, $results[1]);
});

test('wildcard operator (*)', function() {
    $s = fresh_store();
    $s->set('inventory', [
        'prod1' => ['price' => 10],
        'prod2' => ['price' => 20],
        'prod3' => ['price' => 30]
    ]);
    
    $prices = $s->get('inventory.*.price');
    assert_count(3, $prices);
    assert_true(in_array(10, $prices));
    assert_true(in_array(20, $prices));
    assert_true(in_array(30, $prices));
});

// ── 2. Regex Safety ──
echo "\n🛡️  Regex Safety\n";

test('regex backtrack limit protection', function() {
    $s = fresh_store();
    $s->set('text', 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa!');
    
    // This pattern is prone to ReDoS: (a+)+$
    $query = ['text' => ['$regex' => '^(a+)+$']];
    
    $start = microtime(true);
    $results = $s->find('', $query);
    $elapsed = microtime(true) - $start;
    
    assert_count(0, $results);
    assert_true($elapsed < 0.1, "Regex should fail quickly due to backtrack limit, took {$elapsed}s");
});

// ── 3. Visual Error Reporting ──
echo "\n⚠️  Error Reporting\n";

test('visual error marker in exception', function() {
    $s = fresh_store();
    try {
        $s->get('users[0:10:'); // Invalid slice
        throw new \Exception("Should have thrown a JsonQ Exception");
    } catch (\Throwable $e) {
        $msg = $e->getMessage();
        if (str_contains($msg, "Should have thrown")) throw $e;
        assert_true(str_contains($msg, 'users[0:10:'), "Error should contain path fragment");
        assert_true(str_contains($msg, '^'), "Error should contain visual marker ^");
    }
});

// ── 4. UTF-8 SIMD Validation ──
echo "\n🔡 UTF-8 Validation\n";

test('rejection of invalid UTF-8 files', function() {
    $path = tempnam(sys_get_temp_dir(), 'jsonq_utf8_') . '.json';
    // Invalid UTF-8 sequence
    file_put_contents($path, "{\"name\": \"\xf0\x9f\x90\"}"); // Missing last byte of emoji
    
    try {
        new \JsonQ\Store($path);
        unlink($path);
        throw new \Exception("Should have rejected invalid UTF-8");
    } catch (\Throwable $e) {
        $msg = $e->getMessage();
        if (str_contains($msg, "Should have rejected")) throw $e;
        assert_true(str_contains(strtolower($msg), 'utf-8') || str_contains(strtolower($msg), 'invalid'), "Error should mention UTF-8 or invalid content: $msg");
    } finally {
        if (file_exists($path)) unlink($path);
    }
});

// ═══════════════════════════════════════════
echo "\n" . str_repeat('═', 50) . "\n";
$total = $passed + $failed;
echo "Results: {$passed}/{$total} passed\n\n";

if ($failed > 0) {
    exit(1);
}
exit(0);
