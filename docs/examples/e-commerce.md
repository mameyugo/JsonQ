# E-Commerce Example with JsonQ

Build a simple e-commerce system using JsonQ for product catalog, shopping cart, and order management.

## Overview

This example demonstrates how to build an e-commerce backend with JsonQ, including:
- Product catalog management
- Shopping cart functionality
- Order processing
- Inventory tracking

---

## Data Structure

```php
// data/store.json
{
  "products": [
    {
      "id": 1,
      "sku": "LAPTOP-001",
      "name": "Gaming Laptop",
      "category": "electronics",
      "price": 1299.99,
      "stock": 15,
      "description": "High-performance gaming laptop"
    }
  ],
  "carts": [
    {
      "user_id": 1,
      "items": [
        {"product_id": 1, "quantity": 1, "price": 1299.99}
      ]
    }
  ],
  "orders": [
    {
      "id": 1,
      "user_id": 1,
      "items": [...],
      "total": 1299.99,
      "status": "pending",
      "created_at": "2026-02-17 10:30:00"
    }
  ]
}
```

---

## Product Manager

```php
<?php

use JsonQ\Store;

class ProductManager {
    private Store $store;
    
    public function __construct(string $path = 'data/store.json') {
        $this->store = new Store($path);
        
        // Create indexes for performance
        $this->store->createIndex('products', 'sku');
        $this->store->createIndex('products', 'category');
        $this->store->createCompoundIndex('orders', ['user_id', 'status']);
    }
    
    // Product CRUD
    public function getProducts(array $filters = []): array {
        if (empty($filters)) {
            return $this->store->get('products') ?? [];
        }
        
        return $this->store->find('products', $filters);
    }
    
    public function getProductById(int $id): ?array {
        return $this->store->findOne('products', ['id' => $id]);
    }
    
    public function searchProducts(string $query): array {
        return $this->store->find('products', [
            '$or' => [
                ['name' => ['$contains' => $query]],
                ['description' => ['$contains' => $query]],
                ['sku' => ['$contains' => $query]]
            ]
        ]);
    }
    
    public function getProductsByCategory(string $category): array {
        // Uses index for fast lookup
        return $this->store->find('products', ['category' => $category]);
    }
    
    public function getProductsInPriceRange(float $min, float $max): array {
        return $this->store->find('products', [
            'price' => ['$gte' => $min, '$lte' => $max]
        ]);
    }
    
    public function getInStockProducts(): array {
        return $this->store->find('products', [
            'stock' => ['$gt' => 0]
        ]);
    }
    
    public function updateStock(int $productId, int $quantity): bool {
        $products = $this->store->get('products');
        foreach ($products as $index => $product) {
            if ($product['id'] === $productId) {
                $products[$index]['stock'] += $quantity;
                $this->store->set('products', $products);
                return true;
            }
        }
        return false;
    }
}

class CartManager {
    private Store $store;
    
    public function __construct(Store $store) {
        $this->store = $store;
    }
    
    public function getCart(int $userId): array {
        $cart = $this->store->findOne('carts', ['user_id' => $userId]);
        return $cart ?? ['user_id' => $userId, 'items' => []];
    }
    
    public function addToCart(int $userId, int $productId, int $quantity = 1): bool {
        $product = $this->store->findOne('products', ['id' => $productId]);
        
        if (!$product || $product['stock'] < $quantity) {
            return false;
        }
        
        $carts = $this->store->get('carts') ?? [];
        $cartIndex = null;
        
        foreach ($carts as $index => $cart) {
            if ($cart['user_id'] === $userId) {
                $cartIndex = $index;
                break;
            }
        }
        
        if ($cartIndex === null) {
            // Create new cart
            $carts[] = [
                'user_id' => $userId,
                'items' => [
                    [
                        'product_id' => $productId,
                        'quantity' => $quantity,
                        'price' => $product['price'],
                        'name' => $product['name']
                    ]
                ]
            ];
        } else {
            // Add to existing cart
            $itemFound = false;
            foreach ($carts[$cartIndex]['items'] as $itemIndex => $item) {
                if ($item['product_id'] === $productId) {
                    $carts[$cartIndex]['items'][$itemIndex]['quantity'] += $quantity;
                    $itemFound = true;
                    break;
                }
            }
            
            if (!$itemFound) {
                $carts[$cartIndex]['items'][] = [
                    'product_id' => $productId,
                    'quantity' => $quantity,
                    'price' => $product['price'],
                    'name' => $product['name']
                ];
            }
        }
        
        $this->store->set('carts', $carts);
        return true;
    }
    
    public function removeFromCart(int $userId, int $productId): bool {
        $carts = $this->store->get('carts') ?? [];
        
        foreach ($carts as $index => $cart) {
            if ($cart['user_id'] === $userId) {
                $carts[$index]['items'] = array_filter(
                    $cart['items'],
                    fn($item) => $item['product_id'] !== $productId
                );
                $carts[$index]['items'] = array_values($carts[$index]['items']);
                $this->store->set('carts', $carts);
                return true;
            }
        }
        
        return false;
    }
    
    public function calculateTotal(int $userId): float {
        $cart = $this->getCart($userId);
        $total = 0;
        
        foreach ($cart['items'] as $item) {
            $total += $item['price'] * $item['quantity'];
        }
        
        return round($total, 2);
    }
    
    public function clearCart(int $userId): bool {
        $carts = $this->store->get('carts') ?? [];
        
        foreach ($carts as $index => $cart) {
            if ($cart['user_id'] === $userId) {
                array_splice($carts, $index, 1);
                $this->store->set('carts', $carts);
                return true;
            }
        }
        
        return false;
    }
}

class OrderManager {
    private Store $store;
    private ProductManager $productManager;
    private CartManager $cartManager;
    
    public function __construct(Store $store, ProductManager $pm, CartManager $cm) {
        $this->store = $store;
        $this->productManager = $pm;
        $this->cartManager = $cm;
    }
    
    public function createOrder(int $userId): ?array {
        $cart = $this->cartManager->getCart($userId);
        
        if (empty($cart['items'])) {
            return null;
        }
        
        // Begin transaction
        $this->store->begin();
        
        try {
            // Validate stock availability
            foreach ($cart['items'] as $item) {
                $product = $this->productManager->getProductById($item['product_id']);
                if ($product['stock'] < $item['quantity']) {
                    throw new Exception("Insufficient stock for {$product['name']}");
                }
            }
            
            // Create order
            $orders = $this->store->get('orders') ?? [];
            $orderId = empty($orders) ? 1 : max(array_column($orders, 'id')) + 1;
            
            $order = [
                'id' => $orderId,
                'user_id' => $userId,
                'items' => $cart['items'],
                'total' => $this->cartManager->calculateTotal($userId),
                'status' => 'pending',
                'created_at' => date('Y-m-d H:i:s'),
                'updated_at' => date('Y-m-d H:i:s')
            ];
            
            // Update stock
            foreach ($cart['items'] as $item) {
                $this->productManager->updateStock($item['product_id'], -$item['quantity']);
            }
            
            // Save order
            $this->store->push('orders', $order);
            
            // Clear cart
            $this->cartManager->clearCart($userId);
            
            // Commit transaction
            $this->store->commit();
            
            return $order;
            
        } catch (Exception $e) {
            $this->store->rollback();
            throw $e;
        }
    }
    
    public function getUserOrders(int $userId): array {
        return $this->store->find('orders', ['user_id' => $userId]);
    }
    
    public function getOrdersByStatus(string $status): array {
        return $this->store->find('orders', ['status' => $status]);
    }
    
    public function updateOrderStatus(int $orderId, string $status): bool {
        $orders = $this->store->get('orders');
        
        foreach ($orders as $index => $order) {
            if ($order['id'] === $orderId) {
                $orders[$index]['status'] = $status;
                $orders[$index]['updated_at'] = date('Y-m-d H:i:s');
                
                if ($status === 'completed') {
                    $orders[$index]['completed_at'] = date('Y-m-d H:i:s');
                }
                
                $this->store->set('orders', $orders);
                return true;
            }
        }
        
        return false;
    }
    
    public function getRevenueStats(): array {
        $completedOrders = $this->store->find('orders', ['status' => 'completed']);
        
        if (empty($completedOrders)) {
            return [
                'total_orders' => 0,
                'total_revenue' => 0,
                'average_order_value' => 0
            ];
        }
        
        $stats = $this->store->aggregate('orders', [
            'count' => 'id',
            'sum' => 'total',
            'avg' => 'total'
        ]);
        
        return [
            'total_orders' => $stats['count'],
            'total_revenue' => $stats['sum'],
            'average_order_value' => $stats['avg']
        ];
    }
}
```

