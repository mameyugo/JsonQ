<?php
declare(strict_types=1);

namespace JsonQ\Tests;

use JsonQ\Store\HydratableStore;
use JsonQ\Tests\Fixtures\User;
use PHPUnit\Framework\TestCase;

/**
 * @requires extension jsonq
 */
class HydratableStoreTest extends TestCase
{
    private string $tmpFile;
    private HydratableStore $store;

    protected function setUp(): void
    {
        $this->tmpFile = sys_get_temp_dir() . '/jsonq_test_' . uniqid() . '.json';
        $this->store = new HydratableStore($this->tmpFile);
    }

    protected function tearDown(): void
    {
        if (file_exists($this->tmpFile)) {
            unlink($this->tmpFile);
        }
    }

    public function test_findOneAs_returns_typed_object(): void
    {
        $this->store->set('users', [
            ['id' => 1, 'name' => 'Alice', 'active' => true],
            ['id' => 2, 'name' => 'Bob',   'active' => false],
        ]);

        $user = $this->store->findOneAs(User::class, 'users', ['id' => ['$eq' => 1]]);
        $this->assertInstanceOf(User::class, $user);
        $this->assertSame(1, $user->id);
        $this->assertSame('Alice', $user->name);
    }

    public function test_findOneAs_returns_null_when_not_found(): void
    {
        $this->store->set('users', []);
        $result = $this->store->findOneAs(User::class, 'users', ['id' => ['$eq' => 99]]);
        $this->assertNull($result);
    }

    public function test_findInAs_returns_array_of_typed_objects(): void
    {
        $this->store->set('users', [
            ['id' => 1, 'name' => 'Alice', 'active' => true],
            ['id' => 2, 'name' => 'Bob',   'active' => false],
            ['id' => 3, 'name' => 'Carol', 'active' => true],
        ]);

        $users = $this->store->findInAs(User::class, 'users', ['active' => ['$eq' => true]]);
        $this->assertCount(2, $users);
        $this->assertContainsOnlyInstancesOf(User::class, $users);
    }

    public function test_setObject_stores_as_array(): void
    {
        $user = new User();
        $user->id = 42;
        $user->name = 'Dave';

        $this->store->setObject('profile', $user);
        $stored = $this->store->get('profile');
        $this->assertSame(42, $stored['id']);
        $this->assertSame('Dave', $stored['name']);
    }

    public function test_pushObject_appends_to_array(): void
    {
        $this->store->set('users', []);
        $user = new User();
        $user->id = 1;
        $user->name = 'Eve';

        $this->store->pushObject('users', $user);
        $this->assertSame(1, $this->store->count('users'));
    }

    public function test_streamAs_returns_typed_objects(): void
    {
        $users = array_map(fn($i) => ['id' => $i, 'name' => "User_{$i}", 'active' => true], range(1, 50));
        $this->store->set('users', $users);

        $result = $this->store->streamAs(User::class, '/users', ['active' => ['$eq' => true]]);
        $this->assertCount(50, $result);
        $this->assertContainsOnlyInstancesOf(User::class, $result);
    }
}
