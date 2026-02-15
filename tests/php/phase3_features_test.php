<?php
// Phase 3 Features Verification Test

function assert_eq($expected, $actual, $msg = '') {
    if ($expected !== $actual) {
        throw new Exception("Assertion failed: Expected " . var_export($expected, true) . ", got " . var_export($actual, true) . ". " . $msg);
    }
}

function assert_true($val, $msg = '') {
    if ($val !== true) throw new Exception("Assertion failed: Expected true. " . $msg);
}

$path = tempnam(sys_get_temp_dir(), 'jsonq_p3_') . '.json';
$s = new \JsonQ\Store($path);

echo "🧪 Testing Phase 3 Features\n";

// 1. Safe Regex
echo "🔍 Testing Safe Regex...\n";
$s->set('users', [
    ['name' => 'Alice', 'email' => 'alice@gmail.com'],
    ['name' => 'Bob', 'email' => 'bob@company.co.uk'],
    ['name' => 'Charlie', 'email' => 'charlie@outlook.es'],
]);

// Test complex regex pattern
$gmail_users = $s->find('users', ['email' => ['$regex' => '@gmail\.com$']]);
assert_eq(1, count($gmail_users), "Should find 1 gmail user");
assert_eq('Alice', $gmail_users[0]['name']);

$company_users = $s->find('users', ['email' => ['$regex' => '@company\.(com|co\.uk)$']]);
assert_eq(1, count($company_users), "Should find 1 company user");
assert_eq('Bob', $company_users[0]['name']);

echo "  ✓ Regex works with complex patterns\n";

// 2. Metrics
echo "📊 Testing Metrics...\n";
$metrics = $s->getMetrics();
echo "  Metrics Debug: " . json_encode($metrics) . "\n";
assert_true(isset($metrics['reads']), "Metrics should have reads");
assert_true($metrics['reads'] > 0, "Reads should be > 0");
assert_true($metrics['writes'] > 0, "Writes should be > 0");
echo "  ✓ Metrics captured (Reads: {$metrics['reads']}, Writes: {$metrics['writes']})\n";

// 3. Compression
echo "📦 Testing Storage Compression...\n";
$s->setOption('compression', 'zstd');
$s->set('large_data', array_fill(0, 1000, "This is a repetitive string to test compression ratio."));
$s->set('trigger_flush', true);
clearstatcache();
$compressed_size = filesize($path);
$header = bin2hex(file_get_contents($path, false, null, 0, 4));
echo "  Compressed size (Zstd): $compressed_size bytes (Header: $header)\n";

$s->setOption('compression', 'none');
$s->set('trigger_flush_2', true);
clearstatcache();
$uncompressed_size = filesize($path);
$header_none = bin2hex(file_get_contents($path, false, null, 0, 4));
echo "  Uncompressed size: $uncompressed_size bytes (Header: $header_none)\n";

assert_true($compressed_size < $uncompressed_size, "Compressed size ($compressed_size) should be smaller than uncompressed ($uncompressed_size)");
echo "  ✓ Compression works and reduces file size\n";

// 4. Query Optimizer (Correctness)
echo "⚡ Testing Query Optimizer Correctness...\n";
$s->createIndex('users', 'name');
$optimized_result = $s->find('users', ['name' => 'Alice', 'email' => 'alice@gmail.com']);
assert_eq(1, count($optimized_result), "Optimized find should return correct result");
assert_eq('Alice', $optimized_result[0]['name']);
echo "  ✓ Query optimizer returns correct results\n";

echo "\n✅ All Phase 3 features verified successfully!\n";

unlink($path);
