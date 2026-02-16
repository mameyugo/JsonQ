# Example: REST API

This example demonstrates how to build a simple REST API for managing users using JsonQ.

## `api.php`

```php
<?php
require 'vendor/autoload.php'; // If using composer
// Or require your extension if manually installed

header('Content-Type: application/json');

$method = $_SERVER['REQUEST_METHOD'];
$path = trim($_SERVER['PATH_INFO'] ?? '/', '/');
$input = json_decode(file_get_contents('php://input'), true);

// Initialize Store
$store = new \JsonQ\Store(__DIR__ . '/db.json');

// Helper to send response
function jsonResponse($data, $status = 200) {
    http_response_code($status);
    echo json_encode($data);
    exit;
}

// Ensure 'users' collection exists
if (!$store->has('users')) {
    $store->set('users', []);
    $store->createIndex('users', 'id'); // Optimize by ID
}

// Router
if ($path === 'users') {
    switch ($method) {
        case 'GET':
            // Search / Filter
            $role = $_GET['role'] ?? null;
            if ($role) {
                // Use index if available, or scan
                $users = $store->find('users', ['role' => $role]);
            } else {
                $users = $store->get('users');
            }
            jsonResponse($users);
            break;

        case 'POST':
            // Create User
            $newUser = $input;
            if (!isset($newUser['name']) || !isset($newUser['email'])) {
                jsonResponse(['error' => 'Missing fields'], 400);
            }
            
            // Auto-increment ID
            $lastId = $store->aggregate('users', 'id', 'max') ?? 0;
            $newUser['id'] = $lastId + 1;
            
            $store->push('users', $newUser);
            jsonResponse($newUser, 201);
            break;
    }
} elseif (preg_match('/^users\/(\d+)$/', $path, $matches)) {
    $id = (int)$matches[1];
    
    // Find user index in array
    // Note: In a real app, you might map ID to array index or use a map structure
    // jsonQ currently uses array indices for updates/deletes effectively if you know the path
    
    // Find the user to get their current data
    $user = $store->findOne('users', ['id' => $id]);
    
    if (!$user) {
        jsonResponse(['error' => 'User not found'], 404);
    }

    switch ($method) {
        case 'GET':
            jsonResponse($user);
            break;

        case 'PUT':
            // Update User
            // We need to find the specific path "users.X"
            // For simplicity in this example, we scan. 
            // In high-perf scenarios, you'd maintain an ID->Index map or use a different structure.
            
            // Getting index via search (optimization needed for large datasets)
            $allUsers = $store->get('users');
            foreach ($allUsers as $idx => $u) {
                if ($u['id'] === $id) {
                    $updatePath = "users.$idx";
                    $store->merge($updatePath, $input);
                    jsonResponse($store->get($updatePath));
                }
            }
            break;

        case 'DELETE':
            $allUsers = $store->get('users');
            foreach ($allUsers as $idx => $u) {
                if ($u['id'] === $id) {
                    $store->remove("users.$idx");
                    // Note: This leaves a gap or reindexes array depending on implementation
                    // JsonQ remove on array works like unset(), keeping keys. 
                    // To reindex, you might reset the collection or use array_values()
                    $store->set('users', array_values($store->get('users')));
                    jsonResponse(['message' => 'Deleted']);
                }
            }
            break;
    }
} else {
    jsonResponse(['error' => 'Not found'], 404);
}
```

## Testing

**Create User:**
```bash
curl -X POST -d '{"name":"Alice","role":"admin","email":"alice@test.com"}' http://localhost/api.php/users
```

**List Users:**
```bash
curl http://localhost/api.php/users
```

**Get User:**
```bash
curl http://localhost/api.php/users/1
```
