# Informe — ¿Quién protege la memoria?

- **Usuario:** `ldlavado`
- **Fecha:** `2026-11-04`
- **Versiones:** ver `EVIDENCIAS/versions.txt` (salida de `make versions` adaptado; entorno local sin Docker/clang completo)

> Regla: complete la columna **Predicción** y haga commit **antes** de ejecutar.
> Para cada resultado indique el comando exacto y el archivo de `EVIDENCIAS/`.

## 1. Experimento 1 — Límites y comportamiento indefinido

| Caso | Predicción | ¿Compila? | ¿Cuándo se detecta? | Resultado observado | ¿Qué garantía aporta? |
| --- | --- | --- | --- | --- | --- |
| C, índice calculado (a/b/c) | (a/b) UB: puede imprimir basura o crash; (c) ASan detecta stack-buffer-overflow | Sí | (a/b) no o en ejecución impredecible; (c) ejecución con sanitizer | (pendiente) | Ninguna en C puro; ASan es herramienta dinámica |
| Rust, constante | Error de compilación por lint unconditional_panic o panic en ejecución | No (o sí y panic) | Compilación o ejecución | (pendiente) | Comprobación estática de índices constantes |
| Rust, calculado (`[]` y `get`) | `get` retorna None; `[]` hace panic en ejecución | Sí | Ejecución (bounds check dinámico) | (pendiente) | Bounds check en runtime para índices dinámicos |
| Rust, `get_unchecked` (normal y Miri) | Debug puede abortar; release UB; Miri reporta Undefined Behavior | Sí | Miri (estático/dinámico interpretado) | (pendiente) | Ninguna si se usa unsafe; Miri detecta UB |
| Java, arreglo de 5 | ArrayIndexOutOfBoundsException | Sí | Ejecución | (pendiente) | Bounds check dinámico + excepción |
| Overflow C / Rust / Java | C: UB (puede optimizar); Rust debug panic, release wrap; Java wrap + addExact excepción | Sí | Depende | (pendiente) | C: ninguna (UB); Rust: debug checks; Java: definida + opcional checked |

**Comandos y evidencias:**

**Análisis.** ¿Qué mecanismo detecta el índice constante en Rust y por qué no
prueba que todo índice sea válido? ¿Por qué `get_unchecked` es UB y `v[i]` no?
¿Qué semántica de overflow elige cada lenguaje y por qué el compilador de C
puede "optimizar" la comparación?

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

1. ¿Qué diferencia concreta observó entre comportamiento indefinido en C, panic en Rust y excepción en Java? ¿Por qué no son resultados equivalentes?
2. ¿Qué detecta el compilador de Rust en el experimento de ownership que no detecta el de Java, y qué detecta Rust solo en ejecución?
3. ¿Qué garantías ofrece cada modelo y cuáles dependen aún de la disciplina del equipo o de comprobaciones en ejecución?
4. ¿Por qué AddressSanitizer, UBSan, TSan o Miri ayudan a encontrar defectos pero no demuestran que un programa esté libre de ellos?
5. ¿Cuándo es preferible borrowing frente a clonar? Considere semántica, costo y complejidad.
6. ¿Qué diferencia hay entre evitar data races y evitar deadlocks? Ilústrela con el Experimento 3.
7. ¿Qué riesgos permanecen en Rust con unsafe o FFI y cómo mitigarlos en un proyecto real?
8. Si un servicio prioriza latencia predecible y control de recursos, ¿qué factores evaluaría para elegir entre C, Rust y Java, y qué evidencia adicional necesitaría para pasar de este laboratorio a una recomendación de arquitectura? Evite asumir que un lenguaje siempre es más rápido.

## 6. Conclusión ejecutiva (150–250 palabras)

Recomiende una alternativa para un contexto definido y declare los trade-offs.
