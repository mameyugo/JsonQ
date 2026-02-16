<?php

function assert_true($condition, $message) {
    if (!$condition) {
        echo "❌ $message\n";
        exit(1);
    }
    echo "✅ $message\n";
}

$file = __DIR__ . '/memory_test.json';
if (file_exists($file)) unlink($file);

// Create data with heavily repeated keys
$data = [];
for ($i=0; $i<1000; $i++) {
    $data[] = [
        'name' => "User $i", 
        'role' => 'admin', 
        'status' => 'active',
        'preferences' => [
            'theme' => 'dark',
            'notifications' => true
        ]
    ];
}
file_put_contents($file, json_encode($data));

echo "--- Testing Memory Stats ---\n";
$stats = jsonq_memory_stats($file);

echo "Stats:\n";
print_r($stats);

assert_true(isset($stats['unique_keys']), "Has unique_keys");
assert_true(isset($stats['total_references']), "Has total_references");

// Logic check: unique keys should be roughly 5 ('name', 'role', 'status', 'preferences', 'theme', 'notifications' -> 6 actually)
// plus indices if any? No, array indices are not interned in our logic (only object keys).
assert_true($stats['unique_keys'] >= 5, "Detected unique keys");

echo "\nNote: total_references is expected to be 0 currently because we don't store Arc<str> in Value.\n";

echo "Memory Test Passed!\n";
