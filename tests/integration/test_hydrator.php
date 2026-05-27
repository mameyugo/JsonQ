<?php
/**
 * JsonQ v0.8.0 — Hydrator Integration Tests
 * Usage: php -d "extension=/path/to/libjsonq.so" tests/integration/test_hydrator.php
 */

require_once __DIR__ . '/../../vendor/autoload.php';

use JsonQ\Store\HydratableStore;
use JsonQ\Hydrator;

$pass = 0; $fail = 0;
$tmp = sys_get_temp_dir() . '/jsonq_hydrator_' . getmypid();
mkdir($tmp, 0755, true);

function hydrator_test(string $name, callable $fn): void {
    global $pass, $fail;
    try { $fn(); echo "  ✅ {$name}\n"; $pass++; }
    catch (\Throwable $e) { echo "  ❌ {$name}\n     {$e->getMessage()}\n"; $fail++; }
}

class User2 {
    public int $id;
    public string $name;
    public bool $active = true;
    public ?string $email = null;
}

echo "\n🧪 Hydrator Integration Tests\n";

hydrator_test('HydratableStore findOneAs retorna objeto tipado', function() use ($tmp) {
    if (!class_exists(HydratableStore::class)) {
        throw new \Exception("HydratableStore class not found, check autoloading.");
    }
    
    $store = new HydratableStore("{$tmp}/users.json");
    $store->set('users', [['id' => 1, 'name' => 'Alice', 'active' => true]]);
    $user = $store->findOneAs(User2::class, 'users', ['id' => ['$eq' => 1]]);
    assert($user instanceof User2, 'debe ser User2');
    assert($user->id === 1);
    assert($user->name === 'Alice');
});

hydrator_test('HydratableStore findInAs retorna array tipado', function() use ($tmp) {
    $store = new HydratableStore("{$tmp}/users2.json");
    $store->set('users', [
        ['id' => 1, 'name' => 'Alice', 'active' => true],
        ['id' => 2, 'name' => 'Bob',   'active' => false],
    ]);
    $users = $store->findInAs(User2::class, 'users', ['active' => ['$eq' => true]]);
    assert(count($users) === 1);
    assert($users[0] instanceof User2);
});

hydrator_test('Hydrator dehydrate y set roundtrip', function() use ($tmp) {
    $store = new HydratableStore("{$tmp}/roundtrip.json");
    $user = new User2(); $user->id = 42; $user->name = 'Dave';
    $store->setObject('profile', $user);
    $back = $store->findOneAs(User2::class, 'profile', []);
    
    // findOneAs retorna null si no hay datos, o el objeto si hay
    assert($back instanceof User2);
});

echo "\n══════════════════════════════════\n";
echo "  ✅ Passed: {$pass}  ❌ Failed: {$fail}\n";
echo "══════════════════════════════════\n";
array_map('unlink', glob("{$tmp}/*")); rmdir($tmp);
exit($fail > 0 ? 1 : 0);