---

## Usage Example

```php
<?php

require_once 'ecommerce.php';

// Initialize
$store = new JsonQ\Store('data/store.json');
$productMgr = new ProductManager();
$cartMgr = new CartManager($store);
$orderMgr = new OrderManager($store, $productMgr, $cartMgr);

// Browse products
$electronics = $productMgr->getProductsByCategory('electronics');
$affordable = $productMgr->getProductsInPriceRange(100, 500);
$searchResults = $productMgr->searchProducts('laptop');

// Shopping cart
$userId = 1;
$cartMgr->addToCart($userId, 1, 2);  // Add 2x product ID 1
$cartMgr->addToCart($userId, 5, 1);  // Add 1x product ID 5
$total = $cartMgr->calculateTotal($userId);

// Checkout
try {
    $order = $orderMgr->createOrder($userId);
    echo "Order #{$order['id']} created! Total: ${$order['total']}\n";
} catch (Exception $e) {
    echo "Error: " . $e->getMessage() . "\n";
}

// Admin: View orders
$pendingOrders = $orderMgr->getOrdersByStatus('pending');
$orderMgr->updateOrderStatus(1, 'completed');

// Analytics
$stats = $orderMgr->getRevenueStats();
echo "Total Revenue: $" . number_format($stats['total_revenue'], 2) . "\n";
echo "Average Order: $" . number_format($stats['average_order_value'], 2) . "\n";
```

---

## Features Demonstrated

✅ **Complex Queries**: Category filters, price ranges, text search  
✅ **Transactions**: ACID guarantees for order processing  
✅ **Indexing**: Fast lookups on SKU, category, and compound indexes  
✅ **Aggregations**: Revenue statistics and order analytics  
✅ **Data Integrity**: Stock validation and rollback on errors  

---

## See Also

- [Transactions Guide](../guides/transactions.md)
- [Indexing Guide](../guides/indexing.md)
- [Aggregation Guide](../guides/aggregation.md)
