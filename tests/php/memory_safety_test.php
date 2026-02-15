<?php

/**
 * Memory safety tests for string conversion
 */

// Test 1: String survives scope changes
function test_string_memory_safety() {
    $store = new JsonQ\Store('/tmp/test_memory.json');
    
    // Create string in limited scope
    {
        $temp_data = "This is a temporary string that goes out of scope";
        $store->set('test', $temp_data);
        unset($temp_data);
        // $temp_data is gone, garbage collected
    }
    
    // String should still be retrievable
    $retrieved = $store->get('test');
    assert($retrieved === "This is a temporary string that goes out of scope");
    
    echo "✓ String memory safety test passed\n";
}

// Test 2: Large string handling
function test_large_string() {
    $store = new JsonQ\Store('/tmp/test_large.json');
    
    $large_string = str_repeat("x", 1024 * 100); // 100KB string
    $store->set('large', $large_string);
    unset($large_string);
    
    $retrieved = $store->get('large');
    assert(strlen($retrieved) === 1024 * 100);
    
    echo "✓ Large string test passed\n";
}

// Test 3: Unicode safety
function test_unicode_strings() {
    $store = new JsonQ\Store('/tmp/test_unicode.json');
    
    $unicode = "Hello 世界 🌍 مرحبا мир";
    $store->set('unicode', $unicode);
    
    $retrieved = $store->get('unicode');
    assert($retrieved === $unicode);
    
    echo "✓ Unicode string test passed\n";
}

test_string_memory_safety();
test_large_string();
test_unicode_strings();

echo "\n✅ All memory safety tests passed\n";
