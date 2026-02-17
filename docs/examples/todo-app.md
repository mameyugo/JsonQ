# Building a TODO Application with JsonQ

Learn how to build a simple CRUD todo application using JsonQ for data storage.

## Overview

This tutorial demonstrates how to build a complete TODO application with JsonQ, covering create, read, update, and delete operations with proper validation.

---

## Project Setup

### 1. Install JsonQ

Follow the [installation guide](../getting-started/installation.md) to install JsonQ.

### 2. Create Project Structure

```
todo-app/
├── index.php
├── todo.php
├── data/
│   └── todos.json
└── views/
    ├── list.php
    └── form.php
```

---

## Todo Class

Create `todo.php` to handle all todo operations:

```php
<?php

use JsonQ\Store;

class TodoManager {
    private Store $store;
    
    public function __construct(string $dataPath = 'data/todos.json') {
        $this->store = new Store($dataPath);
        
        // Initialize with empty todos if file is new
        if (!$this->store->has('todos')) {
            $this->store->set('todos', []);
        }
        
        // Create index on id for fast lookups
        $this->store->createIndex('todos', 'id');
    }
    
    /**
     * Get all todos, optionally filtered by status
     */
    public function getAll(?string $status = null): array {
        if ($status) {
            return $this->store->find('todos', ['status' => $status]);
        }
        return $this->store->get('todos') ?? [];
    }
    
    /**
     * Get a single todo by ID
     */
    public function getById(int $id): ?array {
        return $this->store->findOne('todos', ['id' => $id]);
    }
    
    /**
     * Create a new todo
     */
    public function create(array $data): array {
        // Validate input
        $errors = $this->validate($data);
        if (!empty($errors)) {
            throw new InvalidArgumentException('Validation failed: ' . implode(', ', $errors));
        }
        
        // Generate ID
        $todos = $this->store->get('todos') ?? [];
        $id = empty($todos) ? 1 : max(array_column($todos, 'id')) + 1;
        
        // Create todo
        $todo = [
            'id' => $id,
            'title' => $data['title'],
            'description' => $data['description'] ?? '',
            'status' => 'pending',
            'priority' => $data['priority'] ?? 'medium',
            'created_at' => date('Y-m-d H:i:s'),
            'updated_at' => date('Y-m-d H:i:s'),
            'completed_at' => null
        ];
        
        // Save
        $this->store->push('todos', $todo);
        
        return $todo;
    }
    
    /**
     * Update an existing todo
     */
    public function update(int $id, array $data): bool {
        $todos = $this->store->get('todos') ?? [];
        
        foreach ($todos as $index => $todo) {
            if ($todo['id'] === $id) {
                // Update allowed fields
                if (isset($data['title'])) {
                    $todos[$index]['title'] = $data['title'];
                }
                if (isset($data['description'])) {
                    $todos[$index]['description'] = $data['description'];
                }
                if (isset($data['priority'])) {
                    $todos[$index]['priority'] = $data['priority'];
                }
                if (isset($data['status'])) {
                    $todos[$index]['status'] = $data['status'];
                    if ($data['status'] === 'completed') {
                        $todos[$index]['completed_at'] = date('Y-m-d H:i:s');
                    }
                }
                
                $todos[$index]['updated_at'] = date('Y-m-d H:i:s');
                
                $this->store->set('todos', $todos);
                return true;
            }
        }
        
        return false;
    }
    
    /**
     * Delete a todo
     */
    public function delete(int $id): bool {
        $todos = $this->store->get('todos') ?? [];
        
        foreach ($todos as $index => $todo) {
            if ($todo['id'] === $id) {
                array_splice($todos, $index, 1);
                $this->store->set('todos', $todos);
                return true;
            }
        }
        
        return false;
    }
    
    /**
     * Toggle todo status
     */
    public function toggleStatus(int $id): bool {
        $todo = $this->getById($id);
        if (!$todo) {
            return false;
        }
        
        $newStatus = $todo['status'] === 'completed' ? 'pending' : 'completed';
        return $this->update($id, ['status' => $newStatus]);
    }
    
    /**
     * Get statistics
     */
    public function getStats(): array {
        $all = $this->getAll();
        $completed = $this->store->find('todos', ['status' => 'completed']);
        $pending = $this->store->find('todos', ['status' => 'pending']);
        
        return [
            'total' => count($all),
            'completed' => count($completed),
            'pending' => count($pending),
            'completion_rate' => count($all) > 0 
                ? round((count($completed) / count($all)) * 100, 1) 
                : 0
        ];
    }
    
    /**
     * Search todos
     */
    public function search(string $query): array {
        return $this->store->find('todos', [
            '$or' => [
                ['title' => ['$contains' => $query]],
                ['description' => ['$contains' => $query]]
            ]
        ]);
    }
    
    /**
     * Validate todo data
     */
    private function validate(array $data): array {
        $errors = [];
        
        if (empty($data['title']) || strlen($data['title']) < 3) {
            $errors[] = 'Title must be at least 3 characters';
        }
        
        if (isset($data['priority']) && !in_array($data['priority'], ['low', 'medium', 'high'])) {
            $errors[] = 'Priority must be low, medium, or high';
        }
        
        return $errors;
    }
}
```

