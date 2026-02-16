<?php

function assert_true($condition, $message) {
    if (!$condition) {
        echo "❌ $message\n";
        exit(1);
    }
    echo "✅ $message\n";
}

$largeFile = __DIR__ . '/large_data.json';
$outputFile = __DIR__ . '/output_stream.json';
$jsonlFile = __DIR__ . '/data.jsonl';

// Clean up
if (file_exists($largeFile)) unlink($largeFile);
if (file_exists($outputFile)) unlink($outputFile);
if (file_exists($jsonlFile)) unlink($jsonlFile);

// Allow .jsonl extension
jsonq_set_allowed_extensions("json,jsonl");

// Create a moderately large file
$data = [];
for ($i = 0; $i < 1000; $i++) {
    $data[] = ['id' => $i, 'name' => "User $i", 'score' => rand(1, 100)];
}
file_put_contents($largeFile, json_encode($data));

echo "--- Testing Stream I/O ---\n";

// Test write_to_file
$start = microtime(true);
$result = jsonq_write_to_file($largeFile, $outputFile, false);
$end = microtime(true);
assert_true($result === true, "write_to_file returned true");
assert_true(file_exists($outputFile), "Output file created");
assert_true(filesize($outputFile) > 0, "Output file has content");

$content = file_get_contents($outputFile);
$decoded = json_decode($content, true);
assert_true(count($decoded) === 1000, "Output content matches input count");
echo "Stream write took " . number_format(($end - $start) * 1000, 2) . " ms\n";

echo "\n--- Testing JSONL ---\n";

// Test append_jsonl
$record1 = json_encode(['event' => 'login', 'user' => 'alice', 'ts' => time()]);
$record2 = json_encode(['event' => 'logout', 'user' => 'bob', 'ts' => time() + 100]);

jsonq_append_jsonl($jsonlFile, $record1);
jsonq_append_jsonl($jsonlFile, $record2);

assert_true(file_exists($jsonlFile), "JSONL file created");
$lines = file($jsonlFile);
assert_true(count($lines) === 2, "JSONL has 2 lines");

// Test read_jsonl
$records = jsonq_read_jsonl($jsonlFile);
assert_true(is_array($records), "read_jsonl returned array");
if (count($records) !== 2) {
    echo "Count was " . count($records) . "\n";
    print_r($records);
}
assert_true(count($records) === 2, "read_jsonl returned 2 records");

$r1 = json_decode($records[0], true);
assert_true($r1['user'] === 'alice', "First record matches");

echo "\nAll Stream I/O tests passed!\n";
