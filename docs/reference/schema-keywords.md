# JSON Schema Keywords Reference

Complete reference for JSON Schema validation keywords supported by JsonQ.

## Overview

JsonQ supports a subset of JSON Schema for validating data. This ensures data integrity and helps catch errors early in your application.

## Type Keywords

### `type`

Specifies the expected data type.

**Supported types:** `string`, `integer`, `number`, `boolean`, `array`, `object`, `null`, `any`

```php
$schema = [
    'type' => 'object',
    'properties' => [
        'name' => ['type' => 'string'],
        'age' => ['type' => 'integer'],
        'score' => ['type' => 'number'],
        'active' => ['type' => 'boolean'],
        'tags' => ['type' => 'array'],
        'metadata' => ['type' => 'object']
    ]
];
```

### `nullable`

Allows null values in addition to the specified type.

```php
$schema = [
    'type' => 'string',
    'nullable' => true  // Accepts string or null
];
```

---

## String Keywords

### `minLength` / `maxLength`

Constrains the length of string values.

```php
$schema = [
    'type' => 'object',
    'properties' => [
        'username' => [
            'type' => 'string',
            'minLength' => 3,
            'maxLength' => 20
        ],
        'password' => [
            'type' => 'string',
            'minLength' => 8,
            'maxLength' => 128
        ]
    ]
];
```

### `format`

Validates string format using predefined patterns.

**Supported formats:**
- `email` - Email address
- `url` / `uri` - URL/URI
- `ipv4` - IPv4 address
- `date` - ISO 8601 date
- `uuid` - UUID string

```php
$schema = [
    'type' => 'object',
    'properties' => [
        'email' => [
            'type' => 'string',
            'format' => 'email'
        ],
        'website' => [
            'type' => 'string',
            'format' => 'url'
        ],
        'ip' => [
            'type' => 'string',
            'format' => 'ipv4'
        ],
        'createdAt' => [
            'type' => 'string',
            'format' => 'date'
        ],
        'id' => [
            'type' => 'string',
            'format' => 'uuid'
        ]
    ]
];
```

### `pattern`

Validates strings against a regular expression.

```php
$schema = [
    'type' => 'object',
    'properties' => [
        'phone' => [
            'type' => 'string',
            'pattern' => '^\+1-\d{3}-\d{3}-\d{4}$'
        ],
        'zipCode' => [
            'type' => 'string',
            'pattern' => '^\d{5}(-\d{4})?$'
        ]
    ]
];
```

---

## Number Keywords

### `minimum` / `maximum`

Constrains numeric values to a range.

```php
$schema = [
    'type' => 'object',
    'properties' => [
        'age' => [
            'type' => 'integer',
            'minimum' => 0,
            'maximum' => 150
        ],
        'rating' => [
            'type' => 'number',
            'minimum' => 0.0,
            'maximum' => 5.0
        ],
        'price' => [
            'type' => 'number',
            'minimum' => 0.01
        ]
    ]
];
```

### `multipleOf`

Requires the number to be a multiple of the specified value.

```php
$schema = [
    'type' => 'object',
    'properties' => [
        'quantity' => [
            'type' => 'integer',
            'multipleOf' => 5  // Must be 5, 10, 15, etc.
        ],
        'price' => [
            'type' => 'number',
            'multipleOf' => 0.25  // Must be 0.25, 0.50, 0.75, etc.
        ]
    ]
];
```

---

## Array Keywords

### `minItems` / `maxItems`

Constrains the number of elements in an array.

```php
$schema = [
    'type' => 'object',
    'properties' => [
        'tags' => [
            'type' => 'array',
            'minItems' => 1,
            'maxItems' => 10
        ],
        'members' => [
            'type' => 'array',
            'minItems' => 2  // At least 2 members required
        ]
    ]
];
```

### `uniqueItems`

Requires all elements in the array to be unique.

```php
$schema = [
    'type' => 'object',
    'properties' => [
        'tags' => [
            'type' => 'array',
            'uniqueItems' => true
        ],
        'emails' => [
            'type' => 'array',
            'uniqueItems' => true
        ]
    ]
];
```

