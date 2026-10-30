[English](../following-the-tab-folder.md) · [Čeština](../cs/following-the-tab-folder.md) · [Deutsch](../de/following-the-tab-folder.md) · **Español** · [Français](../fr/following-the-tab-folder.md) · [Italiano](../it/following-the-tab-folder.md) · [日本語](../ja/following-the-tab-folder.md) · [한국어](../ko/following-the-tab-folder.md) · [Nederlands](../nl/following-the-tab-folder.md) · [Polski](../pl/following-the-tab-folder.md) · [Português (Brasil)](../pt-BR/following-the-tab-folder.md) · [Português (Portugal)](../pt-PT/following-the-tab-folder.md) · [Русский](../ru/following-the-tab-folder.md) · [Svenska](../sv/following-the-tab-folder.md) · [Türkçe](../tr/following-the-tab-folder.md) · [Українська](../uk/following-the-tab-folder.md) · [简体中文](../zh-CN/following-the-tab-folder.md)

# Seguir la carpeta de la pestaña activa

GitPoulpi puede tener varios repositorios abiertos como pestañas. El shell desde
el que lo iniciaste se queda en su carpeta: un programa no puede cambiar la
carpeta de su shell padre. GitPoulpi ofrece dos formas de obtener un shell en la
carpeta de la pestaña activa:

- **Salir y cd aquí**: GitPoulpi se cierra y tu shell cambia a la carpeta de la
  pestaña activa. Requiere una pequeña función de shell, que imprime
  `gitpoulpi shell-init`.
- **Abrir en**: abre la carpeta de la pestaña activa en un terminal, un panel de
  multiplexor, un editor o el gestor de archivos. No necesita configurar la
  shell. Consulta [Abrir en](open-in.md).

```
┌🐙 GitPoulpi vX.Y.Z -- /work/other-repo - Abrir en ▾ ── ⌕ Búsqueda global ⚙ × ┐
│ ⌂  main-repo x  other-repo x                   Salir y cd aquí          │
```

- **Abrir en** está en la barra de título, tras la ruta del repositorio. Se
  oculta cuando la ventana es demasiado estrecha.
- **Salir y cd aquí** está en el extremo derecho de la barra de pestañas. Solo
  aparece cuando se cumplen las dos condiciones siguientes:
  - GitPoulpi se inició mediante la función de shell (más abajo).
  - La pestaña activa no es la carpeta en la que iniciaste GitPoulpi.
- Los comandos siempre están en la paleta (`Ctrl+P`): `Pestaña: Salir y cd aquí`,
  `Pestaña: Abrir en…` (`Alt+O`) y `Pestaña: Abrir en el predeterminado` (`Ctrl+Alt+O`). Sin la
  función de shell, `Pestaña: Salir y cd aquí` te explica cómo configurarla.

Una salida normal (`q`) nunca cambia la carpeta de tu shell. Solo lo hace
**Salir y cd aquí**, salvo que elijas `auto` (más abajo).

## Configurar la función de shell

`gitpoulpi shell-init <shell>` imprime una función llamada `gp`. Usa `--name`
para elegir otro nombre. La función ejecuta
`gitpoulpi --cwd-file <archivo temporal> ...`. Cuando sales con **Salir y cd
aquí**, GitPoulpi escribe la carpeta en ese archivo y la función cambia a esa
carpeta. GitPoulpi no instala ni edita nada: tú añades una línea al archivo de
inicio de tu shell.

| Shell | Añádelo a | Línea |
|---|---|---|
| bash | `~/.bashrc` | `eval "$(gitpoulpi shell-init bash)"` |
| zsh | `~/.zshrc` | `eval "$(gitpoulpi shell-init zsh)"` |
| fish | `~/.config/fish/config.fish` | `gitpoulpi shell-init fish \| source` |
| PowerShell | tu `$PROFILE` (`notepad $PROFILE`) | `gitpoulpi shell-init pwsh \| Out-String \| Invoke-Expression` |

Después, inicia GitPoulpi con `gp` en lugar de `gitpoulpi`. Los argumentos se
pasan tal cual (`gp -s work`), y el código de salida es el de `gitpoulpi`.

**Un `gp` existente nunca se reemplaza sin avisar.** Si tu shell ya tiene un
comando, alias o función llamado `gp`, el script imprime un aviso y no define
nada. Elige otro nombre, por ejemplo
`gitpoulpi shell-init bash --name gpl`. En PowerShell, la función reemplaza el
alias integrado `gp` (`Get-ItemProperty`) y un alias que apunte a `gitpoulpi`.
Cualquier otro `gp` se conserva, con un aviso.

Todos los comandos de `gitpoulpi` aceptan `--cwd-file`, pero solo la TUI escribe
el archivo.

## Ajuste: Seguir la carpeta de la pestaña activa

Ajustes > Abrir en > **Seguir la carpeta de la pestaña activa**
(`ui.tab_dir_follow`):

| Valor | Comportamiento |
|---|---|
| `buttons` (por defecto) | **Salir y cd aquí** y **Abrir en**, como se describe arriba. |
| `auto` | Sin botón **Salir y cd aquí**. **Abrir en** se mantiene. Toda salida cambia tu shell a la carpeta de la pestaña activa (si iniciaste GitPoulpi con la función de shell). La entrada predeterminada de **Abrir en** se ejecuta cada vez que abres una pestaña nueva. |

En el modo `auto`, la entrada predeterminada se ejecuta solo para las pestañas
que abres tú: desde la pestaña Inicio, un worktree, un clon o un init. No se
ejecuta para las pestañas restauradas al arrancar, para la primera pestaña, ni
cuando cambias a una pestaña que ya está abierta.

Los ajustes antiguos `ui.terminal_launch` y `ui.terminal_launch_command` se han
movido a la lista **Abrir en**. Consulta [Abrir en](open-in.md#claves-heredadas).
