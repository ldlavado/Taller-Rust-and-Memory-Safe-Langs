# Informe — ¿Quién protege la memoria?

- **Usuario:** `ldlavado`
- **Fecha:** `2026-11-04`
- **Versiones:** ver `EVIDENCIAS/versions.txt` (salida de `make versions` adaptado; entorno local sin Docker/clang completo)

> Regla: complete la columna **Predicción** y haga commit **antes** de ejecutar.
> Para cada resultado indique el comando exacto y el archivo de `EVIDENCIAS/`.

## 1. Experimento 1 — Límites y comportamiento indefinido

| Caso | Predicción | ¿Compila? | ¿Cuándo se detecta? | Resultado observado | ¿Qué garantía aporta? |
| --- | --- | --- | --- | --- | --- |
| C, índice calculado (a/b/c) | (a/b) UB: basura o crash; (c) ASan stack-buffer-overflow | Sí | (a/b) impredecible; (c) ASan en ejecución | ASan falló por permisos (código 126); gcc -O0/-O2 ejecutaron (UB no estable) | Ninguna en C; ASan dinámica |
| Rust, constante | Error unconditional_panic o panic | No/Sí+panic | Compilación o ejecución | Compiló y panic "index out of bounds" (ver m1.txt) | Lint estático + bounds check |
| Rust, calculado (`[]` y `get`) | get=None; []=panic | Sí | Ejecución | get None; [] panic index out of bounds | Bounds check dinámico |
| Rust, `get_unchecked` (normal y Miri) | Debug abort/UB; Miri UB | Sí | Miri | Debug/release problemas de permisos; Miri reportó UB | Ninguna (unsafe); Miri detecta |
| Java, arreglo de 5 | ArrayIndexOutOfBoundsException | Sí | Ejecución | ArrayIndexOutOfBoundsException: Index 5 out of bounds for length 5 | Bounds check + excepción |
| Overflow C / Rust / Java | C UB; Rust debug panic/release wrap; Java wrap+excepción | Sí | Depende | Rust debug: panic attempt to add with overflow; release: -2147483648; Java: wrap + ArithmeticException | C: UB; Rust: debug checks; Java: definida |

**Comandos y evidencias:**
- `LAB_IN_CONTAINER=1 CARGO_TARGET_DIR=/tmp/cargo-target bash modulos/m1-limites/run-expect-fail.sh` → `EVIDENCIAS/m1.txt`
- `java Overflow.java` y `java Bounds.java` (creado)
- Nota: sin Docker/clang completo; algunos binarios C con sanitizer tuvieron Permission denied. Rust/Java ejecutados correctamente.

**Análisis.**
- El índice constante en Rust lo detecta el lint `unconditional_panic` o el bounds check en compilación/ejecución. No prueba índices dinámicos porque el compilador no puede evaluar todos los valores posibles en tiempo de compilación.
- `get_unchecked` es UB porque omite el bounds check (contrato del programador); `v[i]` siempre hace el check dinámico y panics si falla.
- Overflow: C = UB (permite optimizaciones como plegar `x+1 > x` a true); Rust debug = panic (overflow-checks), release = wrap; Java = wrap definido + `Math.addExact` lanza excepción. El compilador de C puede optimizar porque asume que no hay UB.

## 2. Ownership y dónde termina la garantía

### Puzzle A — move

- Mensaje del compilador y propiedad violada: E0382 (use of moved value); ownership se mueve.
- Corrección con borrowing (código) y justificación: usar referencia `&s` para no mover.
- Corrección con clonación (código) y justificación: `.clone()` para tener dos owners independientes.
- Comparación de semántica y costo: borrowing cero costo, clone copia datos.
- ¿Qué habría pasado en C y en Java? C: double free o dangling; Java: referencias compartidas sin problema de ownership.

### Puzzle B — mutar mientras se itera

- Mensaje del compilador y propiedad violada: E0502 (borrow conflict); mutable borrow mientras immutable.
- Corrección mínima (código) y justificación: recolectar índices o usar iterador que permita mutación controlada.
- Java: `ConcurrentModificationException`: detecta en ejecución (fail-fast iterator), no en compilación.

### RefCell

- Predicción y resultado: Compila, pero panic en ejecución si se viola borrow rules (already borrowed).
- Qué comprobó el compilador / qué comprueba RefCell en ejecución: Compilador no ve interior mutability; RefCell hace dynamic borrow check.
- Por qué el panic es preferible a un acceso indefinido, pero no equivale a una garantía estática: Evita UB, pero es runtime.
- Fugas: `Rc` cíclico en Rust y caché estático en Java: Ambos retienen memoria; GC de Java no libera static cache; Rc necesita Weak para ciclos.

## 3. Concurrencia

| Programa | Clasificación (data race / deadlock / error lógico / ninguno) | ¿Qué lo detecta? | Evidencia |
| --- | --- | --- | --- |
| Java `Carrera` | error lógico (data race a nivel de lenguaje no, pero resultado incorrecto) | nada (o inspección) | dispersión de valores |
| C `carrera.c` (TSan) | data race (UB) | TSan | reporte TSan |
| Rust `Rc` entre hilos | no compila (no Send) | compilador | E0277 |
| Rust dos `Mutex` | deadlock | nada (timeout) | no termina |

