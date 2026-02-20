<?php
declare(strict_types=1);

namespace JsonQ\Store;

use JsonQ\Hydrator;
use JsonQ\HydratorOptions;

class HydratableStore extends \JsonQ\Store
{
    private Hydrator $hydrator;

    public function __construct(string $path, ?HydratorOptions $options = null)
    {
        parent::__construct($path);
        $this->hydrator = new Hydrator($options ?? new HydratorOptions());
    }

    public function findOneAs(string $class, string $path, array $conditions = []): ?object
    {
        $data = $this->findOne($path, $conditions);
        return $data !== null ? $this->hydrator->hydrate($data, $class) : null;
    }

    public function findInAs(string $class, string $path, array $conditions = []): array
    {
        return $this->hydrator->hydrateArray(
            $this->find($path, $conditions),
            $class
        );
    }

    public function streamAs(string $class, string $pointer, array $conditions = [], array $options = []): array
    {
        return $this->hydrator->hydrateArray(
            $this->stream($pointer, $conditions, $options),
            $class
        );
    }

    public function setObject(string $path, object $obj, array $ignore = []): bool
    {
        return $this->set($path, $this->hydrator->dehydrate($obj, $ignore));
    }

    public function pushObject(string $path, object $obj, array $ignore = []): bool
    {
        return $this->push($path, $this->hydrator->dehydrate($obj, $ignore));
    }
}
