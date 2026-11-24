[English](../open-in.md) · [Čeština](../cs/open-in.md) · [Deutsch](../de/open-in.md) · [Español](../es/open-in.md) · [Français](../fr/open-in.md) · [Italiano](../it/open-in.md) · [日本語](../ja/open-in.md) · [한국어](../ko/open-in.md) · [Nederlands](../nl/open-in.md) · [Polski](../pl/open-in.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/open-in.md) · [Русский](../ru/open-in.md) · [Svenska](../sv/open-in.md) · [Türkçe](../tr/open-in.md) · [Українська](../uk/open-in.md) · [简体中文](../zh-CN/open-in.md)

# Abrir em

O botão **Abrir em ▾** na barra de título, ao lado do caminho do repositório, abre a
pasta da aba ativa em outro lugar: seu terminal, uma nova aba ou painel de um
multiplexador, um editor, o gerenciador de arquivos. Você escolhe o que fica na
lista. Nada é fixo no código: cada entrada é um rótulo e um comando.

## Como usar

- **Clique em Abrir em ▾**: uma lista se abre. Clique em uma entrada para executá-la.
  A entrada padrão é marcada com `•`; a última linha, **Configurar...**, abre as
  [Configurações](settings.md#abrir-em) na página Abrir em.

  ![A lista Abrir em: Terminal, dois editores, uma ferramenta Git e o gerenciador de arquivos, o padrão marcado com um ponto e Configurar no final](../images/open-in-menu.png)

- **Teclado**: `Alt+O` abre a mesma lista (`Up`/`Down`, `Enter`, `Esc`; com mais de
  sete entradas, digite para buscar). `Ctrl+Alt+O` executa a entrada padrão na
  hora. Paleta (`Ctrl+P`): `Aba: abrir em...` e `Aba: abrir no padrão`. Os comandos da
  paleta funcionam mesmo quando a barra de título não tem espaço para o botão. As
  duas teclas podem ser reatribuídas; veja [Atalhos de teclado](keybindings.md).
- **Por rótulo**: um passo `tab_open_in` de uma [sequência](config-reference.md#sequences) executa a entrada
  cujo **rótulo** você nomeia (`label = "VS Code"`), então reordenar a lista nunca muda o que
  ela executa. Um rótulo desconhecido interrompe o passo com "nenhuma entrada de “Abrir em” tem o rótulo ...".
  `Aba: terminal aqui` (paleta) abre o seu terminal na pasta, o antigo botão
  "Terminal aqui" da barra de abas.
- **Um agente de IA na pasta**: `Aba: abrir agente de IA aqui` abre o seu terminal na
  pasta da aba executando um agente de IA local; veja [IA](ai.md#abrir-o-agente-na-pasta-de-uma-aba).
- **Clique com o botão direito em uma aba de repositório**: o menu da aba tem um submenu **Abrir
  em** com as mesmas entradas.
- A lista começa na entrada que você usou por último nesta sessão.
- A linha de status diz “abrindo em ...” na hora. Se o programa falhar em cerca de
  1,5 segundo (por exemplo, tmux sem servidor, ou uma flag errada), a primeira
  linha da saída de erro dele aparece na linha de status. Um programa que não está
  instalado é nomeado na hora.

Até você salvar sua própria lista, a entrada padrão é escolhida a partir do que é
detectado: dentro do herdr, “Herdr: new tab”; no Omarchy, a nova janela do kitty (o
kitty é o terminal dele); caso contrário, o gerenciador de arquivos.

O botão aparece em toda aba de repositório, inclusive na pasta em que o GitPoulpi foi
iniciado, e no modo `ui.tab_dir_follow = "auto"`. No modo `auto`, a entrada padrão
também é executada sozinha quando você abre uma nova aba (não para as abas
restauradas na inicialização). Veja
[Seguindo a pasta da aba ativa](following-the-tab-folder.md).

## Como é uma entrada

No `config.toml`, uma tabela `[[ui.open_in]]` por entrada:

```toml
[ui]
open_in_default = "Herdr: new tab"   # um rótulo; vazio significa a primeira entrada

[[ui.open_in]]
label = "Herdr: new tab"
command = "herdr tab create --workspace ${env:HERDR_WORKSPACE_ID} --cwd ${cwd} --focus"

[[ui.open_in]]
label = "VS Code"
command = "code ${cwd}"
```

Você raramente escreve isso à mão: a página de Configurações o edita. Toda chave está
na [referência de configuração](config-reference.md).

### Comandos e variáveis

O comando é primeiro dividido em argumentos (espaços, `'...'` e `"..."` agrupam
palavras), e depois as variáveis são substituídas dentro de cada argumento. Uma
pasta com espaço no nome, portanto, continua sendo um único argumento. **Nenhum
shell é usado**, a menos que o seu comando inicie um.

| Variável | Substituída por |
|---|---|
| `${cwd}` | A pasta da aba. |
| `${repo}` | A raiz do repositório (difere de `${cwd}` em uma aba de worktree). |
| `${name}` | O nome base da pasta. |
| `${branch}` | A branch em checkout; vazio quando desanexado. |
| `${web_url}` | A página web do remoto origin quando ele é GitHub ou GitLab; vazio caso contrário. |
| `${env:NAME}` | A variável de ambiente `NAME`; vazio quando não definida. |

O antigo `{repo}` ainda funciona e significa `${cwd}`. Qualquer outro `${...}` é um
erro que nomeia a variável.

O programa é iniciado com a pasta da aba como diretório de trabalho e recebe três
variáveis de ambiente, úteis para um trecho de shell:

| Variável | Valor |
|---|---|
| `GITPOULPI_CWD` | A pasta da aba. |
| `GITPOULPI_REPO` | A raiz do repositório. |
| `GITPOULPI_BRANCH` | A branch, ou vazio. |

```toml
[[ui.open_in]]
label = "Terminal here, then status"
command = "sh -c 'cd \"$GITPOULPI_CWD\" && git status; exec $SHELL'"
```

No Windows, o programa é procurado por `PATH` e `PATHEXT`, então `code` e `cursor`
(arquivos `.cmd`) funcionam, e nenhuma janela de console pisca na tela.

## Predefinições

Configurações > Abrir em > **Adicionar predefinição** lista entradas prontas só para o
seu sistema operacional (as predefinições do Windows nunca aparecem no Linux, e assim
por diante), por categoria: a linha começa com a categoria, `Explorer: ...`,
`Multiplexer: ...`, `Terminal: ...`, `Editor: ...`, `Git tool: ...`, `AI agent: ...`,
`Web: ...`. No topo, **Detectado** são as entradas da lista padrão calculada. Dentro
de cada categoria, as predefinições que você pode usar de imediato vêm primeiro (você
roda dentro desse multiplexador ou terminal, e o programa está no `PATH`), depois as
instaladas, depois o resto. Digite na caixa de combinação para buscar, por exemplo `lazygit` ou
`Git tool:`. **Testar na aba atual** experimenta uma entrada antes de você
confiar nela.

![A lista + Adicionar da página Abrir em: as entradas detectadas primeiro, depois Explorador, Multiplexador e as outras categorias, cada predefinição com sua categoria como prefixo](../images/settings-open-in-presets.png)

As predefinições “Executar uma ferramenta em um painel” (`lazygit in Herdr tab`,
`yazi in tmux window`, `Claude Code in Herdr split`...) só são oferecidas quando o host
(herdr, tmux, zellij, WezTerm, kitty, `xdg-terminal-exec`, Windows Terminal) e a
ferramenta estão instalados. `$EDITOR in ...` executa o editor nomeado pela sua
variável `EDITOR` (com `vi` como alternativa no herdr) no herdr ou no tmux.

As predefinições de ferramentas do herdr criam a aba ou o split e depois digitam o
comando no shell dele, então a ferramenta inicia na pasta e o painel continua aberto
depois que ela termina. As de `AI agent` usam `herdr agent start`, então o herdr
acompanha o estado do agente.

| Categoria | Predefinições (`V` = verificada executando ou contra a ajuda instalada) |
|---|---|
| Explorador | `explorer.exe` do Windows (V), Finder do macOS (V), `xdg-open` do Linux (V), Nautilus, Dolphin, Thunar, Nemo, PCManFM; yazi, superfile, nnn, lf, ranger em um painel/aba/terminal |
| Multiplexador | herdr aba, workspace, split à direita/abaixo (V); tmux janela, split à direita/abaixo, sessão desanexada (V); zellij painel, aba, flutuante; WezTerm aba, split, nova janela; kitty aba, janela em split, janela do SO (V: flags da ajuda; precisa de `allow_remote_control`); screen; cmux; psmux |
| Terminal | `xdg-terminal-exec` (V), kitty (V), foot (V), Alacritty (+ `alacritty msg`), WezTerm, Ghostty (Linux, macOS), GNOME Terminal, Ptyxis, Konsole, Xfce Terminal, Tilix, Terminator, MATE Terminal, iTerm2, Terminal.app, Warp; Windows: Windows Terminal (janela, aba, split, e uma aba para cada um de PowerShell 7, Windows PowerShell, Prompt de Comando, WSL), janelas do PowerShell 7 / Windows PowerShell / Prompt de Comando / WSL, Git Bash |
| Editor | VS Code (V), VSCodium, Cursor, Windsurf, Zed, Sublime Text, Emacs, IntelliJ IDEA, RustRover, PyCharm, WebStorm, CLion, GoLand, Rider, PhpStorm, Android Studio, Visual Studio (Windows); Neovim, Vim, Helix, `$EDITOR` em um painel/aba/terminal |
| Ferramenta Git | gitk, `git gui`, gitg, Sublime Merge, GitKraken, GitHub Desktop; lazygit, tig, gitui em um painel/aba/terminal (os de herdr, tmux, kitty e `xdg-terminal-exec` são V) |
| Agente de IA | Claude Code, Codex, Gemini CLI, opencode em uma aba/split do herdr (V, com Claude), tmux, zellij, WezTerm, kitty, novo terminal |
| Web | `Repository page on the web` (`${web_url}`), `gh repo view --web` (V), `glab repo view --web` |

WSL no Windows: `wsl.exe --cd ${cwd}` aceita o caminho do Windows como está, e a aba
`wsl.exe` do Windows Terminal inicia na pasta, que o WSL mapeia sob `/mnt/`.

O explorador, o Windows Terminal, o PowerShell, o `cmd`, o WSL, o Warp, o psmux, o
Visual Studio e o Zed (o Zed é só para Unix) aparecem somente onde existem; tmux,
zellij, cmux, screen, o controle remoto do kitty e as predefinições do herdr
conduzidas por `sh` são só para Unix; herdr (predefinições simples), WezTerm,
Alacritty, VS Code, Cursor e as IDEs da JetBrains são oferecidos em todo lugar.

### Variável `${web_url}`

A página web do remoto `origin` do repositório, quando ele é um projeto do GitHub ou
do GitLab (github.com, gitlab.com, ou um host self-managed que tem uma entrada em
`[[integrations]]`); vazia caso contrário, e então uma entrada que a usa para com uma
mensagem em vez de abrir um endereço vazio. É opcional: nenhuma predefinição precisa
dela, exceto `Repository page on the web`.

## A página de Configurações

Configurações > **Abrir em** edita a lista. As entradas formam uma
[lista](settings.md#listas) à esquerda (a entrada padrão é marcada com `★`), e os
campos da entrada selecionada ficam à direita; `+ Adicionar`, `Duplicar`, `Remover`,
`↑` / `↓` e arrastar uma linha gerenciam a lista.

| Linha | O que faz |
|---|---|
| **Entrada padrão** | A entrada que **Abrir em padrão** executa. Ela é marcada com `•` na lista Abrir em. |
| **Rótulo** | O nome mostrado na lista. Único, não vazio. |
| **Comando** | O comando, com os chips de variáveis (`${cwd}`, `${repo}`, `${branch}`, `${name}`, `${web_url}`) para inserir. |
| **Testar na aba atual** | Executa a entrada agora na pasta da aba ativa. |
| **+ Adicionar ▾** | Acrescenta uma entrada pronta (as detectadas primeiro) ou **Personalizada**, uma entrada vazia cujo Comando é editado em seguida. |
| **Restaurar padrões detectados** | Substitui a lista pelo que é detectado nesta máquina e limpa o padrão. |

As edições são salvas conforme você as faz. A primeira edição grava o array
`[[ui.open_in]]` inteiro no `config.toml`; até lá a lista é calculada (veja
[Chaves legadas](#chaves-legadas)) e o arquivo não tem a chave `open_in`. A busca das
Configurações também encontra a página.

## Arquivos binários: Abrir em

Um arquivo binário (um PDF, um documento do Word, uma planilha, um arquivo compactado...) não tem texto para mostrar na visão Diff nem na visão de arquivo. Quando o sistema operacional conhece um aplicativo para o seu tipo, a visão mostra um botão **Abrir em** ao lado do aviso habitual. Sua lista contém o aplicativo com o qual o sistema abre esse tipo, primeiro e marcado com um marcador, e depois todos os outros aplicativos que o sistema registra para ele (a lista *Abrir com* do próprio sistema). No Linux é o tipo MIME do nome do arquivo, `mimeapps.list` para o padrão e os arquivos `.desktop` para os demais; no Windows o registro (a escolha do usuário e os programas registrados para a extensão); no macOS apenas o aplicativo padrão.

`o` executa o aplicativo padrão e `Shift+O` abre a lista. Ambos estão na paleta de comandos (*Arquivo: abrir no aplicativo padrão*, *Arquivo: abrir com...*), e **Abrir com...** está no menu de contexto de um arquivo do painel de status e da lista de arquivos de um commit. A busca roda em segundo plano, uma vez por extensão: a visão mostra “procurando aplicativos...” enquanto isso. Um tipo para o qual o sistema não conhece nenhum aplicativo, e o texto simples, mantêm o aviso simples e não têm botão.

![Um PDF na visualização de arquivo com sua lista Abrir em: primeiro o aplicativo padrão, marcado com um ponto, depois os outros aplicativos que o sistema registra para o tipo](../images/open-with.png)

O arquivo da árvore de trabalho é entregue ao aplicativo como está. A versão de um commit (ou do índice) é gravada em uma **cópia temporária somente leitura**, com um aviso na linha de status, para que um aplicativo não possa alterar o repositório. Nenhuma das duas altera o repositório: sem entrada no diário, Desfazer não é afetado.

A lista do menu abre ao lado do menu, na altura da linha **Abrir com...**, como um submenu (à esquerda quando não há espaço à direita), tanto pelo mouse quanto pelo teclado. A lista dos editores de imagem (**Editar em...**) aberta a partir de um menu de arquivo faz o mesmo.

### Arquivos Git LFS

O cartão de objeto Git LFS (veja [Git LFS](lfs.md)) traz as mesmas duas ações sob o texto: **Abrir no programa padrão** e **Abrir em**, com as mesmas teclas (`o`, `Shift+O`) e as mesmas linhas da paleta. Elas abrem o arquivo real, nunca o ponteiro: o arquivo da árvore de trabalho como está, ou uma cópia temporária somente leitura do objeto do armazenamento LFS local para a versão de um commit ou do índice. Um objeto que ainda não foi baixado não tem botões no cartão, e a linha de status diz que o conteúdo ainda não foi baixado. A mesma recusa vale para um arquivo da árvore de trabalho que ainda guarda só o ponteiro: **Abrir no programa padrão**, **Abrir em** e **Editar em**, pelo cartão ou por qualquer menu de arquivo, dizem que o conteúdo não foi baixado em vez de abrir o texto do ponteiro (baixe-o com as linhas de pull ou checkout da barra LFS e tente de novo). Abrir é somente leitura: sem entrada no diário, nada a desfazer.

### Navegadores no Windows

Chrome e Edge registram `--single-argument %1`, o que os faz ler tudo depois da opção como um único endereço, aspas incluídas. O GitPoulpi passa o caminho a um programa assim literalmente e sem aspas, como o Windows faz, então um arquivo com espaços no caminho abre em vez de falhar com `http://"c/...`. Todo outro programa recebe o caminho como um argumento comum.

## Chaves legadas

As antigas chaves `ui.terminal_launch` e `ui.terminal_launch_command` só são lidas
enquanto `ui.open_in` está ausente. Então a lista calculada é:

1. sua escolha antiga de terminal, se não for `auto` (`custom` vira “Terminal
   (custom)” com `{repo}` reescrito para `${cwd}`; um id de predefinição vira essa
   predefinição). Um valor `auto` vira a primeira entrada, “Default terminal”.
2. o explorador de arquivos do SO,
3. os multiplexadores e terminais detectados no ambiente,
4. os editores encontrados no `PATH`.

Quando você salva a lista pelas Configurações, `ui.open_in` assume e as chaves antigas
são ignoradas. Os comandos da paleta **Aba: abrir no terminal** e **Aba: abrir no
gerenciador de arquivos** não usam esta lista.

## Solução de problemas

- *“programa X não encontrado (PATH verificado)”*: o programa não está no `PATH` visto pelo GitPoulpi.
  Inicie o GitPoulpi de um shell que o tenha, ou escreva o caminho completo.
- *a entrada de tmux/zellij/herdr falha na hora*: eles falam com um servidor em
  execução, então só funcionam quando o GitPoulpi roda dentro dele.
- *a entrada de aba do kitty falha*: ela precisa de `allow_remote_control` no
  `kitty.conf`.
- *Explorer do Windows*: o `explorer.exe` sai com código 1 mesmo em caso de sucesso; o
  GitPoulpi trata isso como sucesso.
