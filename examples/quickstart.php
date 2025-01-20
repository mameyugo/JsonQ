use Rjson\Store;

$store = new Store('data.json');
$store->set('users', [
    ['name' => 'Alice', 'age' => 30],
    ['name' => 'Bob',   'age' => 25],
]);

$admins = $store->find('users', ['name' => 'Alice']);
var_dump($admins);
