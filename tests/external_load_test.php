<?php
use JsonQ\Store;

function assert_eq($expected, $actual, $msg = "") {
    if ($expected !== $actual) {
        echo "  ✗ $msg: Expected " . json_encode($expected) . ", got " . json_encode($actual) . "\n";
        return false;
    }
    echo "  ✓ $msg\n";
    return true;
}

echo "🧪 JsonQ External Data Loading Test\n";
echo "══════════════════════════════════════════════════\n";

$dataFile = __DIR__ . '/sample_data.json';
if (!file_exists($dataFile)) {
    die("Error: Sample data file not found at $dataFile\n");
}

// 1. Initialize store with existing file
$s = new Store($dataFile);

echo "📦 Verifying Loaded Data\n";
assert_eq("TechCorp", $s->get("company"), "Read root key 'company'");
assert_eq("Madrid", $s->get("location"), "Read root key 'location'");
assert_eq(4, $s->count("employees"), "Count loaded array elements");
assert_eq("Juan", $s->get("employees.0.name"), "Read nested array data");
assert_eq("1.0", $s->get("metadata.version"), "Read nested object data");

echo "🔍 Batch Query on Loaded Data\n";
$engineers = $s->find("employees", ["department" => "Engineering"]);
assert_eq(2, count($engineers), "Find engineers in loaded data");
assert_eq("Juan", $engineers[0]["name"], "Verify first engineer name");
assert_eq("Pedro", $engineers[1]["name"], "Verify second engineer name");

echo "══════════════════════════════════════════════════\n";
echo "DONE\n";