- Dispersión observada de `Carrera.java` y corrección con `AtomicInteger`:
- Por qué la data race es UB en C y no en Java: C no tiene modelo de memoria definido para races; Java sí (aunque resultado incorrecto).
- Corrección del deadlock con orden global de adquisición (código): adquirir siempre en el mismo orden (ej. lock1 luego lock2).
- Discusión: contador con `Relaxed`; memory safety vs. thread safety vs. correctness: Rust garantiza memory safety y thread safety (no data races) pero no correctness ni ausencia de deadlocks.

## 4. Reto de ingeniería

**Ruta elegida:** D (minivec soundness)

Código de la solución (o referencia al commit/archivo): ver minivec/ después de corrección.

1. Propiedad deseada: API segura no debe permitir UB desde código seguro.
2. Mecanismo que la impone o comprueba: invariantes en unsafe + Miri.
3. Momento de detección: Miri en test.
4. Costo o limitación: Miri no es prueba exhaustiva.
5. Evidencia: salida Miri antes/después.

Prueba adicional que se necesitaría: pruebas formales o más casos límite.

## 5. Preguntas de reflexión

1. **Diferencia UB / panic / excepción:** En C el UB puede corromper silenciosamente o ser optimizado de formas sorprendentes (resultado no reproducible). En Rust el panic aborta de forma controlada (stack unwind o abort). En Java la excepción es un valor de control de flujo definido. No son equivalentes: UB es "el compilador puede hacer cualquier cosa"; panic/excepción son comportamientos definidos que el programa puede manejar o no.
2. **Ownership Rust vs Java:** Rust rechaza en compilación moves inválidos y borrows conflictivos (E0382, E0502). Java permite aliasing libre y solo detecta ConcurrentModification en ejecución (fail-fast). Rust detecta en ejecución los RefCell borrows y overflow en debug.
3. **Garantías:** Rust ofrece memory safety y thread safety estáticas (sin unsafe). Java ofrece memory safety vía GC + bounds checks, pero no thread safety completa. C ofrece ninguna por defecto. Las restantes dependen de disciplina (no usar unsafe mal) o herramientas (ASan, Miri, TSan).
4. **Sanitizers/Miri:** Son herramientas dinámicas o de interpretación que encuentran defectos en las ejecuciones probadas. No demuestran ausencia de defectos porque no exploran todos los caminos ni todos los inputs.
5. **Borrowing vs clone:** Preferir borrowing cuando la semántica lo permite (cero costo, sin copia). Clone cuando se necesitan dueños independientes o se sale del scope. Trade-off: borrowing puede complicar lifetimes; clone es simple pero costoso.
6. **Data races vs deadlocks:** Rust evita data races en compilación (Send/Sync). Deadlocks son de correctness y no se previenen estáticamente (el ejemplo de dos Mutex en orden opuesto). Java no previene ni data races ni deadlocks; C los trata como UB.
7. **Riesgos unsafe/FFI:** El código unsafe puede introducir UB si se violan invariantes. Mitigación: documentar # Safety, encapsular en API segura, tests con Miri, revisión, minimizar superficie unsafe.
8. **Elección de lenguaje:** Para latencia predecible y control de recursos: evaluar footprint, GC pauses (Java), safety guarantees (Rust), control fino (C). Evidencia necesaria: benchmarks reales, perfiles de memoria, análisis de worst-case latency, no solo microbenchmarks de laboratorio.



## 6. Conclusión ejecutiva (150–250 palabras)

Para un servicio que prioriza latencia predecible y control de recursos (por ejemplo un sistema de trading de baja latencia o un motor de base de datos embebido), recomiendo Rust como alternativa principal. Rust ofrece memory safety y thread safety en tiempo de compilación sin garbage collector, lo que elimina tanto los pausas de GC de Java como la necesidad de disciplina extrema de C para evitar UB. En el laboratorio se observó que el compilador de Rust rechaza moves y borrows inválidos antes de ejecutar, mientras que Java solo detecta ciertos errores en runtime y C permite UB que puede ser optimizado de formas no reproducibles.

Los trade-offs son: curva de aprendizaje más alta (ownership, lifetimes), tiempos de compilación mayores y la necesidad de encapsular cuidadosamente cualquier `unsafe` o FFI (como se demostró con el defecto de soundness en minivec, detectado por Miri). Java sigue siendo preferible cuando el ecosistema y la productividad importan más que la latencia predecible, y C cuando se requiere control absoluto y se acepta el costo de verificación externa (ASan, TSan, formal methods). La evidencia de este taller (panics definidos vs UB, detección estática vs dinámica) apoya que las garantías de Rust reducen la clase de defectos que llegan a producción, siempre que se mantenga la disciplina en los bloques unsafe.

