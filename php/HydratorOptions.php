<?php
declare(strict_types=1);

namespace JsonQ;

class HydratorOptions
{
    /**
     * @param TypeCoercionMode $coercion Default is STRICT.
     * @param string $unknownProperties 'ignore' or 'throw'.
     * @param \Closure|null $keyTransformer Function to transform JSON keys to PHP properties.
     */
    public function __construct(
        public readonly TypeCoercionMode $coercion = TypeCoercionMode::STRICT,
        public readonly string $unknownProperties = 'ignore',
        public readonly ?\Closure $keyTransformer = null
    ) {}

    public static function lenient(): self
    {
        return new self(coercion: TypeCoercionMode::LENIENT);
    }

    public static function withCamelCase(): self
    {
        return new self(
            keyTransformer: fn(string $k) => lcfirst(str_replace('_', '', ucwords($k, '_')))
        );
    }
}
