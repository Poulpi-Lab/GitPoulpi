[English](../following-the-tab-folder.md) · [Čeština](../cs/following-the-tab-folder.md) · [Deutsch](../de/following-the-tab-folder.md) · [Español](../es/following-the-tab-folder.md) · [Français](../fr/following-the-tab-folder.md) · [Italiano](../it/following-the-tab-folder.md) · [日本語](../ja/following-the-tab-folder.md) · [한국어](../ko/following-the-tab-folder.md) · [Nederlands](../nl/following-the-tab-folder.md) · [Polski](../pl/following-the-tab-folder.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/following-the-tab-folder.md) · [Русский](../ru/following-the-tab-folder.md) · [Svenska](../sv/following-the-tab-folder.md) · [Türkçe](../tr/following-the-tab-folder.md) · [Українська](../uk/following-the-tab-folder.md) · [简体中文](../zh-CN/following-the-tab-folder.md)

# Seguindo a pasta da aba ativa

O GitPoulpi pode ter vários repositórios abertos como abas. O shell de onde você
o iniciou permanece na sua pasta: um programa não pode mudar a pasta do shell
que o iniciou. O GitPoulpi oferece duas maneiras de obter um shell na pasta da
aba ativa:

- **Sair e cd aqui**: o GitPoulpi encerra, e o seu shell muda para a pasta da
  aba ativa. Isso exige uma pequena função de shell, impressa por
  `gitpoulpi shell-init`.
- **Abrir em**: abre a pasta da aba ativa em um terminal, um painel de
  multiplexador, um editor ou o gerenciador de arquivos. Não requer nenhuma
  configuração do shell. Veja [Abrir em](open-in.md).

```
┌🐙 GitPoulpi vX.Y.Z -- /work/other-repo - Open in ▾ ── ⌕ Busca global ⚙ × ┐
│ ⌂  main-repo x  other-repo x                   Quit & cd here           │
```

- **Abrir em** fica na barra de título, depois do caminho do repositório. Ele é
  ocultado quando a janela é estreita demais.
- **Sair e cd aqui** fica na ponta direita da barra de abas. Só aparece quando
  as duas condições são verdadeiras:
  - O GitPoulpi foi iniciado pela função de shell (abaixo).
  - A aba ativa não é a pasta em que você iniciou o GitPoulpi.
- Os comandos estão sempre na paleta (`Ctrl+P`): `Aba: sair e cd aqui`,
  `Aba: abrir em...` (`Alt+O`) e `Aba: abrir no padrão` (`Ctrl+Alt+O`). Sem a
  função de shell, `Aba: sair e cd aqui` diz como configurá-la.

Uma saída normal (`q`) nunca muda a pasta do seu shell. Só **Sair e cd aqui**
faz isso, a menos que você escolha `auto` (abaixo).

## Configurando a função de shell

`gitpoulpi shell-init <shell>` imprime uma função chamada `gp`. Use `--name`
para escolher outro nome. A função executa
`gitpoulpi --cwd-file <temp file> ...`. Quando você sai com **Sair e cd aqui**,
o GitPoulpi grava a pasta nesse arquivo, e a função muda para essa pasta. O
GitPoulpi não instala nem edita nada: você adiciona uma linha ao arquivo de
inicialização do seu shell.

| Shell | Adicione isto a | Linha |
|---|---|---|
| bash | `~/.bashrc` | `eval "$(gitpoulpi shell-init bash)"` |
| zsh | `~/.zshrc` | `eval "$(gitpoulpi shell-init zsh)"` |
| fish | `~/.config/fish/config.fish` | `gitpoulpi shell-init fish \| source` |
| PowerShell | seu `$PROFILE` (`notepad $PROFILE`) | `gitpoulpi shell-init pwsh \| Out-String \| Invoke-Expression` |

Depois inicie o GitPoulpi com `gp` em vez de `gitpoulpi`. Os argumentos são
repassados (`gp -s work`), e o código de saída é o código de saída do
`gitpoulpi`.

**Um `gp` existente nunca é substituído sem aviso.** Se o seu shell já tem um
comando, alias ou função chamado `gp`, o script imprime um aviso e não define
nada. Escolha outro nome, por exemplo
`gitpoulpi shell-init bash --name gpl`. No PowerShell, a função substitui o
alias interno `gp` (`Get-ItemProperty`) e um alias que aponte para
`gitpoulpi`. Qualquer outro `gp` é mantido, com um aviso.

Todo comando `gitpoulpi` aceita `--cwd-file`, mas só a TUI grava o arquivo.

## Configuração: Seguir a pasta da aba ativa

Configurações > Abrir em > **Seguir a pasta da aba ativa**
(`ui.tab_dir_follow`):

| Valor | Comportamento |
|---|---|
| `buttons` (padrão) | **Sair e cd aqui** e **Abrir em**, como descrito acima. |
| `auto` | Sem o botão **Sair e cd aqui**. **Abrir em** continua. Toda saída muda o seu shell para a pasta da aba ativa (quando você iniciou o GitPoulpi com a função de shell). A entrada padrão de **Abrir em** é executada cada vez que você abre uma nova aba. |

No modo `auto`, a entrada padrão é executada apenas para as abas que você mesmo
abre: a partir da aba Início, de um worktree, de um clone ou de um init. Ela não
é executada para as abas restauradas na inicialização, para a primeira aba, nem
quando você muda para uma aba que já está aberta.

As configurações mais antigas `ui.terminal_launch` e
`ui.terminal_launch_command` foram movidas para a lista **Abrir em**. Veja
[Abrir em](open-in.md#chaves-legadas).
