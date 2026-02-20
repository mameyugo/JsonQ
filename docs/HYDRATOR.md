# Object Hydration

JsonQ 0.5.0 introduces **Object Hydration**, allowing you to automatically map JSON file data directly into strongly-typed PHP 8.1+ objects. It lives within the PHP companion files and does not require compiling the native extension to use the pure `Hydrator`.

## Basic Usage

The `JsonQ\Store\HydratableStore` is a wrapper around the native `JsonQ\Store` that automatically hydrates results.

```php
use JsonQ\Store\HydratableStore;

class User {
    public int $id;
    public string $name;
    public bool $active = true;
}

$store = new HydratableStore('data.json');

// findOneAs returns a single typed object or null
$user = $store->findOneAs(User::class, 'users', ['name' => 'Alice']);
echo $user->name; // "Alice"

// findInAs returns an array of typed objects
$activeUsers = $store->findInAs(User::class, 'users', ['active' => true]);

// Stream thousands of objects efficiently
$stream = $store->streamAs(User::class, '/users');
```

## Writing Objects

The `HydratableStore` also dehydrates objects back into arrays for storage:

```php
$user = new User();
$user->id = 42;
$user->name = 'Bob';

// Set a single object
$store->setObject('profile', $user);

// Append an object to an array
$store->pushObject('users', $user);
```

## Working with Types

### Nested Objects

The Hydrator automatically resolves nested classes.

```php
class Address {
    public string $city;
}

class User {
    public string $name;
    public ?Address $address = null; // Hydrates Address object if present in JSON
}
```

### Typed Arrays

PHP doesn't have native typed arrays (like `Address[]`). Use the `#[Type]` attribute.

```php
use JsonQ\Attribute\Type;

class Tag {
    public string $name;
}

class Post {
    public string $title;

    #[Type('array<JsonQ\Tests\Fixtures\Tag>')]
    public array $tags = [];
}
```

## Hydrator Options

You can customize the hydrator behavior by passing `HydratorOptions` to `HydratableStore`.

```php
use JsonQ\HydratorOptions;
use JsonQ\TypeCoercionMode;

$options = new HydratorOptions(
    coercion: TypeCoercionMode::LENIENT, // Allow string "1" to become int 1
    unknownProperties: 'throw',          // Throw exception if JSON has unknown fields
    keyTransformer: HydratorOptions::withCamelCase()->keyTransformer // Map JSON snake_case to PHP camelCase
);

$store = new HydratableStore('data.json', $options);
```

## Pure PHP Hydrator

If you just want to use the Hydrator without the storage engine:

```php
use JsonQ\Hydrator;

$hydrator = new Hydrator();

$jsonArray = ['id' => 1, 'name' => 'Carol'];
$user = $hydrator->hydrate($jsonArray, User::class);

$backToJson = $hydrator->dehydrate($user);
```
