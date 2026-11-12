[English](../multiplexers.md) · [Čeština](../cs/multiplexers.md) · [Deutsch](../de/multiplexers.md) · [Español](../es/multiplexers.md) · [Français](../fr/multiplexers.md) · [Italiano](../it/multiplexers.md) · [日本語](../ja/multiplexers.md) · [한국어](../ko/multiplexers.md) · [Nederlands](../nl/multiplexers.md) · [Polski](../pl/multiplexers.md) · [Português (Brasil)](../pt-BR/multiplexers.md) · **Português (Portugal)** · [Русский](../ru/multiplexers.md) · [Svenska](../sv/multiplexers.md) · [Türkçe](../tr/multiplexers.md) · [Українська](../uk/multiplexers.md) · [简体中文](../zh-CN/multiplexers.md)

# Multiplexadores e painéis

Um multiplexador fica entre o GitPoulpi e o seu terminal: redesenha ele próprio as células e
só reencaminha as sequências de escape que compreende. Por isso, o que funciona só no
terminal ([Escolher um terminal](terminals.md)) pode não funcionar dentro do tmux, zellij,
screen ou herdr. Esta página diz o que passa, e a configuração a acrescentar. Os resultados
vêm da mesma ronda de testes de novembro de 2026 das tabelas dos terminais: tmux 3.8 (MSYS2:
3.7c), zellij 0.45.1 e 0.46.0, GNU screen 5.0 (o macOS traz a 4.00.03), herdr 0.9.5,
psmux 3.4.0.

O GitPoulpi deteta um multiplexador a partir de `TMUX`, `ZELLIJ`, `STY`, `TERM=screen*` ou
`tmux*` e `HERDR_*`. Dentro de um, as imagens passam por predefinição a carateres, exceto
onde a tabela abaixo diz que o GitPoulpi deteta um caminho que funciona.
`GITPOULPI_TERMINAL_IMAGE=off` desliga todas as imagens, e `ui.image_protocol` força um
protocolo.

## Em resumo

| | Imagens | Forma do ponteiro (OSC 22) | Área de transferência (OSC 52) | Teclas |
|---|---|---|---|---|
| **tmux** | sixel quando o terminal por trás o aceita (detetado); imagens kitty com `allow-passthrough on` e `ui.image_protocol = "kitty"` | com `allow-passthrough on`, no kitty e no foot; não no Ghostty | com `set-clipboard on` | sem teclado kitty; `Ctrl+Enter` e `Ctrl+,` nunca chegam |
| **zellij** | carateres | não | sim | o teclado kitty funciona |
| **GNU screen** | carateres | não | não | teclas antigas |
| **herdr** | imagens kitty (detetadas com `kitty_graphics = true`) | não | sim | não medido |
| **psmux** (Windows) | carateres (por desenho) | não (o Windows Terminal não tem) | a área de transferência do sistema funciona | não medido |
| **tmux do MSYS2** (Windows) | carateres (por desenho) | não | a área de transferência do sistema funciona | não medido; o ecrã fica baralhado |
| **Painéis do WezTerm**, **painéis do Windows Terminal**, **divisões do kitty e do Ghostty**, **painéis divididos do iTerm2** | como o próprio terminal | como o terminal | como o terminal | como o terminal |
| **`tmux -CC` do iTerm2** | carateres | não | não (o iTerm2 nega-o por predefinição) | não medido |

## tmux

**Imagens.**

- O **sixel** é desenhado pelo próprio tmux, recodificado para um cliente que o suporte. O
  GitPoulpi pergunta uma vez ao tmux no arranque (`tmux display-message -p
  '#{client_termfeatures}'`, limitado a 400 ms) e usa sixel quando o cliente o indica. É o
  caso por trás do foot, WezTerm, Rio e Tabby; não é o caso por trás do kitty e do Ghostty
  (o tmux imprime então o texto «SIXEL IMAGE»). Precisa de um tmux compilado com sixel (o do
  Arch e o do Homebrew são; verifique com `tmux -V` e um teste de sixel). Sem definição. A
  própria resposta do tmux a uma consulta de atributos de dispositivo afirma sixel seja o que
  for que esteja por trás, e é por isso que o GitPoulpi pergunta antes pelas funcionalidades
  do cliente.
