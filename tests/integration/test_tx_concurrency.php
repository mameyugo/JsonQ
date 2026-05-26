<?php
/**
 * Test concurrent multi-process transactions in JsonQ
 * 
 * Run via: php -d extension=target/release/libjsonq.so tests/integration/test_tx_concurrency.php
 */
use JsonQ\Store;

$path = tempnam(sys_get_temp_dir(), 'jsonq_tx_') . '.json';
file_put_contents($path, json_encode(['counter' => 0]));

if (!function_exists('pcntl_fork')) {
    echo "⚠️ pcntl extension not available, skipping multi-process PHP test.\n";
    unlink($path);
    exit(0);
}

echo "🧪 Running Multi-Process PHP Transaction Concurrency Test...\n";

// Fork child process to do concurrent writes
$pid = pcntl_fork();

if ($pid == -1) {
    unlink($path);
    die("Could not fork\n");
} elseif ($pid === 0) {
    // Child process
    $child_start = microtime(true);
    echo "[" . round($child_start, 4) . "] Child: Spawning and sleeping 50ms...\n";
    usleep(50000); // 50ms
    
    $s2_init = microtime(true);
    echo "[" . round($s2_init, 4) . "] Child: Creating Store instance...\n";
    $tx_start = microtime(true);
    $s2 = new Store($path);
    
    echo "[" . round(microtime(true), 4) . "] Child: Calling beginTransaction()...\n";
    $s2->beginTransaction();
    $tx_acquired = microtime(true);
    echo "[" . round($tx_acquired, 4) . "] Child: beginTransaction() returned after " . round($tx_acquired - $tx_start, 4) . "s total\n";
    
    $s2->increment('counter');
    $s2->commit();
    $child_done = microtime(true);
    echo "[" . round($child_done, 4) . "] Child: Finished transaction.\n";
    
    $elapsed = $tx_acquired - $tx_start;
    if ($elapsed < 0.1) {
        echo "✗ Child transaction did not block! Took {$elapsed}s\n";
        exit(1);
    }
    echo "✓ Child transaction blocked successfully. Took " . round($elapsed, 3) . "s\n";
    exit(0);
} else {
    // Parent process
    $parent_start = microtime(true);
    echo "[" . round($parent_start, 4) . "] Parent: Creating Store instance...\n";
    $s1 = new Store($path);
    
    $tx_start = microtime(true);
    echo "[" . round($tx_start, 4) . "] Parent: Calling beginTransaction()...\n";
    $s1->beginTransaction();
    echo "[" . round(microtime(true), 4) . "] Parent: Transaction active, incrementing and sleeping 200ms...\n";
    $s1->increment('counter');
    
    // Hold the transaction for 200ms
    usleep(200000); // 200ms
    
    echo "[" . round(microtime(true), 4) . "] Parent: Calling commit()...\n";
    $s1->commit();
    echo "[" . round(microtime(true), 4) . "] Parent: Transaction committed.\n";
    
    // Wait for child to exit
    pcntl_waitpid($pid, $status);
    $exitCode = pcntl_wexitstatus($status);
    
    $file_content = file_get_contents($path);
    echo "[" . round(microtime(true), 4) . "] Parent: Raw file content on disk: " . $file_content . "\n";
    echo "[" . round(microtime(true), 4) . "] Parent: filemtime on disk: " . filemtime($path) . "\n";
    
    $finalVal = $s1->get('counter');
    echo "[" . round(microtime(true), 4) . "] Parent: \$s1->get('counter') returned: " . $finalVal . "\n";
    unlink($path);
    
    if ($exitCode !== 0 || $finalVal !== 2.0) {
        echo "✗ Concurrency test failed. Exit code: $exitCode, Final Val: $finalVal\n";
        exit(1);
    }
    echo "✓ Multi-process PHP transaction concurrency test passed! Final value: $finalVal\n";
    exit(0);
}
