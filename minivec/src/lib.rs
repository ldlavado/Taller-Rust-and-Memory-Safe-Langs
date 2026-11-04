//! `minivec`: un vector de `i32` con memoria gestionada manualmente.
//!
//! Internamente usa `unsafe` (asignación con `std::alloc` y punteros crudos),
//! pero expone una API pública **segura**: `new`, `push`, `get`, `pop` y `len`.
//! Ningún método público es `unsafe fn`, así que la abstracción debe ser
//! *sound*: desde código seguro no debería ser posible provocar UB a través de ella.
//!
//! Invariantes de la estructura:
//! - `cap` es el número de `i32` reservados en `ptr` (0 si no hay reserva).
//! - `len <= cap`.
//! - Las posiciones `0..len` están inicializadas.

use std::alloc::{self, Layout};
use std::ptr::NonNull;

/// Vector de `i32` con crecimiento geométrico.
pub struct MiniVec {
    ptr: NonNull<i32>,
    cap: usize,
    len: usize,
}

impl MiniVec {
    /// Crea un vector vacío sin reservar memoria.
    pub fn new() -> Self {
        MiniVec {
            ptr: NonNull::dangling(),
            cap: 0,
            len: 0,
        }
    }

    /// Número de elementos almacenados.
    pub fn len(&self) -> usize {
        self.len
    }

    /// `true` si no hay elementos.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Agrega `valor` al final, reservando más memoria si hace falta.
    pub fn push(&mut self, valor: i32) {
        if self.len == self.cap {
            self.crecer();
        }
        // SAFETY: len < cap (garantizado por el if anterior o crecer),
        // ptr apunta a memoria válida de cap elementos, y esta posición no está inicializada aún.
        unsafe {
            self.ptr.as_ptr().add(self.len).write(valor);
        }
        self.len += 1;
    }

    /// Devuelve una referencia al elemento en `indice`, o `None` si no existe.
    pub fn get(&self, indice: usize) -> Option<&i32> {
        if indice < self.len {
            // SAFETY: indice < len <= cap, por lo que la posición está inicializada
            // y dentro de la reserva. El puntero es válido.
            unsafe { Some(&*self.ptr.as_ptr().add(indice)) }
        } else {
            None
        }
    }

    /// Quita y devuelve el último elemento, o `None` si está vacío.
    pub fn pop(&mut self) -> Option<i32> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        // SAFETY: acabamos de decrementar len, así que la posición anterior
        // estaba inicializada y es válida. Leemos y la dejamos sin inicializar.
        unsafe { Some(self.ptr.as_ptr().add(self.len).read()) }
    }

    /// Duplica la capacidad (o reserva 4 elementos si estaba vacío).
    fn crecer(&mut self) {
        let nueva_cap = if self.cap == 0 { 4 } else { self.cap * 2 };
        let nuevo_layout = Layout::array::<i32>(nueva_cap).expect("capacidad demasiado grande");
        let nuevo_ptr = if self.cap == 0 {
            // SAFETY: layout es válido (array de i32). alloc puede devolver null.
            unsafe { alloc::alloc(nuevo_layout) }
        } else {
            let viejo_layout = Layout::array::<i32>(self.cap).unwrap();
            // SAFETY: ptr fue asignado con viejo_layout, nuevo size es mayor,
            // layout válido. realloc puede devolver null.
            unsafe {
                alloc::realloc(
                    self.ptr.as_ptr() as *mut u8,
                    viejo_layout,
                    nuevo_layout.size(),
                )
            }
        };
        self.ptr = match NonNull::new(nuevo_ptr as *mut i32) {
            Some(p) => p,
            None => alloc::handle_alloc_error(nuevo_layout),
        };
        self.cap = nueva_cap;
    }
}

impl Default for MiniVec {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for MiniVec {
    fn drop(&mut self) {
        if self.cap > 0 {
            let layout = Layout::array::<i32>(self.cap).unwrap();
            // SAFETY: ptr fue asignado con este layout exacto y cap > 0.
            unsafe {
                alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}