---

## Main Application

Create `index.php`:

```php
<?php

require_once 'todo.php';

$manager = new TodoManager();

// Handle actions
$action = $_GET['action'] ?? 'list';
$message = '';

try {
    switch ($action) {
        case 'create':
            if ($_SERVER['REQUEST_METHOD'] === 'POST') {
                $manager->create([
                    'title' => $_POST['title'],
                    'description' => $_POST['description'] ?? '',
                    'priority' => $_POST['priority'] ?? 'medium'
                ]);
                $message = 'Todo created successfully!';
            }
            break;
            
        case 'toggle':
            $id = (int)$_GET['id'];
            $manager->toggleStatus($id);
            header('Location: index.php');
            exit;
            
        case 'delete':
            $id = (int)$_GET['id'];
            $manager->delete($id);
            $message = 'Todo deleted!';
            break;
            
        case 'update':
            if ($_SERVER['REQUEST_METHOD'] === 'POST') {
                $id = (int)$_POST['id'];
                $manager->update($id, [
                    'title' => $_POST['title'],
                    'description' => $_POST['description'],
                    'priority' => $_POST['priority']
                ]);
                $message = 'Todo updated!';
            }
            break;
    }
} catch (Exception $e) {
    $message = 'Error: ' . $e->getMessage();
}

// Get filter
$filter = $_GET['filter'] ?? 'all';
$todos = $filter === 'all' 
    ? $manager->getAll() 
    : $manager->getAll($filter);

// Get stats
$stats = $manager->getStats();

?>

<!DOCTYPE html>
<html>
<head>
    <title>Todo App - JsonQ</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }
        .todo { padding: 15px; margin: 10px 0; border: 1px solid #ddd; border-radius: 5px; }
        .completed { background: #e8f5e9; text-decoration: line-through; }
        .pending { background: #fff3e0; }
        .priority-high { border-left: 5px solid #f44336; }
        .priority-medium { border-left: 5px solid #ff9800; }
        .priority-low { border-left: 5px solid #4caf50; }
        .stats { display: flex; gap: 20px; margin: 20px 0; }
        .stat { padding: 10px; background: #f5f5f5; border-radius: 5px; flex: 1; text-align: center; }
        .btn { padding: 8px 15px; margin: 5px; border: none; border-radius: 3px; cursor: pointer; }
        .btn-primary { background: #2196f3; color: white; }
        .btn-success { background: #4caf50; color: white; }
        .btn-danger { background: #f44336; color: white; }
        form { margin: 20px 0; padding: 20px; background: #f9f9f9; border-radius: 5px; }
        input, textarea { width: 100%; padding: 8px; margin: 5px 0; }
    </style>
</head>
<body>
    <h1>📝 Todo App with JsonQ</h1>
    
    <?php if ($message): ?>
        <div style="padding: 10px; background: #4caf50; color: white; border-radius: 5px; margin: 10px 0;">
            <?= htmlspecialchars($message) ?>
        </div>
    <?php endif; ?>
    
    <!-- Statistics -->
    <div class="stats">
        <div class="stat">
            <h3><?= $stats['total'] ?></h3>
            <p>Total Todos</p>
        </div>
        <div class="stat">
            <h3><?= $stats['completed'] ?></h3>
            <p>Completed</p>
        </div>
        <div class="stat">
            <h3><?= $stats['pending'] ?></h3>
            <p>Pending</p>
        </div>
        <div class="stat">
            <h3><?= $stats['completion_rate'] ?>%</h3>
            <p>Completion Rate</p>
        </div>
    </div>
    
    <!-- Create Form -->
    <form method="POST" action="?action=create">
        <h3>➕ Add New Todo</h3>
        <input type="text" name="title" placeholder="Title" required>
        <textarea name="description" placeholder="Description" rows="3"></textarea>
        <select name="priority">
            <option value="low">Low Priority</option>
            <option value="medium" selected>Medium Priority</option>
            <option value="high">High Priority</option>
        </select>
        <button type="submit" class="btn btn-primary">Create Todo</button>
    </form>
    
    <!-- Filter -->
    <div>
        <a href="?filter=all" class="btn <?= $filter === 'all' ? 'btn-primary' : '' ?>">All</a>
        <a href="?filter=pending" class="btn <?= $filter === 'pending' ? 'btn-primary' : '' ?>">Pending</a>
        <a href="?filter=completed" class="btn <?= $filter === 'completed' ? 'btn-primary' : '' ?>">Completed</a>
    </div>
    
    <!-- Todo List -->
    <div style="margin-top: 20px;">
        <?php foreach ($todos as $todo): ?>
            <div class="todo <?= $todo['status'] ?> priority-<?= $todo['priority'] ?>">
                <h3><?= htmlspecialchars($todo['title']) ?></h3>
                <p><?= htmlspecialchars($todo['description']) ?></p>
                <small>
                    Priority: <?= ucfirst($todo['priority']) ?> | 
                    Created: <?= $todo['created_at'] ?>
                    <?php if ($todo['completed_at']): ?>
                        | Completed: <?= $todo['completed_at'] ?>
                    <?php endif; ?>
                </small>
                <div style="margin-top: 10px;">
                    <a href="?action=toggle&id=<?= $todo['id'] ?>" class="btn btn-success">
                        <?= $todo['status'] === 'completed' ? 'Mark Pending' : 'Mark Complete' ?>
                    </a>
                    <a href="?action=delete&id=<?= $todo['id'] ?>" class="btn btn-danger" 
                       onclick="return confirm('Delete this todo?')">Delete</a>
                </div>
            </div>
        <?php endforeach; ?>
        
        <?php if (empty($todos)): ?>
            <p style="text-align: center; color: #999; padding: 40px;">
                No todos found. Create one above!
            </p>
        <?php endif; ?>
    </div>
</body>
</html>
```