- O **iTerm2 por trás do tmux** anuncia sixel mas pinta de forma opaca os píxeis não
  definidos, pelo que o grafo, em grande parte transparente, passaria a um retângulo sólido.
  O GitPoulpi reconhece o iTerm2 (`LC_TERMINAL`, `ITERM_SESSION_ID`, o tipo de cliente) e
  desenha carateres.
- As **imagens kitty** passam pela passagem DCS do tmux (kitty, Ghostty; vistas também no
  iTerm2 e no Rio numa sonda em bruto). Precisam de `set -g allow-passthrough on` (tmux 3.3a
  ou posterior) **e** de `ui.image_protocol = "kitty"` no GitPoulpi, que então embrulha os
  envios e as eliminações. Sem a opção do tmux o embrulho é descartado e o grafo fica em
  branco.
- As imagens em linha do iTerm2 passam pelo tmux mas o tmux não as sabe posicionar: o
  GitPoulpi nunca as usa aí.

**Forma do ponteiro.** O GitPoulpi embrulha `OSC 22` para a passagem. Chega à janela no kitty
e no foot com `allow-passthrough on`. O Ghostty não o mostra quando embrulhado (a seta
fica). O WezTerm, Rio, Alacritty, Tabby e iTerm2 não têm `OSC 22` para mostrar.

**Área de transferência.** `set -g set-clipboard on`. O GitPoulpi envia também a forma
embrulhada para a passagem (funciona com `allow-passthrough on`). Por trás do kitty, Ghostty,
Alacritty e Tabby chega; o iTerm2 nega-o até a sua permissão de área de transferência estar
ligada.

