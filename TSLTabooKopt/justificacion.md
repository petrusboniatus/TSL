# Justificación:
1. Inicialización greedy: va cogiendo el camino más corto, sirve para coger
una mejor solución inicial.
2. Cambio de operador: el operador definido por la tupla (i,j) significa, además
de la inversión de los nodos i e j, la reversión del resto de los nodos entre
i e j. Esto genera unos venimos más "coherentes" y permite un mayor cambio.
3. Se crean una estrategia de reinicialización por diversificación. En la que
se generan vecinos mucho más lejanos (cambiando aleatoriamente la mitad del
camino) y se coger el mejor de los generados aleatoriamente ponderando:
 - La distancia del vecino
 - Lo repetidos que están los nodos en la matriz de frecuencias.

La formula final de distancia es siguiente:
```
let freq_cost = self.freq_mat.get_solution_freq_cost(&new_vec);
let final_cost = self.calculate_cost(&new_vec) + freq_cost * delta_cost
                  * REPETITION_CONST;
```
Donde la función get_solution_freq_cost calcula el costo de todos los nodos de
la matriz de frecuencias de la siguiente forma: Suma los costes individuales
de cada nodo de la solución, que a su vez se calcula como el cociente entre
la frecuencia del nodo y la frecuencia máxima.

Esta estrategia permite que se visiten nodos que nunca se han visitado y a la
vez encontrar soluciones lo suficientemente buenas como para tener potencial
de mejorar la actual antes del siguiente reinicio
