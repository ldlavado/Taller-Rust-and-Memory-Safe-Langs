// Experimento 1 — índice fuera de rango en Java.
// ESPERADO: ArrayIndexOutOfBoundsException
public class Bounds {
    public static void main(String[] args) {
        int[] a = {1, 2, 3, 4, 5};
        System.out.println(a[5]); // índice 5 en arreglo de 5 elementos
    }
}
