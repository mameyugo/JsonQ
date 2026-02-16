<?php

function assert_true($condition, $message) {
    if (!$condition) {
        echo "❌ $message\n";
        exit(1);
    }
    echo "✅ $message\n";
}

$file = __DIR__ . '/path_test.json';
if (file_exists($file)) unlink($file);

$data = [
    'users' => [
        ['name' => 'Alice', 'role' => 'admin', 'age' => 30],
        ['name' => 'Bob', 'role' => 'user', 'age' => 25],
        ['name' => 'Charlie', 'role' => 'user', 'age' => 20],
        ['name' => 'Dave', 'role' => 'guest', 'age' => 18],
        ['name' => 'Eve', 'role' => 'admin', 'age' => 35]
    ],
    'meta' => [
        'version' => '1.0',
        'region' => 'US'
    ]
];
file_put_contents($file, json_encode($data));

echo "--- Testing JSONPath Slices ---\n";
// Slice: first 2 users
$result = jsonq_query_node($file, "users[0:2]");
assert_true(is_array($result), "Result is array");
assert_true(count($result) === 2, "Got 2 users");
$u1 = json_decode($result[0], true);
assert_true($u1['name'] === 'Alice', "First is Alice");

// Slice: Step 2
$result = jsonq_query_node($file, "users[0:5:2]");
// 0 (Alice), 2 (Charlie), 4 (Eve)
assert_true(count($result) === 3, "Got 3 users with step 2");
$u3 = json_decode($result[2], true);
assert_true($u3['name'] === 'Eve', "Third is Eve");

echo "--- Testing Multi-Key ---\n";
// Multi-key on first user
// path: users[0]["name","role"]
// My parser supports users[0] then ["name","role"]?
// No, current parser splits by . or [.
// users[0] -> Key(users), Index(0).
// Then ["name","role"] -> MultiKey.
// So query: "users[0][\"name\",\"role\"]"
$result = jsonq_query_node($file, "users[0]['name','role']");
assert_true(count($result) === 1, "Got 1 result");
$obj = json_decode($result[0], true);
assert_true(isset($obj['name']) && isset($obj['role']), "Has name and role");
assert_true(!isset($obj['age']), "Does not have age");

echo "--- Testing Find with Regex ---\n";
// Create store to use find
$store = new JsonQ\Store($file);
$regex_cond = [
    "name" => ['$regex' => "^A"]
];
$found = $store->find("users", $regex_cond);
assert_true(count($found) === 1, "Found 1 starting with A");
assert_true($found[0]['name'] === 'Alice', "It is Alice");

$regex_cond2 = [
    "name" => ['$regex' => "e$"]
];
$found2 = $store->find("users", $regex_cond2);
// Alice(e), Charlie(e), Dave(e), Eve(e). 4 matches.
assert_true(count($found2) === 4, "Found 4 ending with e");

echo "JSONPath and Regex Test Passed!\n";
