[English](../submodules.md) · [Čeština](../cs/submodules.md) · [Deutsch](../de/submodules.md) · **Español** · [Français](../fr/submodules.md) · [Italiano](../it/submodules.md) · [日本語](../ja/submodules.md) · [한국어](../ko/submodules.md) · [Nederlands](../nl/submodules.md) · [Polski](../pl/submodules.md) · [Português (Brasil)](../pt-BR/submodules.md) · [Português (Portugal)](../pt-PT/submodules.md) · [Русский](../ru/submodules.md) · [Svenska](../sv/submodules.md) · [Türkçe](../tr/submodules.md) · [Українська](../uk/submodules.md) · [简体中文](../zh-CN/submodules.md)

# Submódulos

Un submódulo es otro repositorio anidado dentro del tuyo en un commit fijo.
GitPoulpi los enumera en la barra lateral Refs, los actualiza desde la interfaz,
la CLI o la paleta de comandos, y puede mantenerlos sincronizados con el commit al que
haces checkout.

## La sección SUBMÓDULOS

Cuando un repositorio tiene un `.gitmodules`, la barra lateral Refs incorpora
una sección **SUBMÓDULOS** (debajo de las demás; no aparece si no hay ninguno).
Una fila por submódulo: una marca de estado, su ruta y un commit corto.

| Marca | Estado | Significado |
|---|---|---|
| `✓` | al día | en el commit que registra el repositorio |
| `↑` | modificado | en otro commit (avanzó, o cambiaste de rama sin actualizarlo) |
| `○` | sin clonar | declarado en `.gitmodules` y registrado, pero su carpeta está vacía: actualízalo para clonarlo |
| `✗` | en conflicto | el repositorio tiene conflictos de merge en el submódulo |

| Acción | Ratón | Teclado |
|---|---|---|
| Abrir el submódulo en una pestaña propia | doble clic en la fila | selecciónalo, `Enter` (paleta: `Refs: Abrir submódulo en una pestaña`) |
| Menú: **Actualizar**, **Inicializar**, **Abrir en una pestaña nueva** | clic derecho en la fila | selecciónalo, la tecla de menú contextual (por defecto `.`) |
| Actualizar el submódulo seleccionado (lo clona antes si hace falta) | menú | `Ctrl+P`: `Refs: Actualizar submódulo (seleccionado)` |
| Registrarlo en `.git/config` sin clonarlo | menú | `Ctrl+P`: `Refs: Inicializar submódulo (seleccionado)` |
| Actualizar todos los submódulos | | `Ctrl+P`: `Git: Actualizar todos los submódulos` |
| Saltar a la sección | clic en su encabezado | `Ctrl+P`: `Navegación: Ir a Refs > Submódulos` |

Una actualización puede hacer fetch del remoto del submódulo, por lo que sigue
**Ajustes > SSH y credenciales > Credenciales HTTPS** como cualquier otra
transferencia. Un submódulo sin clonar no tiene nada que abrir como pestaña; la
barra de estado lo indica y apunta a Actualizar.

## Mantener los submódulos al día

**Ajustes > Sincronización > Mantener los submódulos al día**
(`general.keep_submodules_up_to_date`, desactivado por defecto). Si está
activado, tras un **checkout** o un **pull** correcto se ejecuta
`git submodule update --init --recursive`. Esto solo ocurre si algún submódulo
no coincide con lo que el repositorio registra ahora, de modo que un
repositorio sin submódulos no se ralentiza. Si esa actualización falla, el
checkout o el pull se hicieron igualmente y el error lo indica.

## Desde la línea de comandos

```
gitpoulpi submodule list                       # ruta, estado, url (--json: la lista completa)
gitpoulpi submodule init [<path>...]           # registrar, sin clonar
gitpoulpi submodule update [<path>...] [--init] [--recursive]
```

Sin rutas, se aplica a todos los submódulos. Los tres comandos también son
herramientas MCP (`submodule_list` es de lectura; `init` y `update` son
mutaciones y siguen `allow_mutations` de `[mcp]`).

## Notas

- Solo se enumera el primer nivel; los submódulos de un submódulo se ven cuando
  lo abres como pestaña (o usa `--recursive` para actualizarlos).
- Las versiones recientes de git se niegan a clonar un submódulo de *ruta
  local* por el transporte `file` salvo que `protocol.file.allow` lo permita.
  Solo importa para submódulos cuya URL es una carpeta local; configúralo en tu
  configuración global de git si los usas.
