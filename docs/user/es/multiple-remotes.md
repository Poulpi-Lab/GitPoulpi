[English](../multiple-remotes.md) · **Español**

# Varios remotos para una rama

Muchos repositorios viven en más de un host: uno principal (`origin`), un espejo en otro host, una copia de seguridad. GitPoulpi permite que una rama haga push a todos ellos con un clic y, si se quiere, que haga pull desde más de uno, sin cambiar lo que hace git a secas. Nada de esto necesita una cuenta, y todo se guarda en el `.git/config` del propio repositorio, así que git a secas y otras herramientas siguen funcionando.

## Destinos de push y fuentes de pull

- El **upstream** es la noción propia de git (`branch.<name>.remote` y `merge`). Es el remoto *principal* de la rama. GitPoulpi nunca cambia cómo funciona.
- Un **destino de push** es un remoto adicional al que la rama también hace push. El botón Push envía la rama al upstream y a cada destino de push.
- Una **fuente de pull** es un remoto adicional del que un pull también toma commits. Está desactivada salvo que la actives (la casilla **Pull también**): hacer merge de dos remotos que no elegiste no es un valor por defecto que nadie quiera.

Un destino puede usar otro nombre de rama en ese remoto (`backup` recibe `feat-x`).

## El diálogo Remotos

Haz clic derecho en una rama y elige **Remotos de <rama>...** (también en el desplegable de Push, en la paleta como `Git: Remotos de la rama...`, y haciendo clic en la insignia `⇉3` de la fila de la rama en la sección Refs). Lista todos los remotos del repositorio:

| Columna | Significado |
|---|---|
| Remoto, Rama | El remoto y el nombre de la rama allí. **Renombrar en el remoto...** cambia ese nombre para la fila seleccionada. |
| Push | Márcala para que el remoto sea un destino de push de esta rama. |
| Pull también | Márcala para que un pull también integre ese remoto. El upstream siempre se integra. |
| Estado | Dónde está la rama respecto a él: sincronizado, por delante, por detrás, nuevo, sin relación. |

Una fila marcada «remoto ausente» nombra un remoto que la rama aún menciona pero que el repositorio ya no tiene; **Quitar** lo olvida. El mismo diálogo define el upstream (la lista **Upstream**) y **Quitar upstream** lo borra. **Aplicar** escribe todo como un único cambio que se puede deshacer.

Un repositorio también puede dar a cada rama nueva los mismos destinos adicionales: **Ajustes > Git y codificación** (la página propia del repositorio), «Destinos de push adicionales para ramas nuevas» (una casilla por remoto). Se copian a una rama cuando obtiene por primera vez un upstream; las ramas existentes no se tocan.

## Hacer push

- **Botón Push** en una rama con destinos adicionales: hace push a todos. Cuando cada remoto simplemente acepta los commits nuevos, se ejecuta al instante y la vista de progreso muestra cada remoto pasando a una marca de verificación. Si algo necesita una decisión (un remoto tiene commits que tú no tienes), se abre en su lugar el diálogo del plan.
- **Push ▾** (la flecha junto al botón; también el propio botón cuando **Botón Push** está en **preguntar** en Ajustes):
  - `Push a todos (origin, gitlab, backup)`
  - `Push solo a <remoto>`, uno por cada remoto de la rama
  - `Push a remotos seleccionados...`: siempre está, para cualquier rama, incluso una con un solo remoto
  - `Remotos de <rama>...`
- El menú contextual de la rama y la paleta (`Git: Push a todos los destinos`, `Git: Push a un remoto...`, `Git: Push a remotos seleccionados...`) ofrecen las mismas acciones con los mismos nombres.

**Push a remotos seleccionados...** abre una lista de todos los remotos del repositorio, marcados con el upstream y los destinos de la rama. Vale solo para este push; marca **Recordar para esta rama** para guardar la elección como destinos de push de la rama.

### El plan

Antes de un push agrupado, GitPoulpi mira cada remoto (una consulta rápida, y una descarga de ese remoto solo si difiere) para que un problema en el tercer remoto se vea antes de que empiece el primer push. El diálogo lista cada remoto con la situación de la rama y lo que hará:

| Situación | Acción por defecto | También se ofrece |
|---|---|---|
| El remoto no tiene nada de la rama, o le faltan algunos commits | Push | Omitir |
| El remoto va por delante (no tienes nada nuevo para él) | Fast-forward desde él | Omitir |
| El remoto difiere porque hiciste rebase o amend de lo que habías enviado | Force push (lease) | Merge, Rebase, Omitir |
| El remoto tiene commits de otra persona | Merge desde él (Rebase cuando **Botón Pull** está en rebase) | Rebase, Force push (lease), Omitir |
| El historial no comparte nada con el tuyo | Omitir | Force push (lease) |
| El host rechazó la rama (protegida) | Omitir | ninguna: forzar también se rechazaría |

La acción de cada fila es una lista que puedes cambiar (Izquierda/Derecha recorre la lista, Enter la abre, `s` omite la fila). Un merge muestra si tendría conflictos y en qué archivos. Un **force push** siempre se confirma, nombra el remoto y cuántos commits reemplaza, siempre usa lease (nunca un force a secas) y no se puede deshacer: el resultado indica qué commit se reemplazó y el comando que lo recupera. **Omitir problemas** pone en Omitir cada fila con problemas. **Ejecutar** (Ctrl+Enter) empieza.

