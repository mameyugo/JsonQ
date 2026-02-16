<?php

$passed = 0;
$failed = 0;
$errors = [];

function test(string $name, callable $fn): void {
    global $passed, $failed, $errors;
    try {
        $fn();
        $passed++;
        echo "  ✓ {$name}\n";
    } catch (\Throwable $e) {
        $failed++;
        $errors[] = "{$name}: {$e->getMessage()} (line {$e->getLine()})";
        echo "  ✗ {$name}: {$e->getMessage()}\n";
    }
}

function assert_eq($expected, $actual, string $msg = ''): void {
    if ($expected !== $actual) {
        $e = var_export($expected, true);
        $a = var_export($actual, true);
        throw new \RuntimeException($msg ?: "Expected {$e}, got {$a}");
    }
}

function assert_true($val, string $msg = ''): void {
    if ($val !== true) throw new \RuntimeException($msg ?: "Expected true, got " . var_export($val, true));
}

function assert_false($val, string $msg = ''): void {
    if ($val !== false) throw new \RuntimeException($msg ?: "Expected false");
}

function assert_count(int $expected, $arr, string $msg = ''): void {
    $c = is_array($arr) ? count($arr) : -1;
    if ($c !== $expected) throw new \RuntimeException($msg ?: "Expected count {$expected}, got {$c}");
}

function assert_null($val, string $msg = ''): void {
    if ($val !== null) throw new \RuntimeException($msg ?: "Expected null");
}

function fresh_store(): \JsonQ\Store {
    $path = tempnam(sys_get_temp_dir(), 'jsonq_test_') . '.json';
    return new \JsonQ\Store($path);
}

function print_summary(): void {
    global $passed, $failed, $errors;
    $total = $passed + $failed;
    
    echo "\n" . str_repeat('═', 50) . "\n";
    echo "Results: {$passed}/{$total} passed";
    
    if ($failed > 0) {
        echo " ({$failed} failed)\n\n";
        echo "Failures:\n";
        foreach ($errors as $e) echo "  ✗ {$e}\n";
        echo "\n";
        exit(1);
    } else {
        echo " — ALL PASSED ✅\n\n";
        exit(0);
    }
}
