<?php

function assert_contains($haystack, $needle, $message) {
    if (strpos($haystack, $needle) === false) {
        echo "❌ $message\nExpected to contain '$needle', got:\n$haystack\n";
        exit(1);
    }
    echo "✅ $message\n";
}

$file = __DIR__ . '/error_test.json';
if (!file_exists($file)) file_put_contents($file, "{}");

echo "--- Testing Error Reporting ---\n";
try {
    jsonq_query_node($file, "users[0:10:0]"); // Invalid step 0
    echo "❌ Should have failed with step 0\n";
    exit(1);
} catch (Exception $e) {
    echo "Caught expected exception:\n" . $e->getMessage() . "\n";
    assert_contains($e->getMessage(), "step cannot be zero", "Error message correct");
    // Position might be 0 because parse_slice sets it to 0 initially in new logic, or inherited?
    // In my impl, parse_slice returns error with pos 0.
    // parse_json_path wraps it with `bracket_start`?
    // Let's see: `QueryError::new(e.message, bracket_start)`
    // So position should be pointing to '['.
}

try {
    jsonq_query_node($file, "users["); // Unclosed
    echo "❌ Should have failed with unclosed bracket\n";
    exit(1);
} catch (Exception $e) {
    echo "Caught expected exception:\n" . $e->getMessage() . "\n";
    assert_contains($e->getMessage(), "Unclosed bracket", "Error message correct");
    assert_contains($e->getMessage(), "Suggestion: Add ']'", "Suggestion present");
}

echo "Error Reporting Test Passed!\n";
