[English](../following-the-tab-folder.md) · [Čeština](../cs/following-the-tab-folder.md) · [Deutsch](../de/following-the-tab-folder.md) · [Español](../es/following-the-tab-folder.md) · [Français](../fr/following-the-tab-folder.md) · [Italiano](../it/following-the-tab-folder.md) · [日本語](../ja/following-the-tab-folder.md) · [한국어](../ko/following-the-tab-folder.md) · [Nederlands](../nl/following-the-tab-folder.md) · [Polski](../pl/following-the-tab-folder.md) · [Português (Brasil)](../pt-BR/following-the-tab-folder.md) · **Português (Portugal)** · [Русский](../ru/following-the-tab-folder.md) · [Svenska](../sv/following-the-tab-folder.md) · [Türkçe](../tr/following-the-tab-folder.md) · [Українська](../uk/following-the-tab-folder.md) · [简体中文](../zh-CN/following-the-tab-folder.md)

# Seguir a pasta do separador ativo

O GitPoulpi pode ter vários repositórios abertos como separadores. A shell a
partir da qual o iniciou fica na sua pasta: um programa não consegue alterar a
pasta da shell que o lançou. O GitPoulpi oferece duas formas de obter uma shell
na pasta do separador ativo:

- **Sair e cd aqui**: o GitPoulpi sai e a sua shell muda para a pasta do
  separador ativo. Isto precisa de uma pequena função de shell, impressa por
  `gitpoulpi shell-init`.
- **Abrir em**: abre a pasta do separador ativo num terminal, num painel de um
  multiplexador, num editor ou no gestor de ficheiros. Não é preciso configurar
  nenhuma shell. Veja [Abrir em](open-in.md).

```
┌🐙 GitPoulpi vX.Y.Z -- /work/other-repo - Abrir em ▾ ⌕ Pesquisa global ⚙ × ┐
│ ⌂  main-repo x  other-repo x                   Sair e cd aqui          │
```

- **Abrir em** está na barra de título, depois do caminho do repositório. Fica
  oculto quando a janela é demasiado estreita.
- **Sair e cd aqui** está na extremidade direita da barra de separadores. Só
  aparece quando as duas condições se verificam:
  - O GitPoulpi foi iniciado através da função de shell (abaixo).
  - O separador ativo não é a pasta em que iniciou o GitPoulpi.
- Os comandos estão sempre na paleta (`Ctrl+P`): `Separador: sair e cd aqui`,
  `Separador: abrir em...` (`Alt+O`) e `Separador: abrir no predefinido`
  (`Ctrl+Alt+O`). Sem a função de shell, `Separador: sair e cd aqui` diz-lhe
  como configurá-la.

Uma saída normal (`q`) nunca altera a pasta da sua shell. Só **Sair e cd aqui**
o faz, a menos que escolha `auto` (abaixo).

## Configurar a função de shell

`gitpoulpi shell-init <shell>` imprime uma função chamada `gp`. Use `--name`
para escolher outro nome. A função executa
`gitpoulpi --cwd-file <temp file> ...`. Quando sai com **Sair e cd aqui**, o
GitPoulpi escreve a pasta nesse ficheiro e a função muda para essa pasta. O
GitPoulpi não instala nem edita nada: é o utilizador que acrescenta uma linha ao
ficheiro de arranque do seu shell.

| Shell | Adicione isto a | Linha |
|---|---|---|
| bash | `~/.bashrc` | `eval "$(gitpoulpi shell-init bash)"` |
| zsh | `~/.zshrc` | `eval "$(gitpoulpi shell-init zsh)"` |
| fish | `~/.config/fish/config.fish` | `gitpoulpi shell-init fish \| source` |
| PowerShell | o seu `$PROFILE` (`notepad $PROFILE`) | `gitpoulpi shell-init pwsh \| Out-String \| Invoke-Expression` |

Depois inicie o GitPoulpi com `gp` em vez de `gitpoulpi`. Os argumentos são
reencaminhados (`gp -s work`) e o código de saída é o de `gitpoulpi`.

**Um `gp` existente nunca é substituído sem aviso.** Se a sua shell já tiver um
comando, alias ou função chamado `gp`, o script imprime um aviso e não define
nada. Escolha outro nome, por exemplo `gitpoulpi shell-init bash --name gpl`.
No PowerShell, a função substitui o alias `gp` incorporado (`Get-ItemProperty`)
e um alias que aponte para `gitpoulpi`. Qualquer outro `gp` é mantido, com um
aviso.

Todos os comandos `gitpoulpi` aceitam `--cwd-file`, mas só a TUI escreve o
ficheiro.

## Definição: Seguir a pasta do separador ativo

Definições > Abrir em > **Seguir a pasta do separador ativo**
(`ui.tab_dir_follow`):

| Valor | Comportamento |
|---|---|
| `buttons` (predefinição) | **Sair e cd aqui** e **Abrir em** como descrito acima. |
| `auto` | Sem o botão **Sair e cd aqui**. **Abrir em** mantém-se. Cada saída muda a sua shell para a pasta do separador ativo (quando iniciou o GitPoulpi com a função de shell). A entrada predefinida de **Abrir em** é executada sempre que abre um novo separador. |

No modo `auto`, a entrada predefinida só é executada para os separadores que
abre manualmente: a partir do separador Início, de um worktree, de um clone ou de
um init. Não é executada para os separadores restaurados no arranque, para o
primeiro separador, nem quando muda para um separador que já está aberto.

As definições mais antigas `ui.terminal_launch` e `ui.terminal_launch_command`
foram movidas para a lista **Abrir em**. Veja
[Abrir em](open-in.md#chaves-antigas).