### `items`

Defines the schema for array elements.

```php
// All items must match this schema
$schema = [
    'type' => 'object',
    'properties' => [
        'scores' => [
            'type' => 'array',
            'items' => [
                'type' => 'integer',
                'minimum' => 0,
                'maximum' => 100
            ]
        ],
        'users' => [
            'type' => 'array',
            'items' => [
                'type' => 'object',
                'required' => ['name', 'email'],
                'properties' => [
                    'name' => ['type' => 'string'],
                    'email' => ['type' => 'string', 'format' => 'email']
                ]
            ]
        ]
    ]
];
```

---

## Object Keywords

### `required`

Specifies which properties must be present.

```php
$schema = [
    'type' => 'object',
    'required' => ['name', 'email', 'age'],
    'properties' => [
        'name' => ['type' => 'string'],
        'email' => ['type' => 'string', 'format' => 'email'],
        'age' => ['type' => 'integer'],
        'phone' => ['type' => 'string']  // Optional
    ]
];
```

### `properties`

Defines the schema for each object property.

```php
$schema = [
    'type' => 'object',
    'properties' => [
        'profile' => [
            'type' => 'object',
            'properties' => [
                'firstName' => ['type' => 'string', 'minLength' => 1],
                'lastName' => ['type' => 'string', 'minLength' => 1],
                'bio' => ['type' => 'string', 'maxLength' => 500]
            ]
        ]
    ]
];
```

### `additionalProperties`

Controls whether properties not defined in the schema are allowed.

```php
// Strict schema - no extra properties allowed
$schema = [
    'type' => 'object',
    'properties' => [
        'name' => ['type' => 'string'],
        'age' => ['type' => 'integer']
    ],
    'additionalProperties' => false
];

// This would fail validation:
// ['name' => 'Alice', 'age' => 30, 'extra' => 'not allowed']
```

---

## Enumeration

### `enum`

Restricts values to a fixed set of allowed values.

```php
$schema = [
    'type' => 'object',
    'properties' => [
        'role' => [
            'type' => 'string',
            'enum' => ['admin', 'user', 'guest']
        ],
        'status' => [
            'type' => 'string',
            'enum' => ['active', 'inactive', 'pending', 'banned']
        ],
        'priority' => [
            'type' => 'integer',
            'enum' => [1, 2, 3, 4, 5]
        ]
    ]
];
```

---

## Conditional Keywords

### `if` / `then` / `else`

Applies different schemas based on conditions.

```php
$schema = [
    'type' => 'object',
    'properties' => [
        'country' => ['type' => 'string'],
        'postalCode' => ['type' => 'string']
    ],
    'if' => [
        'properties' => ['country' => ['const' => 'US']]
    ],
    'then' => [
        'properties' => [
            'postalCode' => [
                'type' => 'string',
                'pattern' => '^\d{5}(-\d{4})?$'
            ]
        ]
    ],
    'else' => [
        'properties' => [
            'postalCode' => ['type' => 'string']
        ]
    ]
];
```

### `oneOf`

Value must match exactly one of the schemas.

```php
$schema = [
    'oneOf' => [
        [
            'type' => 'object',
            'properties' => [
                'type' => ['const' => 'personal'],
                'firstName' => ['type' => 'string'],
                'lastName' => ['type' => 'string']
            ],
            'required' => ['type', 'firstName', 'lastName']
        ],
        [
            'type' => 'object',
            'properties' => [
                'type' => ['const' => 'business'],
                'companyName' => ['type' => 'string'],
                'taxId' => ['type' => 'string']
            ],
            'required' => ['type', 'companyName', 'taxId']
        ]
    ]
];
```

### `anyOf`

Value must match at least one of the schemas.

```php
$schema = [
    'anyOf' => [
        ['type' => 'string', 'minLength' => 5],
        ['type' => 'number', 'minimum' => 0]
    ]
];
// Accepts: "hello" (string ≥5 chars) or 42 (number ≥0)
```

---

## Complete Examples

### User Schema

