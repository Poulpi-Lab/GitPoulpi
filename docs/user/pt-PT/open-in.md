[English](../open-in.md) · [Čeština](../cs/open-in.md) · [Deutsch](../de/open-in.md) · [Español](../es/open-in.md) · [Français](../fr/open-in.md) · [Italiano](../it/open-in.md) · [日本語](../ja/open-in.md) · [한국어](../ko/open-in.md) · [Nederlands](../nl/open-in.md) · [Polski](../pl/open-in.md) · [Português (Brasil)](../pt-BR/open-in.md) · **Português (Portugal)** · [Русский](../ru/open-in.md) · [Svenska](../sv/open-in.md) · [Türkçe](../tr/open-in.md) · [Українська](../uk/open-in.md) · [简体中文](../zh-CN/open-in.md)

# Abrir em

O botão **Abrir em ▾** na barra de título, junto ao caminho do repositório, abre
a pasta do separador ativo noutro sítio: o seu terminal, um novo separador ou
painel de um multiplexador, um editor, o gestor de ficheiros. É o utilizador que escolhe o que
fica na lista. Nada está fixo no código: cada entrada é uma etiqueta e um
comando.

## Usá-lo

- **Clique em Abrir em ▾**: abre-se uma lista. Clique numa entrada para a
  executar. A entrada predefinida está marcada com `•`; a última linha,
  **Configurar...**, abre as [Definições](settings.md#abrir-em) na página Abrir
  em.

  ![A lista Abrir em: Terminal, dois editores, uma ferramenta Git e o gestor de ficheiros, a predefinida marcada com um ponto, e Configurar no fim](../images/open-in-menu.png)

- **Teclado**: `Alt+O` abre a mesma lista (`Up`/`Down`, `Enter`, `Esc`; com mais
  de sete entradas, escreva para pesquisar). `Ctrl+Alt+O` executa de imediato a
  entrada predefinida. Paleta (`Ctrl+P`): `Separador: abrir em...` e
  `Separador: abrir no predefinido`. Os comandos da paleta funcionam mesmo
  quando a barra de título não tem espaço para o botão. Ambas as teclas podem
  ser reatribuídas; veja [Atalhos de teclado](keybindings.md).
- **Clique com o botão direito num separador de repositório**: o menu do
  separador tem um submenu **Abrir em** com as mesmas entradas.
- A lista começa na entrada que usou da última vez nesta sessão.
- A barra de estado diz "a abrir em ..." de imediato. Se o programa falhar em
  cerca de 1,5 segundos (por exemplo tmux sem servidor, ou uma opção errada), a
  primeira linha da sua saída de erro aparece na barra de estado. Um programa
  que não está instalado é nomeado de imediato.

Até guardar a sua própria lista, a entrada predefinida é escolhida a partir do
que é detetado: dentro do herdr, "Herdr: new tab"; no Omarchy, a nova janela do
kitty (o kitty é o seu terminal); caso contrário, o gestor de ficheiros.

O botão aparece em todos os separadores de repositório, incluindo a pasta onde o
GitPoulpi foi iniciado, e no modo `ui.tab_dir_follow = "auto"`. No modo `auto`,
a entrada predefinida também corre sozinha quando abre um novo separador (não
nos separadores restaurados no arranque). Veja
[Seguir a pasta do separador ativo](following-the-tab-folder.md).

## Como é uma entrada

No `config.toml`, uma tabela `[[ui.open_in]]` por entrada:

```toml
[ui]
open_in_default = "Herdr: new tab"   # uma etiqueta; vazio significa a primeira entrada

[[ui.open_in]]
label = "Herdr: new tab"
command = "herdr tab create --workspace ${env:HERDR_WORKSPACE_ID} --cwd ${cwd} --focus"

[[ui.open_in]]
label = "VS Code"
command = "code ${cwd}"
```

Raramente a escreve à mão: a página das Definições edita-a. Todas as chaves
estão na [referência de configuração](config-reference.md).

### Comandos e variáveis

O comando é primeiro dividido em argumentos (espaços, `'...'` e `"..."` agrupam
palavras), e só depois as variáveis são substituídas dentro de cada argumento.
Uma pasta com um espaço no nome continua, por isso, a ser um só argumento.
**Nenhuma shell é envolvida**, a menos que o seu comando inicie uma.

| Variável | Substituída por |
|---|---|
| `${cwd}` | A pasta do separador. |
| `${repo}` | A raiz do repositório (difere de `${cwd}` num separador de worktree). |
| `${name}` | O nome base da pasta. |
| `${branch}` | A branch em checkout; vazio quando desanexada. |
| `${web_url}` | A página web do remoto origin quando é GitHub ou GitLab; vazio caso contrário. |
| `${env:NAME}` | A variável de ambiente `NAME`; vazio quando não definida. |

O antigo `{repo}` ainda funciona e significa `${cwd}`. Qualquer outro `${...}` é
um erro que nomeia a variável.

O programa é iniciado com a pasta do separador como diretório de trabalho, e
recebe três variáveis de ambiente, úteis para um trecho de shell:

| Variável | Valor |
|---|---|
| `GITPOULPI_CWD` | A pasta do separador. |
| `GITPOULPI_REPO` | A raiz do repositório. |
| `GITPOULPI_BRANCH` | A branch, ou vazio. |

```toml
[[ui.open_in]]
label = "Terminal here, then status"
command = "sh -c 'cd \"$GITPOULPI_CWD\" && git status; exec $SHELL'"
```

No Windows o programa é procurado através de `PATH` e `PATHEXT`, pelo que `code`
e `cursor` (ficheiros `.cmd`) funcionam, e nenhuma janela de consola pisca.

## Predefinições

Definições > Abrir em > **Adicionar predefinição** lista entradas já prontas
apenas para o seu sistema operativo (as predefinições do Windows nunca aparecem
no Linux, e assim por diante), por categoria: a linha começa pela categoria,
`Explorador: ...`, `Multiplexador: ...`, `Terminal: ...`, `Editor: ...`,
`Ferramenta Git: ...`, `Agente de IA: ...`, `Web: ...`. No topo, **Detetado**
são as entradas da lista predefinida calculada. Dentro de cada categoria, as
predefinições que pode usar de imediato vêm primeiro (está a correr dentro desse
multiplexador ou terminal, e o programa está no `PATH`), depois as instaladas,
depois as restantes. Escreva na caixa combinada para pesquisar, por exemplo
`lazygit` ou `Ferramenta Git:`. **Testar no separador atual** experimenta uma
entrada antes de contar com ela.

![A lista + Adicionar da página Abrir em: primeiro as entradas Detetado, depois Explorador, Multiplexador e as outras categorias, cada predefinição com o prefixo da sua categoria](../images/settings-open-in-presets.png)

As predefinições "executar uma ferramenta num painel" (`lazygit in Herdr tab`,
`yazi in tmux window`, `Claude Code in Herdr split`...) só são oferecidas quando
o anfitrião (herdr, tmux, zellij, WezTerm, kitty, `xdg-terminal-exec`, Windows
Terminal) e a ferramenta estão instalados. `$EDITOR in ...` executa o editor
indicado pela sua variável `EDITOR` (recorrendo a `vi` no herdr) no herdr ou no
tmux.

As predefinições de ferramentas do herdr criam o separador ou a divisão e depois
escrevem o comando na sua shell, pelo que a ferramenta arranca na pasta e o
painel fica aberto depois de ela terminar. As de `Agente de IA` usam `herdr agent
start`, pelo que o herdr acompanha o estado do agente.

| Categoria | Predefinições (`V` = verificada ao executar ou contra a ajuda instalada) |
|---|---|
| Explorador | `explorer.exe` do Windows (V), Finder do macOS (V), `xdg-open` do Linux (V), Nautilus, Dolphin, Thunar, Nemo, PCManFM; yazi, superfile, nnn, lf, ranger num painel/separador/terminal |
| Multiplexador | herdr separador, workspace, divisão à direita/em baixo (V); tmux janela, divisão à direita/em baixo, sessão desanexada (V); zellij painel, separador, flutuante; WezTerm separador, divisão, nova janela; kitty separador, janela dividida, janela do SO (V: opções da ajuda; precisa de `allow_remote_control`); screen; cmux; psmux |
| Terminal | `xdg-terminal-exec` (V), kitty (V), foot (V), Alacritty (+ `alacritty msg`), WezTerm, Ghostty (Linux, macOS), GNOME Terminal, Ptyxis, Konsole, Xfce Terminal, Tilix, Terminator, MATE Terminal, iTerm2, Terminal.app, Warp; Windows: Windows Terminal (janela, separador, divisão, e um separador para cada um de PowerShell 7, Windows PowerShell, Linha de Comandos, WSL), janelas de PowerShell 7 / Windows PowerShell / Linha de Comandos / WSL, Git Bash |
| Editor | VS Code (V), VSCodium, Cursor, Windsurf, Zed, Sublime Text, Emacs, IntelliJ IDEA, RustRover, PyCharm, WebStorm, CLion, GoLand, Rider, PhpStorm, Android Studio, Visual Studio (Windows); Neovim, Vim, Helix, `$EDITOR` num painel/separador/terminal |
| Ferramenta Git | gitk, `git gui`, gitg, Sublime Merge, GitKraken, GitHub Desktop; lazygit, tig, gitui num painel/separador/terminal (as de herdr, tmux, kitty e `xdg-terminal-exec` V) |
| Agente de IA | Claude Code, Codex, Gemini CLI, opencode num separador/divisão do herdr (V, com Claude), tmux, zellij, WezTerm, kitty, novo terminal |
| Web | `Repository page on the web` (`${web_url}`), `gh repo view --web` (V), `glab repo view --web` |

WSL no Windows: `wsl.exe --cd ${cwd}` aceita o caminho do Windows tal como está,
e o separador `wsl.exe` do Windows Terminal arranca na pasta, que o WSL mapeia
sob `/mnt/`.

O explorador, o Windows Terminal, o PowerShell, o `cmd`, o WSL, o Warp, o psmux,
o Visual Studio e o Zed (o Zed é só para Unix) aparecem apenas onde existem;
tmux, zellij, cmux, screen, o controlo remoto do kitty e as predefinições do
herdr baseadas em `sh` são só para Unix; o herdr (predefinições simples),
WezTerm, Alacritty, VS Code, Cursor e os IDEs da JetBrains são oferecidos em
todo o lado.

### Variável `${web_url}`

A página web do remoto `origin` do repositório, quando é um projeto GitHub ou
GitLab (github.com, gitlab.com, ou um anfitrião auto-gerido que tenha uma
entrada em `[[integrations]]`); vazio caso contrário, e nesse caso uma entrada
que a use pára com uma mensagem em vez de abrir um endereço vazio. É opcional:
nenhuma predefinição precisa dela, exceto `Repository page on the web`.

## A página das Definições

Definições > **Abrir em** edita a lista. As entradas formam uma
[lista](settings.md#listas) à esquerda (a entrada predefinida está marcada com `★`);
os campos da entrada selecionada ficam à direita; `+ Adicionar`, `Duplicar`,
`Remover`, `↑` / `↓` e arrastar uma linha gerem a lista.

| Linha | O que faz |
|---|---|
| **Entrada predefinida** | A entrada que **Abrir no predefinido** executa. Está marcada com `•` na lista Abrir em. |
| **Etiqueta** | O nome mostrado na lista. Único, não vazio. |
| **Comando** | O comando, com os chips de variável (`${cwd}`, `${repo}`, `${branch}`, `${name}`, `${web_url}`) para inserir. |
| **Testar no separador atual** | Executa a entrada agora na pasta do separador ativo. |
| **+ Adicionar ▾** | Acrescenta uma entrada já pronta (as detetadas primeiro) ou **Personalizada**, uma entrada vazia cujo Comando é editado a seguir. |
| **Repor predefinições detetadas** | Substitui a lista pelo que é detetado neste computador e limpa a predefinida. |

As edições são guardadas à medida que as faz. A primeira edição escreve todo o
array `[[ui.open_in]]` no `config.toml`; até lá a lista é calculada (veja
[Chaves antigas](#chaves-antigas)) e o ficheiro não tem a chave `open_in`. A
pesquisa das Definições também encontra a página.

## Ficheiros binários: Abrir em

Um ficheiro binário (um PDF, um documento do Word, uma folha de cálculo, um arquivo comprimido...) não tem texto para mostrar na vista Diff nem na vista de ficheiro. Quando o sistema operativo conhece uma aplicação para o seu tipo, a vista mostra um botão **Abrir em** junto do aviso habitual. A sua lista contém a aplicação com que o sistema abre esse tipo, primeiro e marcada com um marcador, e depois todas as outras aplicações que o sistema regista para ele (a lista *Abrir com* do próprio sistema). No Linux é o tipo MIME do nome do ficheiro, `mimeapps.list` para a predefinida e os ficheiros `.desktop` para as outras; no Windows o registo (a escolha do utilizador e os programas registados para a extensão); no macOS apenas a aplicação predefinida.

`o` executa a aplicação predefinida e `Shift+O` abre a lista. Ambos estão na paleta de comandos (*Ficheiro: abrir na aplicação predefinida*, *Ficheiro: abrir com...*), e **Abrir com...** está no menu de contexto de um ficheiro do painel de estado e da lista de ficheiros de um commit. A pesquisa corre em segundo plano, uma vez por extensão: a vista mostra "a procurar aplicações..." enquanto isso. Um tipo para o qual o sistema não conhece nenhuma aplicação, e o texto simples, mantêm o aviso simples e não têm botão.

![Um PDF na vista de ficheiro com a sua lista Abrir em: primeiro a aplicação predefinida, marcada com um ponto, depois as outras aplicações que o sistema regista para o tipo](../images/open-with.png)

O ficheiro da árvore de trabalho é entregue à aplicação tal como está. A versão de um commit (ou do índice) é escrita numa **cópia temporária só de leitura**, com um aviso na barra de estado, para que uma aplicação não possa alterar o repositório. Nenhuma das duas altera o repositório: sem entrada no diário, Desfazer não é afetado.

## Chaves antigas

As chaves antigas `ui.terminal_launch` e `ui.terminal_launch_command` só são
lidas enquanto `ui.open_in` estiver em falta. Nesse caso, a lista calculada é:

1. a sua antiga escolha de terminal, se não for `auto` (`custom` torna-se
   "Terminal (custom)" com `{repo}` reescrito para `${cwd}`; um id de
   predefinição torna-se essa predefinição). Um valor `auto` torna-se a primeira
   entrada, "Default terminal".
2. o explorador de ficheiros do SO,
3. os multiplexadores e terminais detetados a partir do ambiente,
4. os editores encontrados no `PATH`.

Quando guarda a lista a partir das Definições, `ui.open_in` assume o comando e as
chaves antigas são ignoradas. Os comandos da paleta **Separador: abrir no
terminal** e **Separador: abrir no gestor de ficheiros** não usam esta lista.

## Resolução de problemas

- *"{label}: programa X não encontrado (PATH verificado)"*: o programa não está no `PATH` visto pelo
  GitPoulpi. Inicie o GitPoulpi a partir de uma shell que o tenha, ou escreva o
  caminho completo.
- *a entrada de tmux/zellij/herdr falha de imediato*: estes falam com um
  servidor em execução, por isso só funcionam quando o GitPoulpi corre dentro
  dele.
- *a entrada do separador do kitty falha*: precisa de `allow_remote_control` no
  `kitty.conf`.
- *Explorador do Windows*: o `explorer.exe` termina com o código 1 mesmo em caso
  de sucesso; o GitPoulpi trata isso como sucesso.
