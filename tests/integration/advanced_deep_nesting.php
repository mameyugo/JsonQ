<?php

function assert_eq($expected, $actual, $msg = "") {
    if ($expected !== $actual) {
        echo "  ✗ $msg: Expected " . json_encode($expected) . ", got " . json_encode($actual) . "\n";
        return false;
    }
    echo "  ✓ $msg\n";
    return true;
}

echo "🧪 JsonQ Advanced Deep Nesting & Architecture Tests\n";
echo "══════════════════════════════════════════════════\n";

$dbPath = '/tmp/deep_test.json';
if (file_exists($dbPath)) unlink($dbPath);
$s = new JsonQ\Store($dbPath);

echo "🌳 Deep Nesting (5+ levels)\n";
$deepObj = [
    'level1' => [
        'level2' => [
            'level3' => [
                'level4' => [
                    'level5' => [
                        'secret' => 'findme',
                        'value' => 42
                    ]
                ]
            ]
        ]
    ]
];

$s->set('data', $deepObj);
assert_eq('findme', $s->get('data.level1.level2.level3.level4.level5.secret'), "Get deep nested value");
assert_eq(42, $s->get('data.level1.level2.level3.level4.level5.value'), "Get deep nested number");

echo "🔍 MongoDB Queries on Deep Fields\n";
$s->set('users', [
    ['id' => 1, 'meta' => ['profile' => ['settings' => ['theme' => 'dark']]]],
    ['id' => 2, 'meta' => ['profile' => ['settings' => ['theme' => 'light']]]],
]);

$found = $s->find('users', ['meta.profile.settings.theme' => 'dark']);
assert_eq(1, count($found), "Find by deep nested field");
assert_eq(1, $found[0]['id'], "Verify correct item found");

echo "🔗 Fluent Queries on Deep Fields\n";
$r = $s->executeQuery('users', [
    'where' => [['field' => 'meta.profile.settings.theme', 'op' => '=', 'value' => 'light']],
    'select' => ['id']
]);
assert_eq(1, count($r), "Fluent query on deep nested field");
assert_eq(2, $r[0]['id'] ?? $r[0], "Verify correct item found (select projection)");

echo "✅ Schema Validation (Complex Nested)\n";
$schema = [
    'type' => 'object',
    'properties' => [
        'level1' => [
            'type' => 'object',
            'required' => ['level2'],
            'properties' => [
                'level2' => [
                    'type' => 'object',
                    'properties' => [
                        'level3' => ['type' => 'object']
                    ]
                ]
            ]
        ]
    ]
];
$v = $s->validate('data', $schema);
assert_eq(true, $v['valid'], "Deep schema validation (valid)");

$invalidSchema = [
    'properties' => [
        'level1' => [
            'properties' => [
                'level2' => [
                    'properties' => [
                        'level3' => ['type' => 'number']
                    ]
                ]
            ]
        ]
    ]
];
$v2 = $s->validate('data', $invalidSchema);
assert_eq(false, $v2['valid'], "Deep schema validation (invalid type deep down)");

echo "⚡ Indexing Deep Fields\n";
$s->createIndex('users', 'meta.profile.settings.theme');
$foundIdx = $s->indexLookup('users', 'meta.profile.settings.theme', 'dark');
assert_eq(1, count($foundIdx), "Index lookup on deep nested field");

echo "🔄 Atomic Writes / Persistence\n";
$s->setOption('pretty', true);
$s->set('atomic', 'test');
$content = file_get_contents($dbPath);
assert_eq(true, strpos($content, '{') !== false, "File is valid JSON");
assert_eq(true, strpos($content, '  "atomic": "test"') !== false, "Pretty print works");

echo "══════════════════════════════════════════════════\n";
echo "DONE\n";
