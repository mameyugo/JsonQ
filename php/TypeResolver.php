<?php
declare(strict_types=1);

namespace JsonQ;

use ReflectionNamedType;
use ReflectionProperty;
use JsonQ\Attribute\Type;

/**
 * @internal
 */
class TypeResolver
{
    public static function resolvePropertyType(ReflectionProperty $property): ?string
    {
        $type = $property->getType();
        
        if ($type instanceof ReflectionNamedType) {
            return $type->getName();
        }

        return null;
    }

    public static function isNullable(ReflectionProperty $property): bool
    {
        $type = $property->getType();
        return $type ? $type->allowsNull() : true;
    }

    public static function getArrayItemType(ReflectionProperty $property): ?string
    {
        $attributes = $property->getAttributes(Type::class);
        if (empty($attributes)) {
            return null;
        }

        $typeAttr = $attributes[0]->newInstance();
        $typeString = $typeAttr->type;

        if (preg_match('/^array<(.+)>$/', $typeString, $matches)) {
            return $matches[1];
        }

        return null;
    }
}
