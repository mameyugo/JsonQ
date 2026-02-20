<?php
declare(strict_types=1);

namespace JsonQ\Tests;

use JsonQ\Hydrator;
use JsonQ\HydratorOptions;
use JsonQ\TypeCoercionMode;
use JsonQ\Exception\HydrationException;
use JsonQ\Exception\TypeMismatchException;
use JsonQ\Tests\Fixtures\User;
use JsonQ\Tests\Fixtures\Address;
use JsonQ\Tests\Fixtures\Tag;
use PHPUnit\Framework\TestCase;

class CamelUser {
    public int $id;
    public string $firstName;
}

class HydratorTest extends TestCase
{
    private Hydrator $hydrator;

    protected function setUp(): void
    {
        $this->hydrator = new Hydrator();
    }

    // ── Hydration básica ──────────────────────────────────────────

    public function test_hydrate_simple_object(): void
    {
        $user = $this->hydrator->hydrate(['id' => 1, 'name' => 'Alice'], User::class);
        $this->assertInstanceOf(User::class, $user);
        $this->assertSame(1, $user->id);
        $this->assertSame('Alice', $user->name);
    }

    public function test_hydrate_preserves_defaults(): void
    {
        $user = $this->hydrator->hydrate(['id' => 1, 'name' => 'Bob'], User::class);
        $this->assertTrue($user->active); // default = true
        $this->assertSame([], $user->tags); // default = []
    }

    public function test_hydrate_null_for_nullable_missing_property(): void
    {
        $user = $this->hydrator->hydrate(['id' => 1, 'name' => 'Carol'], User::class);
        $this->assertNull($user->email); // ?string, no presente → null
    }

    public function test_hydrate_array_returns_typed_objects(): void
    {
        $data = [
            ['id' => 1, 'name' => 'Alice'],
            ['id' => 2, 'name' => 'Bob'],
        ];
        $users = $this->hydrator->hydrateArray($data, User::class);
        $this->assertCount(2, $users);
        $this->assertContainsOnlyInstancesOf(User::class, $users);
        $this->assertSame(2, $users[1]->id);
    }

    public function test_hydrate_empty_array(): void
    {
        $result = $this->hydrator->hydrateArray([], User::class);
        $this->assertSame([], $result);
    }

    // ── Nested objects ────────────────────────────────────────────

    public function test_hydrate_nested_object(): void
    {
        $data = [
            'id'      => 1,
            'name'    => 'Alice',
            'address' => ['street' => '123 Main St', 'city' => 'NYC'],
        ];
        $user = $this->hydrator->hydrate($data, User::class);
        $this->assertInstanceOf(Address::class, $user->address);
        $this->assertSame('NYC', $user->address->city);
    }

    public function test_hydrate_nested_object_with_null_value(): void
    {
        $data = ['id' => 1, 'name' => 'Alice', 'address' => null];
        $user = $this->hydrator->hydrate($data, User::class);
        $this->assertNull($user->address);
    }

    // ── Typed arrays (#[Type]) ────────────────────────────────────

    public function test_hydrate_typed_array_of_objects(): void
    {
        $data = [
            'id'   => 1,
            'name' => 'Alice',
            'tags' => [['id' => 10, 'name' => 'php'], ['id' => 11, 'name' => 'rust']],
        ];
        $user = $this->hydrator->hydrate($data, User::class);
        $this->assertCount(2, $user->tags);
        $this->assertContainsOnlyInstancesOf(Tag::class, $user->tags);
        $this->assertSame('php', $user->tags[0]->name);
    }

    public function test_hydrate_typed_array_empty(): void
    {
        $data = ['id' => 1, 'name' => 'Alice', 'tags' => []];
        $user = $this->hydrator->hydrate($data, User::class);
        $this->assertSame([], $user->tags);
    }

    // ── TypeCoercionMode ──────────────────────────────────────────

    public function test_strict_mode_throws_on_type_mismatch(): void
    {
        $this->expectException(TypeMismatchException::class);
        $options = new HydratorOptions(coercion: TypeCoercionMode::STRICT);
        $hydrator = new Hydrator($options);
        $hydrator->hydrate(['id' => 'not-an-int', 'name' => 'Alice'], User::class);
    }

    public function test_lenient_mode_coerces_string_to_int(): void
    {
        $options = new HydratorOptions(coercion: TypeCoercionMode::LENIENT);
        $hydrator = new Hydrator($options);
        $user = $hydrator->hydrate(['id' => '42', 'name' => 'Alice'], User::class);
        $this->assertSame(42, $user->id);
    }

    public function test_lenient_mode_coerces_int_to_string(): void
    {
        $options = new HydratorOptions(coercion: TypeCoercionMode::LENIENT);
        $hydrator = new Hydrator($options);
        $user = $hydrator->hydrate(['id' => 1, 'name' => 123], User::class);
        $this->assertSame('123', $user->name);
    }

    // ── Key transformer ───────────────────────────────────────────

    public function test_snake_to_camel_key_transformer(): void
    {
        $options = HydratorOptions::withCamelCase();
        $hydrator = new Hydrator($options);

        $data = ['id' => 1, 'first_name' => 'Alice'];
        $user = $hydrator->hydrate($data, CamelUser::class);
        
        $this->assertSame('Alice', $user->firstName);
    }

    // ── Unknown properties ────────────────────────────────────────

    public function test_unknown_properties_are_ignored_by_default(): void
    {
        $data = ['id' => 1, 'name' => 'Alice', 'unknown_field' => 'value'];
        $user = $this->hydrator->hydrate($data, User::class);
        $this->assertSame(1, $user->id); // No lanza excepción
    }

    public function test_unknown_properties_throw_in_strict_mode(): void
    {
        $options = new HydratorOptions(unknownProperties: 'throw');
        $hydrator = new Hydrator($options);
        $this->expectException(HydrationException::class);
        $hydrator->hydrate(['id' => 1, 'name' => 'Alice', 'ghost' => 'field'], User::class);
    }

    // ── Dehydration ───────────────────────────────────────────────

    public function test_dehydrate_simple_object(): void
    {
        $user = new User();
        $user->id = 1;
        $user->name = 'Alice';
        $user->active = true;

        $array = $this->hydrator->dehydrate($user);
        $this->assertIsArray($array);
        $this->assertSame(1, $array['id']);
        $this->assertSame('Alice', $array['name']);
    }

    public function test_dehydrate_with_ignore(): void
    {
        $user = new User();
        $user->id = 1;
        $user->name = 'Alice';

        $array = $this->hydrator->dehydrate($user, ignore: ['name']);
        $this->assertArrayNotHasKey('name', $array);
        $this->assertArrayHasKey('id', $array);
    }

    public function test_dehydrate_nested_object(): void
    {
        $address = new Address();
        $address->street = '123 Main';
        $address->city = 'NYC';

        $user = new User();
        $user->id = 1;
        $user->name = 'Alice';
        $user->address = $address;

        $array = $this->hydrator->dehydrate($user);
        $this->assertIsArray($array['address']);
        $this->assertSame('NYC', $array['address']['city']);
    }

    // ── Round-trip ────────────────────────────────────────────────

    public function test_hydrate_dehydrate_roundtrip(): void
    {
        $original = ['id' => 1, 'name' => 'Alice', 'active' => true, 'tags' => []];
        $user = $this->hydrator->hydrate($original, User::class);
        $back = $this->hydrator->dehydrate($user);

        $this->assertSame($original['id'], $back['id']);
        $this->assertSame($original['name'], $back['name']);
    }
}
