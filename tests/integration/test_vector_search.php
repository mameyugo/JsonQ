<?php
/**
 * Vector Search Integration Tests
 */

require_once __DIR__ . '/helpers.php';

echo "\n🔮 Native Vector Search Tests\n";
echo str_repeat('═', 50) . "\n";

function setup_vector_data() {
    $s = fresh_store();
    $s->set('items', [
        ['id' => 1, 'name' => 'Vector A', 'embedding' => [1.0, 0.0, 0.0]],
        ['id' => 2, 'name' => 'Vector B', 'embedding' => [0.0, 1.0, 0.0]],
        ['id' => 3, 'name' => 'Vector C', 'embedding' => [0.707, 0.707, 0.0]], // Cosine similarity with [1, 0, 0] is 0.707
        ['id' => 4, 'name' => 'Vector D', 'embedding' => [-1.0, 0.0, 0.0]], // Opposite
    ]);
    return $s;
}

// ── Fallback (Unindexed) Search ──
echo "\n🔍 Fallback (Unindexed) Vector Search\n";

test('Unindexed vector search - Cosine similarity', function() {
    $s = setup_vector_data();
    
    // Query vector matches Vector A perfectly
    $results = $s->vectorSearch('items', 'embedding', [1.0, 0.0, 0.0], 2, 'cosine');
    
    assert_count(2, $results);
    assert_eq(1, $results[0]['item']['id']);
    assert_eq('Vector A', $results[0]['item']['name']);
    // Cosine similarity of A with query is 1.0
    assert_true(abs($results[0]['score'] - 1.0) < 0.001);
    
    // Vector C is second closest (cosine ~0.707)
    assert_eq(3, $results[1]['item']['id']);
    assert_true(abs($results[1]['score'] - 0.707) < 0.005);
});

test('Unindexed vector search - L2 distance', function() {
    $s = setup_vector_data();
    
    // Query vector matches Vector A perfectly
    $results = $s->vectorSearch('items', 'embedding', [1.0, 0.0, 0.0], 2, 'l2');
    
    assert_count(2, $results);
    assert_eq(1, $results[0]['item']['id']);
    // Distance should be 0.0
    assert_true(abs($results[0]['score'] - 0.0) < 0.001);
    
    // Next closest is Vector C (distance = sqrt((1-0.707)^2 + (0.707)^2) = sqrt(0.0858 + 0.5) ~0.765)
    assert_eq(3, $results[1]['item']['id']);
    assert_true(abs($results[1]['score'] - 0.765) < 0.01);
});

test('Unindexed vector search - Dot Product', function() {
    $s = setup_vector_data();
    
    // Query vector matches Vector A perfectly
    $results = $s->vectorSearch('items', 'embedding', [2.0, 0.0, 0.0], 2, 'dot');
    
    assert_count(2, $results);
    assert_eq(1, $results[0]['item']['id']);
    // Dot product: 1.0 * 2.0 = 2.0
    assert_true(abs($results[0]['score'] - 2.0) < 0.001);
});

// ── Vector Indexing ──
echo "\n⚡ Indexed Vector Search\n";

test('Create vector index and search - Cosine', function() {
    $s = setup_vector_data();
    
    $created = $s->createVectorIndex('items', 'embedding', [
        'dimension' => 3,
        'metric' => 'cosine'
    ]);
    assert_true($created, 'Should successfully create vector index');
    
    // Confirm index is listed
    $indexes = $s->listIndexes();
    $vectorIdx = null;
    foreach ($indexes as $idx) {
        if ($idx['type'] === 'vector' && $idx['field'] === 'embedding') {
            $vectorIdx = $idx;
            break;
        }
    }
    assert_true($vectorIdx !== null, 'Vector index should be listed');
    assert_eq(3, $vectorIdx['dimension']);
    assert_eq('cosine', $vectorIdx['metric']);
    assert_eq(4, $vectorIdx['total_entries']);
    
    // Query search with index
    $results = $s->vectorSearch('items', 'embedding', [1.0, 0.0, 0.0], 2);
    assert_count(2, $results);
    assert_eq(1, $results[0]['item']['id']);
    assert_eq(3, $results[1]['item']['id']);
    assert_true(abs($results[0]['score'] - 1.0) < 0.001);
});

