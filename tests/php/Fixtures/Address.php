<?php
declare(strict_types=1);

namespace JsonQ\Tests\Fixtures;

class Address
{
    public string $street;
    public string $city;
    public ?string $zip = null;
}
