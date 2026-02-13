# Migración del Módulo Conversion - Guía Paso a Paso

## 📁 Estructura de Archivos

```
jsonq/
├── src/
│   ├── lib.rs                      # Actualizar para usar el módulo
│   └── conversion/
│       ├── mod.rs                  # ✅ Copiar de src/conversion/
│       ├── value_to_zval.rs        # ✅ Copiar de src/conversion/
│       └── zval_to_value.rs        # ✅ Copiar de src/conversion/
│
└── tests/
    └── unit/
        ├── mod.rs                  # ✅ Copiar de tests/unit/
        └── conversion/
            ├── mod.rs              # ✅ Copiar de tests/unit/conversion/
            ├── value_to_zval_tests.rs
            ├── zval_to_value_tests.rs
            └── roundtrip_tests.rs
```

## 🔧 Paso 1: Copiar Archivos del Módulo

```bash
# Crear directorios
mkdir -p src/conversion
mkdir -p tests/unit/conversion

# Copiar archivos del módulo (SIN tests inline)
cp outputs/src/conversion/mod.rs src/conversion/
cp outputs/src/conversion/value_to_zval.rs src/conversion/
cp outputs/src/conversion/zval_to_value.rs src/conversion/

# Copiar archivos de tests
cp outputs/tests/unit/mod.rs tests/unit/
cp outputs/tests/unit/conversion/mod.rs tests/unit/conversion/
cp outputs/tests/unit/conversion/value_to_zval_tests.rs tests/unit/conversion/
cp outputs/tests/unit/conversion/zval_to_value_tests.rs tests/unit/conversion/
cp outputs/tests/unit/conversion/roundtrip_tests.rs tests/unit/conversion/
```

## 🔧 Paso 2: Actualizar src/lib.rs

### Añadir declaración del módulo (al inicio del archivo):

```rust
//! JsonQ - High-performance JSON file storage engine for PHP
#![allow(non_snake_case)]

// ══════════ MODULE DECLARATIONS ══════════
mod conversion;  // ← AÑADIR ESTA LÍNEA

// ══════════ IMPORTS ══════════
use ext_php_rs::prelude::*;
use ext_php_rs::types::{Zval, ArrayKey};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use memmap2::Mmap;

// ← AÑADIR ESTA LÍNEA
use conversion::{value_to_zval, zval_to_value, ht_to_value};
```

### Eliminar las funciones viejas del lib.rs:

Buscar y **ELIMINAR** completamente estas tres funciones:

```rust
// ← ELIMINAR TODO ESTO:
fn value_to_zval(val: &Value) -> Zval {
    // ... toda la implementación ...
}

fn zval_to_value(zval: &Zval) -> Value {
    // ... toda la implementación ...
}

fn ht_to_value(ht: &ext_php_rs::types::ZendHashTable) -> Value {
    // ... toda la implementación ...
}
```

**IMPORTANTE**: El resto de `lib.rs` NO cambia. Las funciones `value_to_zval()` y `zval_to_value()` ya se están usando en todo el código, ahora simplemente vienen del módulo `conversion` en lugar de estar definidas localmente.

## 🔧 Paso 3: Actualizar tests/lib.rs (si existe)

Si tienes un archivo `tests/lib.rs`, añade:

```rust
// tests/lib.rs
mod unit;
```

Si NO existe, créalo con ese contenido.

## ✅ Paso 4: Compilar

```bash
# Compilar el proyecto
cargo build --release

# Si hay errores, verificar:
# 1. Que los archivos estén en las rutas correctas
# 2. Que se hayan eliminado las funciones viejas de lib.rs
# 3. Que se haya añadido el import
```

## 🧪 Paso 5: Ejecutar Tests

### Tests unitarios (Rust):

```bash
# Ejecutar solo tests del módulo conversion
cargo test conversion

# Ejecutar todos los tests unitarios
cargo test --lib

# Con output detallado
cargo test conversion -- --nocapture
```

### Tests de integración (PHP):

```bash
# Ejecutar suite completa de tests PHP
php -d "extension=$(pwd)/target/release/libjsonq.so" tests/run_tests.php

# Debería mostrar:
# ✅ All tests passed
```

## 📝 Cambios Realizados - Resumen

### Archivos Añadidos:
- ✅ `src/conversion/mod.rs` (módulo principal)
- ✅ `src/conversion/value_to_zval.rs` (conversión Rust→PHP)
- ✅ `src/conversion/zval_to_value.rs` (conversión PHP→Rust)
- ✅ `tests/unit/mod.rs` (raíz de tests unitarios)
- ✅ `tests/unit/conversion/mod.rs` (módulo de tests)
- ✅ `tests/unit/conversion/value_to_zval_tests.rs` (26 tests)
- ✅ `tests/unit/conversion/zval_to_value_tests.rs` (documentación)
- ✅ `tests/unit/conversion/roundtrip_tests.rs` (20 tests)

### Archivos Modificados:
- ✅ `src/lib.rs` (añadir módulo, eliminar funciones viejas)
- ✅ `tests/lib.rs` (crear si no existe)

### Archivos NO Modificados:
- ✅ Todo el resto del código sigue igual
- ✅ Los tests PHP existentes siguen funcionando
- ✅ La API pública no cambia

## 🎯 Verificación Final

✅ **Compilación exitosa**: `cargo build --release`
✅ **Tests unitarios pasan**: `cargo test conversion`
✅ **Tests PHP pasan**: Todos los tests de `tests/run_tests.php`
✅ **No warnings de compilación**
✅ **Extension carga correctamente**: `php -m | grep jsonq`

## 🔍 Troubleshooting

### Error: "cannot find value `value_to_zval` in this scope"

**Causa**: No se agregó el `use conversion::...`

**Solución**: Añadir al inicio de lib.rs:
```rust
use conversion::{value_to_zval, zval_to_value, ht_to_value};
```

### Error: "duplicate definitions of `value_to_zval`"

**Causa**: No se eliminaron las funciones viejas de lib.rs

**Solución**: Eliminar completamente las funciones `value_to_zval`, `zval_to_value` y `ht_to_value` del archivo lib.rs

### Error: "unresolved import `conversion`"

**Causa**: No se declaró el módulo

**Solución**: Añadir al inicio de lib.rs:
```rust
mod conversion;
```

### Tests fallan con "module not found"

**Causa**: Falta `tests/lib.rs` o no declara el módulo

**Solución**: Crear/actualizar `tests/lib.rs`:
```rust
mod unit;
```

## 📊 Resultados Esperados

Después de la migración:

```bash
$ cargo test conversion
   Compiling jsonq v0.1.0
    Finished test [unoptimized + debuginfo] target(s)
     Running unittests src/lib.rs

running 46 tests
test conversion::value_to_zval_tests::test_null_conversion ... ok
test conversion::value_to_zval_tests::test_bool_true ... ok
test conversion::value_to_zval_tests::test_integer_positive ... ok
...
test conversion::roundtrip_tests::roundtrip_complex_structure ... ok

test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured
```

## 🎉 ¡Listo!

La migración está completa. El módulo `conversion` ahora está:
- ✅ Separado del código monolítico
- ✅ Bien organizado y documentado
- ✅ Con tests aislados en `tests/unit/`
- ✅ Funcionando exactamente igual que antes

**Próximos pasos**: Seguir con los módulos `path`, `utils`, y `store` según el REFACTORING_GUIDE.md
