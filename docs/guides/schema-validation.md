# Schema Validation

JsonQ includes built-in support for JSON Schema validation, allowing you to enforce data integrity at the application level without external libraries.

## Validation Basics

The validation engine supports a subset of the JSON Schema specification (Draft 7).

### Validating a Document

Use the `validate($path, $schema)` method to check a value at a specific path against a schema.

```php
$schema = [
    'type' => 'object',
    'required' => ['name', 'email'],
    'properties' => [
        'name' => ['type' => 'string'],
        'email' => ['type' => 'string', 'format' => 'email'],
        'age' => ['type' => 'integer', 'minimum' => 18]
    ]
];

// Validate the 'user' object in the store
$result = $store->validate('user', $schema);

if ($result['valid']) {
    echo "Valid user!";
} else {
    print_r($result['errors']);
}
```

### Validating a Collection

Use `validateCollection($collectionPath, $itemSchema)` to validate every item in an array.

```php
$itemSchema = [
    'type' => 'object',
    'properties' => [
        'sku' => ['type' => 'string'],
        'price' => ['type' => 'number', 'minimum' => 0]
    ]
];

$result = $store->validateCollection('products', $itemSchema);

echo "Valid items: " . $result['valid_items'] . "\n";
echo "Invalid items: " . $result['invalid_items'] . "\n";
```

## Supported Keywords

### Common
- `type`: `string`, `number`, `integer`, `boolean`, `array`, `object`, `null`
- `enum`: List of allowed values
- `const`: Exact value match

### Strings
- `minLength`, `maxLength`
- `pattern` (Regex)
- `format`: `email`, `uri`, `ipv4`, `ipv6`, `date`, `date-time`, `uuid`

### Numbers
- `minimum`, `maximum`
- `exclusiveMinimum`, `exclusiveMaximum`
- `multipleOf`

### Arrays
- `items`: Schema for array items
- `minItems`, `maxItems`
- `uniqueItems`: `true`/`false`

### Objects
- `properties`: Map of property schemas
- `required`: List of required property names
- `additionalProperties`: Schema or `false`

### Conditional Logic
- `if`, `then`, `else`
- `allOf`, `anyOf`, `oneOf`, `not`

## Example: Conditional Validation

Validate that if a user is an "admin", they must have a "permissions" array.

```php
$schema = [
    'type' => 'object',
    'if' => [
        'properties' => ['role' => ['const' => 'admin']]
    ],
    'then' => [
        'required' => ['permissions']
    ]
];

$store->validate('user', $schema);
```
