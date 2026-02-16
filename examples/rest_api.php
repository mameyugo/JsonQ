<?php
/**
 * JsonQ REST API Example
 *
 * Demonstrates using JsonQ as a backend for a simple REST API.
 * Run: php -d "extension=path/to/libjsonq.so" -S localhost:8080 examples/rest_api.php
 *
 * Endpoints:
 *   GET    /users              — List users (supports ?role=admin&sort=age&limit=10)
 *   GET    /users/{id}         — Get user by ID
 *   POST   /users              — Create user
 *   PUT    /users/{id}         — Update user
 *   DELETE /users/{id}         — Delete user
 *   GET    /users/stats        — Aggregation stats
 */

use JsonQ\Store;

// ── Setup ──
$store = new Store(__DIR__ . '/../storage/api_data.json');

// Seed initial data if empty
if ($store->count('users') <= 0) {
    $store->set('users', [
        ['id' => 1, 'name' => 'Alice',   'email' => 'alice@example.com',   'role' => 'admin', 'age' => 30],
        ['id' => 2, 'name' => 'Bob',     'email' => 'bob@example.com',     'role' => 'user',  'age' => 25],
        ['id' => 3, 'name' => 'Charlie', 'email' => 'charlie@example.com', 'role' => 'admin', 'age' => 35],
    ]);
    $store->set('meta.next_id', 4);
    $store->createIndex('users', 'id');
    $store->createIndex('users', 'email');
}

// ── Router ──
$method = $_SERVER['REQUEST_METHOD'];
$path   = parse_url($_SERVER['REQUEST_URI'], PHP_URL_PATH);
$query  = [];
parse_str(parse_url($_SERVER['REQUEST_URI'], PHP_URL_QUERY) ?? '', $query);

header('Content-Type: application/json');

function json_response(mixed $data, int $code = 200): void {
    http_response_code($code);
    echo json_encode($data, JSON_PRETTY_PRINT | JSON_UNESCAPED_UNICODE);
    exit;
}

function get_body(): array {
    return json_decode(file_get_contents('php://input'), true) ?? [];
}

// ── Routes ──

// GET /users/stats
if ($method === 'GET' && $path === '/users/stats') {
    json_response([
        'total'    => $store->count('users'),
        'avg_age'  => $store->aggregate('users', 'age', 'avg'),
        'by_role'  => array_map('count', (array) $store->groupBy('users', 'role')),
        'ages'     => [
            'min' => $store->aggregate('users', 'age', 'min'),
            'max' => $store->aggregate('users', 'age', 'max'),
        ],
    ]);
}

// GET /users
if ($method === 'GET' && $path === '/users') {
    $spec = [];

    // Build where conditions from query params
    $wheres = [];
    foreach (['role', 'name', 'email'] as $field) {
        if (isset($query[$field])) {
            $wheres[] = ['field' => $field, 'op' => '=', 'value' => $query[$field]];
        }
    }
    if (isset($query['min_age'])) {
        $wheres[] = ['field' => 'age', 'op' => '>=', 'value' => (int) $query['min_age']];
    }
    if (isset($query['max_age'])) {
        $wheres[] = ['field' => 'age', 'op' => '<=', 'value' => (int) $query['max_age']];
    }
    if (!empty($wheres)) $spec['where'] = $wheres;

    // Sorting
    if (isset($query['sort'])) {
        $dir = ($query['order'] ?? 'asc') === 'desc' ? 'desc' : 'asc';
        $spec['order_by'] = ['field' => $query['sort'], 'direction' => $dir];
    }

    // Pagination
    if (isset($query['limit']))  $spec['limit']  = (int) $query['limit'];
    if (isset($query['offset'])) $spec['offset'] = (int) $query['offset'];

    // Field selection
    if (isset($query['fields'])) $spec['select'] = explode(',', $query['fields']);

    $users = empty($spec)
        ? $store->get('users')
        : $store->executeQuery('users', $spec);

    json_response(['data' => $users, 'total' => $store->count('users')]);
}

// GET /users/{id}
if ($method === 'GET' && preg_match('#^/users/(\d+)$#', $path, $m)) {
    $user = $store->findOne('users', ['id' => (int) $m[1]]);
    if (!$user) json_response(['error' => 'User not found'], 404);
    json_response(['data' => $user]);
}

// POST /users
if ($method === 'POST' && $path === '/users') {
    $body = get_body();

    // Validate input
    $errors = [];
    if (empty($body['name']))  $errors[] = 'name is required';
    if (empty($body['email'])) $errors[] = 'email is required';
    if (!empty($errors)) json_response(['errors' => $errors], 422);

    // Check unique email
    $existing = $store->findOne('users', ['email' => $body['email']]);
    if ($existing) json_response(['error' => 'Email already exists'], 409);

    // Create
    $id = $store->get('meta.next_id') ?? 1;
    $user = [
        'id'    => $id,
        'name'  => $body['name'],
        'email' => $body['email'],
        'role'  => $body['role'] ?? 'user',
        'age'   => (int) ($body['age'] ?? 0),
    ];
    $store->push('users', $user);
    $store->increment('meta.next_id');

    // Rebuild indexes
    $store->createIndex('users', 'id');
    $store->createIndex('users', 'email');

    json_response(['data' => $user], 201);
}

// PUT /users/{id}
if ($method === 'PUT' && preg_match('#^/users/(\d+)$#', $path, $m)) {
    $targetId = (int) $m[1];
    $body = get_body();
    $users = $store->get('users');
    $found = false;

    foreach ($users as $i => $user) {
        if (($user['id'] ?? null) === $targetId) {
            $users[$i] = array_merge($user, $body, ['id' => $targetId]);
            $found = true;
            break;
        }
    }

    if (!$found) json_response(['error' => 'User not found'], 404);

    $store->set('users', $users);
    json_response(['data' => $users[$i]]);
}

// DELETE /users/{id}
if ($method === 'DELETE' && preg_match('#^/users/(\d+)$#', $path, $m)) {
    $targetId = (int) $m[1];
    $users = $store->get('users');
    $newUsers = array_values(array_filter($users, fn($u) => ($u['id'] ?? null) !== $targetId));

    if (count($newUsers) === count($users)) {
        json_response(['error' => 'User not found'], 404);
    }

    $store->set('users', $newUsers);
    json_response(['deleted' => true]);
}

// 404
json_response(['error' => 'Not found', 'path' => $path], 404);
