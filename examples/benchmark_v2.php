<?php
/**
 * JsonQ Benchmark Suite v2 (Advanced Features + High Load)
 *
 * Usage: php -d "extension=/path/to/libjsonq.so" examples/benchmark_v2.php
 */

echo "╔═══════════════════════════════════════════════════════════════════╗\n";
echo "║          JsonQ Benchmark Suite v" . jsonq_version() . " (Advanced + 100K)          ║\n";
echo "╚═══════════════════════════════════════════════════════════════════╝\n\n";

// Sizes to test: 10K (standard), 100K (high load)
$sizes = [10_000, 100_000];

// ── Helpers ──

function generate_users(int $count): array {
    $roles  = ['admin', 'user', 'viewer', 'editor', 'moderator'];
    $users  = [];
    for ($i = 0; $i < $count; $i++) {
        $users[] = [
            'id'    => $i + 1,
            'name'  => "User_{$i}",
            'email' => "user{$i}@example.com",
            'age'   => rand(18, 65),
            'role'  => $roles[array_rand($roles)],
            'score' => rand(0, 100),
            'active'=> (bool)rand(0, 1),
        ];
    }
    return $users;
}

function bench(string $label, callable $fn, int $iterations = 100): float {
    // Warmup
    for ($i = 0; $i < min(5, $iterations); $i++) $fn();

    $start = hrtime(true);
    // For very slow ops, reduce iterations dynamically if needed, but let's stick to fixed for consistency
    for ($i = 0; $i < $iterations; $i++) $fn();
    $elapsed = (hrtime(true) - $start) / 1e6; // ms
    $per_op = $elapsed / $iterations;

    printf("  %-45s %8.2f ms total  %8.3f ms/op\n", $label, $elapsed, $per_op);
    return $per_op;
}

function compare(float $jsonq_ms, float $php_ms): void {
    if ($jsonq_ms == 0) $jsonq_ms = 0.0001; // Avoid div/0
    $ratio = $php_ms / $jsonq_ms;
    $faster = $ratio > 1 ? 'JsonQ' : 'PHP';
    $r = $ratio > 1 ? $ratio : 1 / $ratio;
    printf("  → %s is %.1fx faster\n\n", $faster, $r);
}

// ── Main Loop ──

foreach ($sizes as $size) {
    echo "━━━ {$size} records ━━━\n\n";

    $users = generate_users($size);
    // Adjust iterations based on size
    $iterations = max(5, (int)(10000 / ($size / 1000))); // e.g. 10K -> 1000 iters? No, 10000/10 = 1000. 100K -> 100.
    if ($size >= 100_000) $iterations = 20;

    // File paths
    $jsonqPath = "/tmp/jsonq_bench_{$size}.json";
    $jsonlPath = "/tmp/jsonq_bench_{$size}.jsonl";
    $phpPath   = "/tmp/php_bench_{$size}.json";
    $phpJsonl  = "/tmp/php_bench_{$size}.jsonl";

    // Initialize Store
    if (file_exists($jsonqPath)) unlink($jsonqPath);
    $store = new JsonQ\Store($jsonqPath);
    
    // 1. Basic Write (Baseline)
    // We need data in the store for subsequent tests
    $store->set('users', $users);
    
    // Create PHP baseline file
    file_put_contents($phpPath, json_encode(['users' => $users]));

    // ── Stream I/O ──
    echo "[Stream I/O]\n";
    $streamOut = "/tmp/jsonq_stream_out_{$size}.json";
    $rs = bench("JsonQ jsonq_write_to_file (Stream)", function() use ($jsonqPath, $streamOut) {
        jsonq_write_to_file($jsonqPath, $streamOut, false);
    }, $iterations);

    $ps = bench("PHP file_get + file_put (Memory)", function() use ($phpPath, $streamOut) {
        // PHP approach: Read all, write all
        $data = file_get_contents($phpPath);
        file_put_contents($streamOut, $data);
    }, $iterations);
    compare($rs, $ps);

    // ── JSONL Append ──
    echo "[JSONL Append]\n";
    // Prepare a line
    $line = ['id' => 999999, 'name' => 'NewUser', 'role' => 'admin'];
    if (file_exists($jsonlPath)) unlink($jsonlPath);
    if (file_exists($phpJsonl)) unlink($phpJsonl);

    // Initialize files for append test
    touch($jsonlPath);
    touch($phpJsonl);

    $rl = bench("JsonQ jsonq_append_jsonl", function() use ($jsonlPath, $line) {
        jsonq_append_jsonl($jsonlPath, $line);
    }, $iterations * 10);

    $pl = bench("PHP file_put_contents(FILE_APPEND)", function() use ($phpJsonl, $line) {
        file_put_contents($phpJsonl, json_encode($line) . "\n", FILE_APPEND);
    }, $iterations * 10);
    compare($rl, $pl);

    // ── Regex Find ──
    echo "[Regex Find (name starts with 'User_10')]\n";
    $rr = bench("JsonQ find(\$regex)", function() use ($store) {
        $store->find('users', ['name' => ['$regex' => '^User_10']]);
    }, $iterations);

    $pr = bench("PHP array_filter + preg_match", function() use ($phpPath) {
        $data = json_decode(file_get_contents($phpPath), true);
        array_filter($data['users'], fn($u) => preg_match('/^User_10/', $u['name']));
    }, $iterations);
    compare($rr, $pr);

    // ── JSONPath Slice ──
    echo "[JSONPath Slice (first 100 items)]\n";
    // Note: jsonq_query_node re-reads file every time currently?
    // Yes, jsonq_query_node takes path, so it opens Store, reads (cached), parses, applies.
    $rq = bench("JsonQ jsonq_query_node(users[0:100])", function() use ($jsonqPath) {
        jsonq_query_node($jsonqPath, "users[0:100]");
    }, $iterations);

    $pq = bench("PHP json_decode + array_slice", function() use ($phpPath) {
        $data = json_decode(file_get_contents($phpPath), true);
        array_slice($data['users'], 0, 100);
    }, $iterations);
    compare($rq, $pq); // Expect PHP might be faster due to re-opening/parsing overhead in current impl?

    // ── Aggregation ──
    echo "[Aggregation (avg age)]\n";
    $ra = bench("JsonQ aggregate(avg)", function() use ($store) {
        $store->aggregate('users', 'age', 'avg');
    }, $iterations);

    $pa = bench("PHP array_sum + count", function() use ($phpPath) {
        $data = json_decode(file_get_contents($phpPath), true);
        array_sum(array_column($data['users'], 'age')) / count($data['users']);
    }, $iterations);
    compare($ra, $pa);

    // Cleanup
    @unlink($jsonqPath);
    @unlink($phpPath);
    @unlink($jsonlPath);
    @unlink($phpJsonl);
    @unlink($streamOut);
    echo "\n";
}

// Extension size
echo "━━━ Extension Info ━━━\n\n";
$so = '/usr/lib/php/20230831/jsonq.so';
if (!file_exists($so)) {
    // Try local
     $so = glob(__DIR__ . '/../target/release/libjsonq.so')[0] ?? '';
}

if (file_exists($so)) {
    $size = filesize($so);
    echo "Size: " . round($size / 1024 / 1024, 2) . " MB\n";
}

echo "✅ Benchmark v2 complete!\n";
