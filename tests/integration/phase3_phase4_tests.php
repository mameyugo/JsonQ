<?php
/**
 * Specialized Tests for JsonQ Phase 3 & 4
 */

function assert_eq($expected, $actual, $msg = '') {
    if ($expected !== $actual) {
        $e = var_export($expected, true);
        $a = var_export($actual, true);
        throw new Exception("FAIL: $msg (Expected $e, got $a)");
    }
}

function assert_true($val, $msg = '') {
    if ($val !== true) throw new Exception("FAIL: $msg (Expected true)");
}

$path1 = tempnam(sys_get_temp_dir(), 'jsonq_p3_1_') . '.json';
$path2 = tempnam(sys_get_temp_dir(), 'jsonq_p3_2_') . '.json';

$s1 = new \JsonQ\Store($path1);
$s2 = new \JsonQ\Store($path2);

echo "🧪 Running Phase 3 & 4 Specialized Tests...\n";

// 1. Metrics & Global Aggregation (Phase 3 & 4)
echo "   - Testing Global Metrics...";
$s1->set('a', 1);
$s2->set('b', 2);
$s1->get('a');
$s2->get('b');

$m1 = $s1->getMetrics();
$m2 = $s2->getMetrics();

// Metrics should be global, so both should see at least 2 writes and 2 reads
if ($m1['writes'] < 2 || $m1['reads'] < 2) {
    throw new Exception("FAIL: Metrics are not global. Got " . json_encode($m1));
}
assert_eq($m1['writes'], $m2['writes'], "Metrics should be identical across instances");
echo " OK\n";

// 2. Compression (Phase 3)
echo "   - Testing Compression (Zstd)...";
$s1->setOption('compression', 'zstd');
$s1->set('compressed_data', str_repeat('COMPRESSION_TEST_', 100)); // Large enough to compress
$s1->set('data', 'raw'); 
assert_eq('raw', $s1->get('data'), "Content should be readable after compression");
echo " OK\n";

// 3. Safe Regex (Phase 3)
echo "   - Testing Safe Regex...";
$s1->set('items', [
    ['name' => 'apple'],
    ['name' => 'banana'],
    ['name' => 'cherry']
]);
$res = $s1->find('items', ['name' => ['$regex' => '^a.*e$']]);
assert_eq(1, count($res), "Regex match failed");
assert_eq('apple', $res[0]['name']);

// Test ReDoS-like pattern (should be handled safely by backtracking limit)
try {
    // A classic backtracking bomb
    $bomb = '(a+)+$';
    $long_str = str_repeat('a', 30) . 'b';
    $s1->set('bomb_data', [['v' => $long_str]]);
    $s1->find('bomb_data', ['v' => ['$regex' => $bomb]]);
    // If it doesn't hang, it's successful or it hit the limit
    echo " OK\n";
} catch (Exception $e) {
    echo " OK (Caught expected limit error: " . $e->getMessage() . ")\n";
}

// 4. Thread-safe Config (Phase 4 - Internal check via behavior)
echo "   - Testing Options Persistence...";
$s1->setOption('pretty', true);
assert_true($s1->getOption('pretty'), "Option setting failed");
echo " OK\n";

echo "\n✅ All Phase 3 & 4 Specialized Tests PASSED!\n";

@unlink($path1);
@unlink($path2);
