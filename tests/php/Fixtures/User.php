<?php
declare(strict_types=1);

namespace JsonQ\Tests\Fixtures;

use JsonQ\Attribute\Type;

class User
{
    public int $id;
    public string $name;
    public ?string $email = null;
    public ?Address $address = null;

    #[Type('array<JsonQ\Tests\Fixtures\Tag>')]
    public array $tags = [];

    public bool $active = true;
}