```php
$userSchema = [
    'type' => 'object',
    'required' => ['name', 'email', 'age'],
    'properties' => [
        'name' => [
            'type' => 'string',
            'minLength' => 2,
            'maxLength' => 50
        ],
        'email' => [
            'type' => 'string',
            'format' => 'email'
        ],
        'age' => [
            'type' => 'integer',
            'minimum' => 18,
            'maximum' => 120
        ],
        'role' => [
            'type' => 'string',
            'enum' => ['admin', 'user', 'guest'],
            'default' => 'user'
        ],
        'phone' => [
            'type' => 'string',
            'pattern' => '^\+?[1-9]\d{1,14}$'
        ],
        'tags' => [
            'type' => 'array',
            'items' => ['type' => 'string'],
            'minItems' => 0,
            'maxItems' => 10,
            'uniqueItems' => true
        ]
    ],
    'additionalProperties' => false
];

// Validate
$user = [
    'name' => 'Alice Johnson',
    'email' => 'alice@example.com',
    'age' => 30,
    'role' => 'admin',
    'tags' => ['developer', 'team-lead']
];

$result = $store->validate($user, $userSchema);
```

### Product Schema

```php
$productSchema = [
    'type' => 'object',
    'required' => ['name', 'price', 'category'],
    'properties' => [
        'name' => [
            'type' => 'string',
            'minLength' => 3,
            'maxLength' => 100
        ],
        'description' => [
            'type' => 'string',
            'maxLength' => 1000
        ],
        'price' => [
            'type' => 'number',
            'minimum' => 0.01,
            'multipleOf' => 0.01
        ],
        'category' => [
            'type' => 'string',
            'enum' => ['electronics', 'books', 'clothing', 'food']
        ],
        'inStock' => [
            'type' => 'boolean'
        ],
        'quantity' => [
            'type' => 'integer',
            'minimum' => 0
        ],
        'tags' => [
            'type' => 'array',
            'items' => ['type' => 'string'],
            'uniqueItems' => true
        ],
        'rating' => [
            'type' => 'number',
            'minimum' => 0,
            'maximum' => 5
        ]
    ]
];
```

---

## Validation Methods

### Validate Single Document

```php
$result = $store->validate('user', $userSchema);

if ($result['valid']) {
    echo "✅ Valid!\n";
} else {
    echo "❌ Validation failed:\n";
    foreach ($result['errors'] as $error) {
        echo "  - {$error['path']}: {$error['error']}\n";
    }
}
```

### Validate Collection

```php
$result = $store->validateCollection('users', $userSchema);

echo "Total: {$result['total_items']}\n";
echo "Valid: {$result['valid_items']}\n";
echo "Invalid: {$result['invalid_items']}\n";

if (!$result['valid']) {
    foreach ($result['details'] as $detail) {
        echo "Item {$detail['index']} errors:\n";
        foreach ($detail['errors'] as $error) {
            echo "  - {$error['error']}\n";
        }
    }
}
```

---

## Best Practices

1. **Start Simple**: Begin with basic type and required field validation, then add constraints

2. **Use Format Validators**: Leverage built-in formats (email, url, uuid) instead of custom patterns

3. **Set Reasonable Limits**: Use minLength/maxLength to prevent abuse and ensure data quality

4. **Document Your Schema**: Add comments explaining validation rules for complex schemas

5. **Test Edge Cases**: Validate against empty strings, null values, and boundary conditions

6. **Fail Fast**: Validate data at the entry point before persisting

7. **Provide Clear Error Messages**: Use validation results to give users specific feedback

---

## Error Response Format

```php
[
    'valid' => false,
    'error_count' => 2,
    'errors' => [
        [
            'path' => 'user.email',
            'error' => "Invalid format: 'email'",
            'code' => 'FORMAT_INVALID'
        ],
        [
            'path' => 'user.age',
            'error' => 'Value must be >= 18',
            'code' => 'MINIMUM_VIOLATION'
        ]
    ]
]
```

---

## See Also

- [Schema Validation Guide](../guides/schema-validation.md) - Complete validation guide
- [API Reference](../api/store-class.md) - `validate()` and `validateCollection()` methods
- [JSON Schema Specification](https://json-schema.org/) - Official JSON Schema docs
