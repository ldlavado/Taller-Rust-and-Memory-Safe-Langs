# Informe — ¿Quién protege la memoria?

- **Usuario:** `su-usuario`
- **Fecha:** `AAAA-MM-DD`
- **Versiones:** ver `EVIDENCIAS/versions.txt` (salida de `make versions`)

> Regla: complete la columna **Predicción** y haga commit **antes** de ejecutar.
> Para cada resultado indique el comando exacto y el archivo de `EVIDENCIAS/`.

## 1. Experimento 1 — Límites y comportamiento indefinido

| Caso | Predicción | ¿Compila? | ¿Cuándo se detecta? | Resultado observado | ¿Qué garantía aporta? |
| --- | --- | --- | --- | --- | --- |
| C, índice calculado (a/b/c) | | | | | |
| Rust, constante | | | | | |
| Rust, calculado (`[]` y `get`) | | | | | |
| Rust, `get_unchecked` (normal y Miri) | | | | | |
| Java, arreglo de 5 | | | | | |
| Overflow C / Rust / Java | | | | | |

**Comandos y evidencias:**

**Análisis.** ¿Qué mecanismo detecta el índice constante en Rust y por qué no
prueba que todo índice sea válido? ¿Por qué `get_unchecked` es UB y `v[i]` no?
¿Qué semántica de overflow elige cada lenguaje y por qué el compilador de C
puede "optimizar" la comparación?

## 2. Ownership y dónde termina la garantía

### Puzzle A — move

- Mensaje del compilador y propiedad violada:
- Corrección con borrowing (código) y justificación:
- Corrección con clonación (código) y justificación:
- Comparación de semántica y costo:
- ¿Qué habría pasado en C y en Java?

### Puzzle B — mutar mientras se itera

- Mensaje del compilador y propiedad violada:
- Corrección mínima (código) y justificación:
- Java: `ConcurrentModificationException`: ¿qué detecta y cuándo, frente a Rust?

### RefCell

- Predicción y resultado:
- Qué comprobó el compilador / qué comprueba RefCell en ejecución:
- Por qué el panic es preferible a un acceso indefinido, pero no equivale a una garantía estática:
- Fugas: `Rc` cíclico en Rust y caché estático en Java:

## 3. Concurrencia

| Programa | Clasificación (data race / deadlock / error lógico / ninguno) | ¿Qué lo detecta? | Evidencia |
| --- | --- | --- | --- |
| Java `Carrera` | | | |
| C `carrera.c` (TSan) | | | |
| Rust `Rc` entre hilos | | | |
| Rust dos `Mutex` | | | |

- Dispersión observada de `Carrera.java` y corrección con `AtomicInteger`:
- Por qué la data race es UB en C y no en Java:
- Corrección del deadlock con orden global de adquisición (código):
- Discusión: contador con `Relaxed`; memory safety vs. thread safety vs. correctness:

## 4. Reto de ingeniería

**Ruta elegida:** A / B / C / D

Código de la solución (o referencia al commit/archivo):

1. Propiedad deseada:
2. Mecanismo que la impone o comprueba:
3. Momento de detección:
4. Costo o limitación:
5. Evidencia:

Prueba adicional que se necesitaría:

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
