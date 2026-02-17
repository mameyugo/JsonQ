# Real-Time Analytics Example

Build a real-time analytics system using JsonQ for tracking events, page views, and user behavior.

## Overview

This example shows how to use JsonQ for real-time analytics and metrics tracking.

---

## Event Tracking

```php
<?php

use JsonQ\Store;

class AnalyticsTracker {
    private Store $store;
    
    public function __construct(string $path = 'data/analytics.json') {
        $this->store = new Store($path);
        
        // Create indexes
        $this->store->createIndex('events', 'type');
        $this->store->createCompoundIndex('events', ['user_id', 'type']);
    }
    
    public function trackEvent(array $data): void {
        $event = [
            'id' => uniqid(),
            'type' => $data['type'],
            'user_id' => $data['user_id'] ?? null,
            'session_id' => $data['session_id'] ?? null,
            'data' => $data['data'] ?? [],
            'timestamp' => microtime(true),
            'date' => date('Y-m-d'),
            'hour' => date('H'),
            'ip' => $_SERVER['REMOTE_ADDR'] ?? null,
            'user_agent' => $_SERVER['HTTP_USER_AGENT'] ?? null
        ];
        
        $this->store->push('events', $event);
    }
    
    public function trackPageView(string $url, ?int $userId = null): void {
        $this->trackEvent([
            'type' => 'page_view',
            'user_id' => $userId,
            'data' => ['url' => $url]
        ]);
    }
    
    public function trackClick(string $element, ?int $userId = null): void {
        $this->trackEvent([
            'type' => 'click',
            'user_id' => $userId,
            'data' => ['element' => $element]
        ]);
    }
    
    // Analytics queries
    public function getPageViews(string $date): int {
        $events = $this->store->find('events', [
            'type' => 'page_view',
            'date' => $date
        ]);
        return count($events);
    }
    
    public function getUniqueVisitors(string $date): int {
        $events = $this->store->find('events', [
            'type' => 'page_view',
            'date' => $date,
            'user_id' => ['$exists' => true]
        ]);
        
        $uniqueUsers = array_unique(array_column($events, 'user_id'));
        return count($uniqueUsers);
    }
    
    public function getTopPages(string $date, int $limit = 10): array {
        $events = $this->store->find('events', [
            'type' => 'page_view',
            'date' => $date
        ]);
        
        $pages = [];
        foreach ($events as $event) {
            $url = $event['data']['url'] ?? 'unknown';
            $pages[$url] = ($pages[$url] ?? 0) + 1;
        }
        
        arsort($pages);
        return array_slice($pages, 0, $limit, true);
    }
    
    public function getUserActivity(int $userId, int $days = 7): array {
        $since = date('Y-m-d', strtotime("-{$days} days"));
        
        $events = $this->store->find('events', [
            'user_id' => $userId,
            'date' => ['$gte' => $since]
        ]);
        
        return [
            'total_events' => count($events),
            'event_types' => $this->store->groupBy('events', 'type'),
            'daily_activity' => $this->groupByDate($events)
        ];
    }
    
    public function getRealTimeMetrics(): array {
        $now = date('Y-m-d H:i:s');
        $oneHourAgo = date('Y-m-d H:i:s', strtotime('-1 hour'));
        
        $recent = $this->store->find('events', [
            'timestamp' => ['$gte' => strtotime($oneHourAgo)]
        ]);
        
        return [
            'events_last_hour' => count($recent),
            'active_users' => count(array_unique(array_column($recent, 'user_id'))),
            'event_breakdown' => $this->groupByType($recent)
        ];
    }
    
    private function groupByDate(array $events): array {
        $byDate = [];
        foreach ($events as $event) {
            $date = $event['date'];
            $byDate[$date] = ($byDate[$date] ?? 0) + 1;
        }
        return $byDate;
    }
    
    private function groupByType(array $events): array {
        $byType = [];
        foreach ($events as $event) {
            $type = $event['type'];
            $byType[$type] = ($byType[$type] ?? 0) + 1;
        }
        return $byType;
    }
}

// Usage
$analytics = new AnalyticsTracker();

// Track events
$analytics->trackPageView('/products', 123);
$analytics->trackClick('#buy-button', 123);
$analytics->trackEvent([
    'type' => 'purchase',
    'user_id' => 123,
    'data' => ['product_id' => 456, 'amount' => 99.99]
]);

// Get metrics
$today = date('Y-m-d');
$pageViews = $analytics->getPageViews($today);
$uniqueVisitors = $analytics->getUniqueVisitors($today);
$topPages = $analytics->getTopPages($today, 5);
$realTime = $analytics->getRealTimeMetrics();

echo "Today: {$pageViews} views, {$uniqueVisitors} unique visitors\n";
print_r($topPages);
print_r($realTime);
```

---

## Dashboard Example

```php
class AnalyticsDashboard {
    private AnalyticsTracker $tracker;
    
    public function __construct(AnalyticsTracker $tracker) {
        $this->tracker = $tracker;
    }
    
    public function getDashboardData(): array {
        $today = date('Y-m-d');
        $yesterday = date('Y-m-d', strtotime('-1 day'));
        
        return [
            'today' => [
                'page_views' => $this->tracker->getPageViews($today),
                'unique_visitors' => $this->tracker->getUniqueVisitors($today)
            ],
            'yesterday' => [
                'page_views' => $this->tracker->getPageViews($yesterday),
                'unique_visitors' => $this->tracker->getUniqueVisitors($yesterday)
            ],
            'top_pages' => $this->tracker->getTopPages($today, 10),
            'real_time' => $this->tracker->getRealTimeMetrics()
        ];
    }
}
```

---

## Features

✅ Event tracking with metadata  
✅ Real-time metrics  
✅ User activity analysis  
✅ Page view analytics  
✅ Compound indexes for performance  

---

## See Also

- [Indexing Guide](../guides/indexing.md)
- [Queries Guide](../guides/queries.md)
- [Aggregation Guide](../guides/aggregation.md)