**Teclas.** O tmux não responde à consulta de teclado kitty, e `extended-keys` não faz passar
`Ctrl+Enter` nem `Ctrl+,`: medido com o tmux 3.8 dentro do kitty, ambos chegam como um
simples `Enter` e `,` com `extended-keys` off, on ou always. Combinações de letras como
`Ctrl+Shift+Z` só chegam com `extended-keys` on ou always **e** `extended-keys-format csi-u`
(tmux 3.5 ou posterior) **e** uma aplicação que peça teclas estendidas; o GitPoulpi nunca o
faz, pelo que a definição aqui não muda nada. Use as variantes `Alt` dentro do tmux
([Atalhos de teclado](keybindings.md#teclas-que-o-terminal-não-consegue-enviar-e-o-protocolo-de-teclado-do-kitty)).

**Configuração** (`~/.tmux.conf`):

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a or later: pointer shapes, forced kitty pictures
```

**tmux dentro do WSL.** Não medido.

## zellij

Imagens: o zellij aceita o protocolo kitty mas não os seus marcadores Unicode, e não leva
imagens sixel nem iTerm2, pelo que o GitPoulpi desenha carateres (um veto deliberado). Forma
do ponteiro: engolida, pelo que o GitPoulpi não a envia. Área de transferência: o `OSC 52` é
reencaminhado (funciona por trás do kitty, Ghostty, Alacritty, Tabby; o iTerm2 e o
Terminal.app negam-no ou ignoram-no). Teclas: o zellij responde à consulta de teclado kitty,
pelo que o teclado completo funciona.

O zellij 0.46 abre uma janela «Welcome to Zellij» num arranque novo: prima `Esc`. Uma sessão
que terminou mantém o nome e o arranque seguinte com o mesmo nome pergunta o que fazer.
Não é precisa nenhuma configuração.

## GNU screen

Nada passa: as imagens e o `OSC 22` são impressos como carateres ou descartados, o `OSC 52` é
ignorado e o teclado é o antigo. O GitPoulpi desenha carateres. O screen 4.00.03 (a versão
que o macOS traz) também baralha as cores e os glifos largos na interface; o screen 5 foi
testado só em Linux. Use o tmux ou o zellij para uma sessão utilizável.

## herdr

O herdr incorpora um terminal que volta a emitir **imagens kitty**. O GitPoulpi usa-as quando
o `config.toml` do herdr diz `kitty_graphics = true` (em `[terminal]`, ou em `[experimental]`
nas versões mais antigas) e o terminal à volta do herdr as desenha (kitty, Ghostty,
WezTerm): detetado, sem definição. Se a configuração estiver noutro lugar (por exemplo sob
`HERDR_HOME`, que o GitPoulpi não lê) defina `ui.image_protocol = "kitty"`. Por SSH
(`herdr --remote`) o GitPoulpi não consegue ver o cliente local, pelo que desenha carateres;
veja [Resolução de problemas](troubleshooting.md#o-grafo-é-desenhado-com-carateres-não-como-imagem-herdr-tmux-outros-terminais).
O `OSC 22` não é reencaminhado pelo herdr 0.9.5, pelo que o GitPoulpi não o envia. O `OSC 52`
chega por trás do kitty, Ghostty, Alacritty e Tabby. Na sonda em bruto o herdr desenhou
imagens kitty no kitty, Ghostty e iTerm2, também com a configuração predefinida. Teclas: não
medidas.

## psmux e tmux do MSYS2 (Windows)

- O **psmux** (Rust, ConPTY, linguagem de comandos do tmux; `winget install psmux`) define
  `TMUX`, pelo que o GitPoulpi desenha aí carateres por desenho. Dentro do Windows Terminal
  corre normalmente e copiar funciona pela área de transferência do sistema; o ponteiro não
  muda (o Windows Terminal não tem `OSC 22`). Os outros terminais anfitriões não foram
  medidos.
- O **tmux do MSYS2 ou do Cygwin** também define `TMUX` (carateres por desenho), e no teste o
  ecrã ficou baralhado: fotogramas antigos desenhados uns sobre os outros no mintty e no
  Windows Terminal (com e sem `-u`; sem ele o desenho de caixas imprimia `qqqq`). Um programa
  nativo do Windows dentro de um pseudo-terminal MSYS2 não é redesenhado corretamente pelo
  tmux. Isso não se pode corrigir no GitPoulpi: use o psmux, os painéis do Windows Terminal
  ou os painéis do WezTerm.
- O zellij e o screen não têm versão para Windows. O herdr 0.9.5 foi instalado mas não
  executado.

## Painéis de terminal

Os painéis que pertencem ao próprio terminal não são multiplexadores para o GitPoulpi:

- **Painéis do WezTerm** (`wezterm cli split-pane`, `wezterm connect`, domínios SSH): o
  WezTerm desenha tudo, pelo que as imagens funcionam (grafo e avatares no painel) e o
  ponteiro comporta-se como no próprio WezTerm (as formas funcionam numa compilação com o
  PR #8092 do WezTerm upstream; o divisor do painel dá o ponteiro de redimensionar). Não
  medido em Linux nem em macOS.
- **Painéis do Windows Terminal** (`wt ; split-pane`): o grafo e os avatares sixel funcionam
  no painel, copiar funciona, sem formas de ponteiro.
- **Divisões do kitty, Ghostty e iTerm2**: nativas, como o terminal. Não medido em macOS para
  o kitty e o Ghostty (precisam de GPU).

## `tmux -CC` do iTerm2 (modo de controlo)

O iTerm2 abre uma janela nativa por cada janela do tmux e mantém uma janela de ligação
«[tmux bash]». `TMUX` está definido, pelo que o GitPoulpi desenha carateres na janela nativa.
As imagens chegam de facto ao iTerm2 pela ligação de controlo (a janela «Allow
Terminal-Initiated Display?» apareceu num teste `OSC 1337`), mas o GitPoulpi não as usa aí. A
carga kitty de um `ui.image_protocol = "kitty"` forçado é impressa na janela de ligação, pelo
que o deixe em `auto`. `OSC 22` e `OSC 52` não chegam.

## Definições a conhecer

| Definição | Uso |
|---|---|
| `ui.image_protocol` | `kitty` força imagens kitty através de um multiplexador que as reencaminha (tmux com `allow-passthrough on`, herdr com uma configuração que o GitPoulpi não lê); `none` para carateres. |
| `ui.graph_renderer` | `text` está sempre correto. |
| `ui.kitty_keyboard` | `off` se um multiplexador se portar mal com a consulta de teclado. |

Veja [Escolher um terminal](terminals.md) para o terminal por trás do multiplexador, e
[Resolução de problemas](troubleshooting.md) quando o grafo é desenhado com carateres.