test('Vector index persistence and reload', function() {
    $path = tempnam(sys_get_temp_dir(), 'jsonq_vector_persist_') . '.json';
    
    // Setup and save index
    {
        $s = new \JsonQ\Store($path);
        $s->set('items', [
            ['id' => 1, 'name' => 'Vector A', 'embedding' => [1.0, 0.0, 0.0]],
            ['id' => 2, 'name' => 'Vector B', 'embedding' => [0.0, 1.0, 0.0]],
        ]);
        
        $created = $s->createVectorIndex('items', 'embedding', [
            'dimension' => 3,
            'metric' => 'l2'
        ]);
        assert_true($created);
    }
    
    // Verify file exists
    $vidxPath = str_replace('.json', '.items.' . md5('embedding') . '.vidx', $path);
    assert_true(file_exists($vidxPath), 'Vector index file should exist on disk');
    
    // Load store again and verify it loads index
    {
        $s2 = new \JsonQ\Store($path);
        
        // Search should work (this will trigger lazy loading)
        $results = $s2->vectorSearch('items', 'embedding', [0.0, 1.0, 0.0], 1);
        assert_count(1, $results);
        assert_eq(2, $results[0]['item']['id']);
        assert_true(abs($results[0]['score'] - 0.0) < 0.001);

        // Now listIndexes should find the loaded index
        $indexes = $s2->listIndexes();
        $vectorIdx = null;
        foreach ($indexes as $idx) {
            if ($idx['type'] === 'vector' && $idx['field'] === 'embedding') {
                $vectorIdx = $idx;
                break;
            }
        }
        assert_true($vectorIdx !== null, 'Vector index should be automatically reloaded on demand');
        assert_eq(3, $vectorIdx['dimension']);
        assert_eq('l2', $vectorIdx['metric']);
    }
    
    // Clean up
    if (file_exists($path)) {
        unlink($path);
    }
    if (file_exists($vidxPath)) {
        unlink($vidxPath);
    }
});

// ── Error Handling ──
echo "\n⚠️ Error Handling and Validations\n";

test('Create vector index with dimension mismatch returns false', function() {
    $s = setup_vector_data();
    
    // Items have dimension 3, index requires 4
    $created = $s->createVectorIndex('items', 'embedding', [
        'dimension' => 4,
        'metric' => 'cosine'
    ]);
    assert_false($created, 'Should return false due to dimension mismatch');
});

test('Search with query vector dimension mismatch throws', function() {
    $s = setup_vector_data();
    $s->createVectorIndex('items', 'embedding', [
        'dimension' => 3
    ]);
    
    try {
        // Query has 2 elements instead of 3
        $s->vectorSearch('items', 'embedding', [1.0, 0.0], 2);
        assert_true(false, 'Should have thrown exception due to dimension mismatch on search');
    } catch (\Exception $e) {
        assert_true(str_contains($e->getMessage(), 'dimension') || str_contains($e->getMessage(), 'length'), 'Exception mentions dimension: ' . $e->getMessage());
    }
});

test('Search with non-array or non-numeric query vector throws', function() {
    $s = setup_vector_data();
    
    try {
        $s->vectorSearch('items', 'embedding', 'not-an-array', 2);
        assert_true(false, 'Should have failed with string query');
    } catch (\Exception $e) {
        assert_true(str_contains($e->getMessage(), 'array'), 'Exception mentions array: ' . $e->getMessage());
    }
    
    try {
        $s->vectorSearch('items', 'embedding', [1.0, 'two', 3.0], 2);
        assert_true(false, 'Should have failed with non-numeric string element');
    } catch (\Exception $e) {
        assert_true(str_contains($e->getMessage(), 'number') || str_contains($e->getMessage(), 'numeric'), 'Exception mentions numbers: ' . $e->getMessage());
    }
});

print_summary();
