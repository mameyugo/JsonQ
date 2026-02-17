<?php
/**
 * JsonQ Benchmark Suite
 *
 * Compares JsonQ native extension vs pure PHP json_encode/json_decode.
 * Run: php -d "extension=path/to/libjsonq.so" examples/benchmark.php
 */

echo "╔══════════════════════════════════════════════════════╗\n";
echo "║          JsonQ Benchmark Suite v" . jsonq_version() . str_repeat(' ', 17) . "║\n";
echo "╚══════════════════════════════════════════════════════╝\n\n";

$sizes = [100, 1_000, 10_000];

// ── Generate test data ──
function generate_users(int $count): array {
    $roles  = ['admin', 'user', 'viewer', 'editor', 'moderator'];
    $cities = ['NYC', 'LA', 'Chicago', 'Houston', 'Phoenix', 'Philadelphia', 'San Antonio', 'San Diego'];
    $users  = [];
    for ($i = 0; $i < $count; $i++) {
        $users[] = [
            'id'    => $i + 1,
            'name'  => "User_{$i}",
            'email' => "user{$i}@example.com",
            'age'   => rand(18, 65),
            'role'  => $roles[array_rand($roles)],
            'city'  => $cities[array_rand($cities)],
            'score' => rand(0, 100),
            'active'=> (bool)rand(0, 1),
        ];
    }
    return $users;
}

function bench(string $label, callable $fn, int $iterations = 1000): float {
    // Warmup
    for ($i = 0; $i < min(10, $iterations); $i++) $fn();

    $start = hrtime(true);
    for ($i = 0; $i < $iterations; $i++) $fn();
    $elapsed = (hrtime(true) - $start) / 1e6; // ms
    $per_op = $elapsed / $iterations;

    printf("  %-38s %8.2f ms total  %8.3f ms/op\n", $label, $elapsed, $per_op);
    return $per_op;
}

function compare(float $jsonq_ms, float $php_ms): void {
    $ratio = $php_ms / $jsonq_ms;
    $faster = $ratio > 1 ? 'JsonQ' : 'PHP';
    $r = $ratio > 1 ? $ratio : 1 / $ratio;
    printf("  → %s is %.1fx faster\n\n", $faster, $r);
}

foreach ($sizes as $size) {
    echo "━━━ {$size} records ━━━\n\n";

    $users = generate_users($size);
    $iterations = max(10, (int)(10000 / $size));

    // Setup JsonQ store
    $JsonQPath = "/tmp/jsonq_bench_{$size}.json";
    $store = new JsonQ\Store($JsonQPath);
    $store->set('users', $users);

    // Setup PHP file
    $phpPath = "/tmp/php_bench_{$size}.json";
    file_put_contents($phpPath, json_encode(['users' => $users], JSON_PRETTY_PRINT));

    // ── Write ──
    echo "Write:\n";
    $rw = bench("JsonQ set()", function() use ($store, $users) {
        $store->set('users', $users);
    }, $iterations);

    $pw = bench("PHP json_encode + file_put_contents", function() use ($phpPath, $users) {
        file_put_contents($phpPath, json_encode(['users' => $users], JSON_PRETTY_PRINT));
    }, $iterations);
    compare($rw, $pw);

    // ── Read (cached) ──
    echo "Read (cached):\n";
    $rr = bench("JsonQ get()", function() use ($store) {
        $store->get('users');
    }, $iterations * 2);

    $pr = bench("PHP json_decode + file_get_contents", function() use ($phpPath) {
        json_decode(file_get_contents($phpPath), true);
    }, $iterations * 2);
    compare($rr, $pr);

    // ── Find (scan) ──
    echo "Find (full scan, role='admin'):\n";
    $rf = bench("JsonQ find()", function() use ($store) {
        $store->find('users', ['role' => 'admin']);
    }, $iterations);

    $pf = bench("PHP array_filter", function() use ($phpPath) {
        $data = json_decode(file_get_contents($phpPath), true);
        array_filter($data['users'], fn($u) => $u['role'] === 'admin');
    }, $iterations);
    compare($rf, $pf);

    // ── Find (indexed) ──
    echo "Find (indexed, role='admin'):\n";
    $store->createIndex('users', 'role');
    $ri = bench("JsonQ indexLookup()", function() use ($store) {
        $store->indexLookup('users', 'role', 'admin');
    }, $iterations * 5);
    echo sprintf("  (vs scan: %.3f ms/op → %.3f ms/op = %.0fx faster)\n\n", $rf, $ri, $rf / max($ri, 0.001));

    // ── Complex query ──
    echo "Complex query (age>25, score>50, limit 10, sorted):\n";
    $rc = bench("JsonQ executeQuery()", function() use ($store) {
        $store->executeQuery('users', [
            'where'    => [
                ['field' => 'age', 'op' => '>', 'value' => 25],
                ['field' => 'score', 'op' => '>', 'value' => 50],
            ],
            'order_by' => ['field' => 'score', 'direction' => 'desc'],
            'limit'    => 10,
        ]);
    }, $iterations);

    $pc = bench("PHP manual filter+sort+slice", function() use ($phpPath) {
        $data = json_decode(file_get_contents($phpPath), true);
        $filtered = array_filter($data['users'], fn($u) => $u['age'] > 25 && $u['score'] > 50);
        usort($filtered, fn($a, $b) => $b['score'] <=> $a['score']);
        array_slice($filtered, 0, 10);
    }, $iterations);
    compare($rc, $pc);

    // ── Aggregation ──
    echo "Aggregation (avg age):\n";
    $ra = bench("JsonQ aggregate()", function() use ($store) {
        $store->aggregate('users', 'age', 'avg');
    }, $iterations);

    $pa = bench("PHP array_sum + count", function() use ($phpPath) {
        $data = json_decode(file_get_contents($phpPath), true);
        array_sum(array_column($data['users'], 'age')) / count($data['users']);
    }, $iterations);
    compare($ra, $pa);

    // Cleanup
    $store->dropAllIndexes();
    unlink($JsonQPath);
    unlink($phpPath);
}

// ── File size comparison ──
echo "━━━ Extension size ━━━\n\n";
$soFiles = glob(dirname(__DIR__) . '/target/release/libjsonq.so');
if (empty($soFiles)) $soFiles = glob(dirname(__DIR__) . '/target/release/libjsonq.dylib');
if (!empty($soFiles)) {
    $soSize = filesize($soFiles[0]);
    $h = $soSize < 1048576 ? sprintf("%.0f KB", $soSize / 1024) : sprintf("%.1f MB", $soSize / 1048576);
    echo "  libjsonq.so: {$h}\n";
} else {
    echo "  (extension file not found in expected path)\n";
}

echo "\n✅ Benchmark complete!\n";
