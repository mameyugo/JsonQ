<?php
declare(strict_types=1);

namespace JsonQ;

use JsonQ\Exception\HydrationException;
use JsonQ\Exception\TypeMismatchException;
use ReflectionClass;
use ReflectionProperty;

class Hydrator
{
    public function __construct(
        private readonly HydratorOptions $options = new HydratorOptions()
    ) {}

    public function hydrate(array $data, string $class): object
    {
        if (!class_exists($class)) {
            throw new HydrationException("Class {$class} does not exist.");
        }

        $refClass = new ReflectionClass($class);
        $object = $refClass->newInstanceWithoutConstructor();
        
        $propertyMap = [];
        foreach ($refClass->getProperties(ReflectionProperty::IS_PUBLIC) as $prop) {
            $propertyMap[$prop->getName()] = $prop;
        }

        $transformer = $this->options->keyTransformer;

        foreach ($data as $key => $value) {
            $propertyName = $transformer ? $transformer($key) : $key;

            if (!isset($propertyMap[$propertyName])) {
                if ($this->options->unknownProperties === 'throw') {
                    throw new HydrationException("Unknown property: {$key}");
                }
                continue;
            }

            $property = $propertyMap[$propertyName];
            $value = $this->coerceValue($value, $property);
            $property->setValue($object, $value);
            unset($propertyMap[$propertyName]);
        }

        foreach ($propertyMap as $prop) {
            if (!$prop->isInitialized($object)) {
                if (TypeResolver::isNullable($prop)) {
                    $prop->setValue($object, null);
                }
            }
        }

        return $object;
    }

    public function hydrateArray(array $items, string $class): array
    {
        $result = [];
        foreach ($items as $k => $item) {
            if (!is_array($item)) {
                throw new TypeMismatchException("hydrateArray expects an array of arrays. Found " . gettype($item));
            }
            $result[$k] = $this->hydrate($item, $class);
        }
        return $result;
    }

    public function dehydrate(object $object, array $ignore = []): array
    {
        $refClass = new ReflectionClass($object);
        $data = [];

        foreach ($refClass->getProperties(ReflectionProperty::IS_PUBLIC) as $property) {
            $name = $property->getName();
            if (in_array($name, $ignore, true)) {
                continue;
            }

            if (!$property->isInitialized($object)) {
                continue;
            }

            $value = $property->getValue($object);
            $data[$name] = $this->dehydrateValue($value, $ignore);
        }

        return $data;
    }

    public function dehydrateArray(array $items, array $ignore = []): array
    {
        $result = [];
        foreach ($items as $k => $item) {
            if (is_object($item)) {
                $result[$k] = $this->dehydrate($item, $ignore);
            } else {
                $result[$k] = $this->dehydrateValue($item, $ignore);
            }
        }
        return $result;
    }

    private function dehydrateValue(mixed $value, array $ignore = []): mixed
    {
        if (is_object($value)) {
            return $this->dehydrate($value, $ignore);
        }

        if (is_array($value)) {
            $result = [];
            foreach ($value as $k => $v) {
                $result[$k] = $this->dehydrateValue($v, $ignore);
            }
            return $result;
        }

        return $value;
    }

    private function coerceValue(mixed $value, ReflectionProperty $property): mixed
    {
        if ($value === null) {
            if (!TypeResolver::isNullable($property)) {
                throw new HydrationException("Property {$property->getName()} is not nullable.");
            }
            return null;
        }

        $type = TypeResolver::resolvePropertyType($property);

        if ($type === null) {
            return $value;
        }

        if ($type === 'array') {
            if (!is_array($value)) {
                throw new TypeMismatchException("Property {$property->getName()} expects array, got " . gettype($value));
            }
            $itemType = TypeResolver::getArrayItemType($property);
            if ($itemType) {
                $result = [];
                foreach ($value as $k => $v) {
                    if (class_exists($itemType)) {
                        $result[$k] = $this->hydrate($v, $itemType);
                    } else {
                        $result[$k] = $this->coercePrimitive($v, $itemType, $property->getName());
                    }
                }
                return $result;
            }
            return $value;
        }

        if (class_exists($type)) {
            if (!is_array($value)) {
                 throw new TypeMismatchException("Property {$property->getName()} expects object of {$type}, got " . gettype($value));
            }
            return $this->hydrate($value, $type);
        }

        return $this->coercePrimitive($value, $type, $property->getName());
    }

    private function coercePrimitive(mixed $value, string $expectedType, string $propertyName): mixed
    {
        $actualType = gettype($value);
        if ($actualType === 'integer') $actualType = 'int';
        if ($actualType === 'boolean') $actualType = 'bool';
        if ($actualType === 'double') $actualType = 'float';

        if ($actualType === $expectedType) {
            return $value;
        }

        if ($this->options->coercion === TypeCoercionMode::STRICT) {
            throw new TypeMismatchException("Property {$propertyName} expects {$expectedType}, got {$actualType}");
        }

        return match ($expectedType) {
            'int' => (int)$value,
            'float' => (float)$value,
            'string' => (string)$value,
            'bool' => (bool)$value,
            default => $value,
        };
    }
}
