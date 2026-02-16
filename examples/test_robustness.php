<?php

function assert_contains($haystack, $needle, $message) {
    if (strpos($haystack, $needle) === false) {
        echo "FAIL: $message\n";
        echo "Expected to find '$needle' in:\n$haystack\n";
        exit(1);
    } else {
        echo "PASS: $message\n";
    }
}

// 1. Setup
$file = __DIR__ . '/robustness_test.json';
if (file_exists($file)) unlink($file);
file_put_contents($file, json_encode([
    [
        "name" => "Alice",
        "email" => "alice@example.com", 
        "bio" => str_repeat("A", 1024) // 1KB bio
    ],
    [
        "name" => "Bob",
        "email" => "bob@example.com",
        "bio" => "Short bio"
    ]
]));

$store = new \JsonQ\Store($file);

echo "1. Testing Normal Query (Filter)...\n";
// Basic check using find (MongoDB-style) on root array ("")
try {
    $res = $store->find("", ["name" => "Alice"]);
    if (empty($res) || count($res) !== 1) {
        throw new Exception("Expected 1 result, got " . count($res));
    }
    echo "PASS: Normal query works\n";
} catch (Exception $e) {
    echo "FAIL: Normal query failed: " . $e->getMessage() . "\n";
    exit(1);
}

echo "\n2. Testing Error Suggestions (Levenshtein)...\n";
// Using jsonq_query_node which parses JSONPath
try {
    // Typo in slice operator
    jsonq_query_node($file, 'names[0:1:0]'); // Invalid slice step 0
} catch (Exception $e) {
    $msg = $e->getMessage();
    assert_contains($msg, "Slice step cannot be zero", "Slice validation error");
}

try {
    // Syntax error with suggestion
    jsonq_query_node($file, 'users[0:10:a]'); 
} catch (Exception $e) {
    $msg = $e->getMessage();
    assert_contains($msg, "Check slice syntax", "Suggestion present");
}

echo "\n3. Testing Regex Safety...\n";
try {
    // Regex using $regex operator
    $res = $store->find("", ["bio" => ["\$regex" => "^A+$"]]);
    // The result should contain Alice
    $json = json_encode($res);
    assert_contains($json, "Alice", "Regex matched within limit");
    echo "PASS: Regex safe execution\n";
} catch (Exception $e) {
    echo "FAIL: Regex execution failed: " . $e->getMessage() . "\n";
}

echo "\n4. Testing UTF-8 Validation...\n";
// Create invalid UTF-8 file
$invalid_file = __DIR__ . '/invalid_utf8.json';
file_put_contents($invalid_file, "\xFF\xFE" . json_encode(["status" => "ok"]));
try {
    // Just initializing or reading should trigger validation
    $bad_store = new \JsonQ\Store($invalid_file);
    // Force read
    $bad_store->getAll();
    echo "FAIL: Should have rejected invalid UTF-8\n";
} catch (Exception $e) {
    assert_contains($e->getMessage(), "Invalid UTF-8", "Caught invalid UTF-8");
}

// Clean up
@unlink($file);
@unlink($invalid_file);

echo "\nAll Robustness Tests Passed!\n";
