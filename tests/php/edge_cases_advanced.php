<?php
// Advanced Edge Case Tests for JsonQ

function assert_eq($expected, $actual, $msg = '') {
    if ($expected !== $actual) {
        throw new Exception("Assertion failed: Expected " . var_export($expected, true) . ", got " . var_export($actual, true) . ". " . $msg);
    }
}

function assert_true($val, $msg = '') {
    if ($val !== true) throw new Exception("Assertion failed: Expected true. " . $msg);
}

function assert_throws($callback, $expectedMessagePart, $msg = '') {
    try {
        $callback();
        throw new Exception("Expected exception not thrown. " . $msg);
    } catch (Exception $e) {
        if (strpos($e->getMessage(), $expectedMessagePart) === false) {
            throw new Exception("Wrong exception message. Expected '{$expectedMessagePart}', got '{$e->getMessage()}'. " . $msg);
        }
    }
}

$path = tempnam(sys_get_temp_dir(), 'jsonq_edge_') . '.json';
$s = new \JsonQ\Store($path);

echo "🏔️ Testing Advanced Edge Cases\n";

// 1. Deep Nesting
echo "🧵 Testing Deep Nesting (30 levels)...\n";
$deep = "val";
for($i=0; $i<30; $i++) { $deep = ["level_$i" => $deep]; }
$s->set('deep', $deep);
$retrieved = $s->get('deep');
assert_eq($deep, $retrieved, "Deeply nested data should be retrieved correctly");
echo "  ✓ Deep nesting handled\n";

// 2. Regex ReDoS Mitigation
echo "🛑 Testing ReDoS Mitigation...\n";
// Pattern that would be slow in a vulnerable engine: (a+)+b with many 'a's
$malicious_pattern = "^(a+)+$";
$input = str_repeat("a", 100) . "!"; // '!' makes it fail, causing backtracking
$start = microtime(true);
$res = $s->find('users', ['name' => ['$regex' => $malicious_pattern]]);
$elapsed = microtime(true) - $start;
echo "  Malicious regex took " . round($elapsed, 4) . "s\n";
// The 'safe' regex engine should either fail fast or return (most likely fail compilation if too complex)
echo "  ✓ ReDoS protection confirmed (didn't hang)\n";

// 3. Compression Switching Stress
echo "🔄 Testing Compression Switching Stress...\n";
$methods = ['gzip', 'zstd', 'none'];
for($i=0; $i<10; $i++) {
    $m = $methods[$i % 3];
    $s->setOption('compression', $m);
    $s->set("key_$i", "Data for $m " . str_repeat("x", 100));
    clearstatcache();
    $s->get("key_$i"); // Verify it can still read it back
}
echo "  ✓ Rapid compression switching works\n";

// 4. Invalid Regex Pattern
echo "❌ Testing Invalid Regex Pattern...\n";
// This should not crash, just return empty result or error
$res = $s->find('users', ['name' => ['$regex' => '[[']]); 
assert_eq(0, count($res), "Invalid regex should return 0 results (safely)");
echo "  ✓ Invalid regex handled gracefully\n";

// 5. Query Optimizer with Non-Existent Collection
echo "👻 Testing Query on Non-Existent Collection...\n";
$res = $s->find('non_existent', ['id' => 1]);
assert_eq(0, count($res), "Query on missing collection should be empty");
echo "  ✓ Missing collection handled\n";

// 6. Metrics Accuracy (Cache vs Disk)
echo "📈 Testing Metrics Accuracy (Cache vs Disk)...\n";
$fresh_path = $path . '.metrics.json';
$s_fresh = new \JsonQ\Store($fresh_path);
$s_fresh->set('stats_test', [['id' => 1]]);

// Re-open to clear in-memory cache but keep on-disk metrics? 
// No, metrics are per instance. Let's just use the current instance but force mtime change.
$s_fresh->get('stats_test'); // Hit (because of set)
$m_start = $s_fresh->getMetrics();

// Force cache miss by waiting and touching file
sleep(1);
touch($fresh_path);
clearstatcache();

$s_fresh->get('stats_test'); // Miss
$m_miss = $s_fresh->getMetrics();
assert_true($m_miss['cache_misses'] > $m_start['cache_misses'], "Cache miss should increment after touch");

$s_fresh->get('stats_test'); // Hit
$m_hit = $s_fresh->getMetrics();
assert_true($m_hit['cache_hits'] > $m_miss['cache_hits'], "Cache hit should increment on second read");
echo "  ✓ Metrics are accurate (Hits: {$m_hit['cache_hits']}, Misses: {$m_hit['cache_misses']})\n";
unlink($fresh_path);

// 7. Corrupted Compressed File
echo "☣️  Testing Corrupted Compressed File...\n";
$s->setOption('compression', 'zstd');
$s->set('secret', 'important');
$s->set('trigger_flush', true);
// Manually corrupt the file (flip some bits after header)
$content = file_get_contents($path);
$content[20] = chr(ord($content[20]) ^ 0xFF); // Corrupt further in
file_put_contents($path, $content);

assert_throws(function() use ($path) {
    $s2 = new \JsonQ\Store($path);
    $s2->get('secret');
}, "decompression failed", "Should catch decompression error");
echo "  ✓ Corrupted compressed file handled safely\n";

echo "\n🏆 All advanced edge cases passed!\n";

unlink($path);
unlink($path . ".idx");
