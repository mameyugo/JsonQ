#!/bin/bash
# Fix test files to use Arc::new() for write() calls

cd /var/www/html/JsonQ/tests

# Find all .rs files and replace .write(&json!(...)) with .write(Arc::new(json!(...)))
find . -name '*.rs' -type f | while read file; do
    # Use perl for more precise regex replacement
    perl -i -pe 's/\.write\(&(json!\([^)]+\))\)/\.write(Arc::new($1))/g' "$file"
    # Handle multi-line json! macros
    perl -i -0777 -pe 's/\.write\(&(json!\(\{[^}]+\}\)))/\.write(Arc::new($1))/gs' "$file"
    # Handle variable references like .write(&data) or .write(&test_data)
    perl -i -pe 's/\.write\(&([a-z_]+)\)/\.write(Arc::new($1))/g' "$file"
done

echo "Fixed test files"