Los merges, fast-forwards y rebases se ejecutan primero, en el orden de la lista, de modo que tu rama llega una sola vez a su estado final; después los push salen de un remoto en uno, con el upstream primero. No hay un «todo o nada» atómico entre remotos: cada uno termina con su propio resultado, y un fallo en uno (red, un hook) no detiene a los demás. **Reintentar fallidos** (o `Git: Reintentar remotos fallidos`) los intenta de nuevo. Cerrar la vista de progreso nunca detiene la ejecución; `Git: Mostrar push agrupado` la recupera.

### Conflictos y Continuar

Si un merge o rebase de la ejecución se detiene por conflictos, el diálogo se cierra y toman el relevo el banner de merge habitual y el editor de conflictos. Una franja sobre el Log dice `Pausa del push agrupado (gitlab): resuelve y concluye, luego Continuar.` con los botones **Continuar** y **Abortar grupo**.

- Resuelve los conflictos y concluye el merge o rebase (en GitPoulpi o con git a secas).
- **Continuar** está atenuado hasta que eso esté hecho, y **nunca se pulsa por ti**: nada continúa solo. Púlsalo (o ejecuta `Git: Continuar push agrupado`) y la ejecución vuelve a hacer push a cada remoto desde tu nueva punta. Tras un rebase, un remoto que aún tiene los commits antiguos vuelve al diálogo del plan (ahora necesita un force push) en lugar de forzarse en silencio.
- **Abortar grupo** (con confirmación; `Git: Abortar push agrupado`) descarta el merge o rebase que inició la ejecución y olvida la ejecución. Los push ya hechos se quedan.

La franja sobrevive a un reinicio de GitPoulpi, y la CLI ve la misma ejecución guardada.

## Hacer pull

- El botón **Pull** hace pull del upstream exactamente como antes, salvo que la rama tenga fuentes de pull: entonces hace pull del upstream y de cada fuente, por orden, cada una en el modo fijado por **Botón Pull** en Ajustes (merge, solo fast-forward o rebase). Una fuente que no puede avanzar con fast-forward se integra con merge (o rebase) como el upstream; con solo fast-forward, una fuente que necesita más pregunta antes.
- **Pull ▾** añade `Pull de todas las fuentes (origin + gitlab)` cuando hay fuentes, y `Pull solo de <remoto>` para cada remoto de la rama: un pull puntual de la rama de ese remoto, digan lo que digan las casillas.
- La paleta tiene `Git: Pull de todas las fuentes` y `Git: Pull de un remoto...`.
- Un pull que se detiene por conflictos deja la misma franja, rotulada como pull agrupado.

## Ajustes

Los cuatro están en **Ajustes > Git y codificación**.

| Ajuste | Por defecto | Significado |
|---|---|---|
| **Botón Push** | directo | directo hace push al instante; preguntar abre antes el menú de Push. |
| **Botón Pull** | merge | El modo del botón Pull (merge, solo fast-forward o rebase), y de las fuentes de pull. |
| **Comprobar todos los remotos antes de un push** | activado | Mira cada remoto antes de un push agrupado (la CLI lo sigue; `--no-preflight` lo desactiva para una llamada). |
| **Destinos de push adicionales para ramas nuevas** (por repositorio) | ninguno | Los remotos a los que una rama nueva también hace push. |

## Desde la línea de comandos

```
gitpoulpi branch upstream add gitlab --pull      # un destino de push que también es fuente de pull
gitpoulpi branch upstream list                    # cómo está configurada la rama
gitpoulpi push --all-targets                      # el upstream más cada destino
gitpoulpi push --to gitlab --to backup:feat-x     # exactamente estos, solo esta vez
gitpoulpi pull --all-sources                      # el upstream más cada fuente de pull
gitpoulpi remote-group plan                       # lo que haría un push agrupado, sin cambiar nada
gitpoulpi remote-group continue                   # tras resolver y concluir una parada
```

Cuando un remoto necesita una decisión, el comando responde con el plan (código 3) y no cambia nada; respóndele con `--resolve gitlab=merge`. Consulta [la referencia de la CLI](cli-reference.md) para cada opción. El servidor MCP tiene las mismas herramientas.

## Con git a secas

Los destinos son claves de configuración normales, así que un script o un alias pueden usarlas sin GitPoulpi:

```
git config --get-all branch.feature.gitpoulpi-push-to     # un valor por línea: remote o remote:branch

# push de la rama activa a su upstream y a cada destino
git config alias.pushall '!b=$(git symbolic-ref --short HEAD); git push; for t in $(git config --get-all branch.$b.gitpoulpi-push-to); do r=${t%%:*}; d=${t#*:}; [ "$d" = "$t" ] && d=$b; git push "$r" "$b:$d"; done'
```

Un valor es `<remote>` (el mismo nombre de rama) o `<remote>:<branch>`, que es lo que resuelve la línea `d=${t#*:}`. El alias simple no mira antes los remotos, no ofrece un merge y no puede continuar tras un conflicto: para eso está el diálogo.

## Conviene saber

- Dos remotos que apuntan a la misma dirección se detectan («gitlab y mirror usan la misma URL de push»); el segundo se omite por defecto.
- Un host con una rama protegida rechaza un force push; la fila lo dice y ofrece solo Omitir.
