<?php
/**
 * JsonQ v0.8.0 — Point-in-Time Recovery (PITR) & Revision History Integration Tests
 * Usage: php -d "extension=/path/to/libjsonq.so" tests/integration/test_pitr.php
 */

require_once __DIR__ . '/../../vendor/autoload.php';

use JsonQ\Store;

$pass = 0; $fail = 0;
$tmp = sys_get_temp_dir() . '/jsonq_pitr_' . getmypid();
mkdir($tmp, 0755, true);

function pitr_test(string $name, callable $fn): void {
    global $pass, $fail;
    try { $fn(); echo "  ✅ {$name}\n"; $pass++; }
    catch (\Throwable $e) { echo "  ❌ {$name}\n     {$e->getMessage()}\n"; $fail++; }
}

echo "\n⏳ Point-in-Time Recovery (PITR) Tests\n";

pitr_test('Revision logging creates journal file with correct format', function() use ($tmp) {
    $dbPath = "{$tmp}/test_format.json";
    $journalPath = "{$tmp}/test_format.json.journal";
    
    if (file_exists($dbPath)) unlink($dbPath);
    if (file_exists($journalPath)) unlink($journalPath);

    $s = new Store($dbPath);
    
    // Perform set mutation
    $s->set('key1', 'value1');
    
    assert(file_exists($journalPath), 'Journal file should be created');
    
    $lines = file($journalPath, FILE_IGNORE_NEW_LINES | FILE_SKIP_EMPTY_LINES);
    assert(count($lines) === 1, 'Should have exactly one log line');
    
    $log = json_decode($lines[0], true);
    assert($log['id'] === 1, 'First revision ID should be 1');
    assert($log['op'] === 'set', 'Op should be set');
    assert($log['path'] === 'key1', 'Path should be key1');
    assert($log['old'] === null, 'Old value should be null');
    assert($log['new'] === 'value1', 'New value should be value1');
    assert($log['existed'] === false, 'existed should be false');
    
    // Perform update
    $s->set('key1', 'value2');
    $lines = file($journalPath, FILE_IGNORE_NEW_LINES | FILE_SKIP_EMPTY_LINES);
    assert(count($lines) === 2, 'Should have two log lines');
    
    $log = json_decode($lines[1], true);
    assert($log['id'] === 2, 'Second revision ID should be 2');
    assert($log['old'] === 'value1', 'Old value should be value1');
    assert($log['new'] === 'value2', 'New value should be value2');
    assert($log['existed'] === true, 'existed should be true');
});

pitr_test('History can be retrieved and filtered by path', function() use ($tmp) {
    $dbPath = "{$tmp}/test_history.json";
    $s = new Store($dbPath);
    
    $s->set('user.name', 'Alice');
    $s->set('config.theme', 'dark');
    $s->set('user.role', 'admin');
    
    // Full history
    $h = $s->history();
    assert(count($h) === 3, 'Should have 3 revisions');
    
    // Filtered history by 'user'
    $hu = $s->history('user');
    assert(count($hu) === 2, 'Should have 2 user revisions');
    assert($hu[0]['path'] === 'user.name');
    assert($hu[1]['path'] === 'user.role');
    
    // Filtered history by specific path 'config.theme'
    $hc = $s->history('config.theme');
    assert(count($hc) === 1, 'Should have 1 config.theme revision');
    assert($hc[0]['path'] === 'config.theme');
});

pitr_test('RollbackTo reverts changes and truncates journal', function() use ($tmp) {
    $dbPath = "{$tmp}/test_rollback.json";
    $journalPath = "{$tmp}/test_rollback.json.journal";
    
    $s = new Store($dbPath);
    
    $s->set('name', 'Bob');         // Rev 1
    $s->set('role', 'user');        // Rev 2
    $s->set('theme', 'blue');       // Rev 3
    
    // Rollback to Rev 2
    $res = $s->rollbackTo(2);
    assert($res === true, 'Rollback should succeed');
    
    // Verify values reverted
    assert($s->get('name') === 'Bob');
    assert($s->get('role') === 'user');
    assert(!$s->has('theme'), 'theme should be deleted');
    
    // Verify journal file truncated to 2 lines
    $lines = file($journalPath, FILE_IGNORE_NEW_LINES | FILE_SKIP_EMPTY_LINES);
    assert(count($lines) === 2, 'Journal should contain exactly 2 lines');
    
    // Rollback to Rev 0 (beginning)
    $s->rollbackTo(0);
    assert(!$s->has('name'), 'name should be deleted');
    assert(!$s->has('role'), 'role should be deleted');
    
    $lines = file($journalPath, FILE_IGNORE_NEW_LINES | FILE_SKIP_EMPTY_LINES);
    assert(count($lines) === 0, 'Journal should be empty');
});

pitr_test('RollbackToTimestamp reverts to correct point in time', function() use ($tmp) {
    $dbPath = "{$tmp}/test_timestamp.json";
    $s = new Store($dbPath);
    
    $s->set('x', 10);
    $t1 = time();
    
    sleep(2);
    $s->set('x', 20);
    $t2 = time();
    
    sleep(2);
    $s->set('x', 30);
    
    // Rollback to t1 (should restore x = 10)
    $s->rollbackToTimestamp($t1);
    assert($s->get('x') === 10, 'x should be 10 at t1');
});

pitr_test('Option revision_log toggles logging', function() use ($tmp) {
    $dbPath = "{$tmp}/test_toggle.json";
    $journalPath = "{$tmp}/test_toggle.json.journal";
    
    $s = new Store($dbPath);
    
    // Verify revision_log is true by default
    assert($s->getOption('revision_log') === true, 'revision_log should be true by default');
    
    $s->set('a', 1);
    assert(file_exists($journalPath), 'Journal created');
    
    // Turn off revision log
    $s->setOption('revision_log', false);
    assert($s->getOption('revision_log') === false, 'revision_log should be false');
    
    $linesBefore = count(file($journalPath, FILE_IGNORE_NEW_LINES | FILE_SKIP_EMPTY_LINES));
    
    $s->set('b', 2);
    
    $linesAfter = count(file($journalPath, FILE_IGNORE_NEW_LINES | FILE_SKIP_EMPTY_LINES));
    assert($linesBefore === $linesAfter, 'No new lines should be logged');
});

echo "\n══════════════════════════════════\n";
echo "  ✅ Passed: {$pass}  ❌ Failed: {$fail}\n";
echo "══════════════════════════════════\n";

// Cleanup
array_map('unlink', glob("{$tmp}/*"));
rmdir($tmp);

exit($fail > 0 ? 1 : 0);