---

## Features Demonstrated

### ✅ CRUD Operations
- **Create**: Add new todos with validation
- **Read**: List all todos with filtering
- **Update**: Toggle status, edit details
- **Delete**: Remove todos

### ✅ Querying
- Filter by status (pending/completed)
- Search by title and description
- Use MongoDB-style queries

### ✅ Indexing
- Index on `id` field for fast lookups
- O(1) performance for finding todos by ID

### ✅ Aggregation
- Calculate completion statistics
- Count total, completed, and pending todos
- Compute completion rate percentage

### ✅ Data Validation
- Validate title length
- Validate priority values
- Handle errors gracefully

---

## Running the Application

1. **Set up the project**:
   ```bash
   mkdir todo-app
   cd todo-app
   mkdir data
   ```

2. **Copy the code** above into `index.php` and `todo.php`

3. **Start PHP development server**:
   ```bash
   php -S localhost:8000
   ```

4. **Open in browser**:
   ```
   http://localhost:8000
   ```

---

## Next Steps

### Enhancements

1. **Add Categories/Tags**:
   ```php
   $manager->create([
       'title' => 'Learn JsonQ',
       'tags' => ['programming', 'php', 'database']
   ]);
   
   // Query by tag
   $programmingTodos = $store->find('todos', [
       'tags' => ['$contains' => 'programming']
   ]);
   ```

2. **Add Due Dates**:
   ```php
   // Find overdue todos
   $overdue = $store->find('todos', [
       'due_date' => ['$lt' => date('Y-m-d')],
       'status' => 'pending'
   ]);
   ```

3. **Add User Support**:
   ```php
   // Multi-user todos
   $myTodos = $store->find('todos', ['user_id' => $currentUserId]);
   ```

4. **Add Schema Validation**:
   ```php
   $schema = [
       'type' => 'object',
       'required' => ['title', 'status'],
       'properties' => [
           'title' => ['type' => 'string', 'minLength' => 3],
           'status' => ['type' => 'string', 'enum' => ['pending', 'completed']],
           'priority' => ['type' => 'string', 'enum' => ['low', 'medium', 'high']]
       ]
   ];
   
   $result = $store->validate('todos', $schema);
   ```

---

## See Also

- [Querying Guide](../guides/queries.md) - Advanced query techniques
- [Schema Validation](../guides/schema-validation.md) - Validate your data
- [Indexing Guide](../guides/indexing.md) - Optimize performance
- [E-Commerce Example](e-commerce.md) - More complex application
