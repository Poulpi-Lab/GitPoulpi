[English](../multiplexers.md) · [Čeština](../cs/multiplexers.md) · [Deutsch](../de/multiplexers.md) · [Español](../es/multiplexers.md) · [Français](../fr/multiplexers.md) · [Italiano](../it/multiplexers.md) · [日本語](../ja/multiplexers.md) · [한국어](../ko/multiplexers.md) · [Nederlands](../nl/multiplexers.md) · [Polski](../pl/multiplexers.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/multiplexers.md) · [Русский](../ru/multiplexers.md) · [Svenska](../sv/multiplexers.md) · [Türkçe](../tr/multiplexers.md) · [Українська](../uk/multiplexers.md) · [简体中文](../zh-CN/multiplexers.md)

# Multiplexadores e painéis

Um multiplexador fica entre o GitPoulpi e o seu terminal: ele redesenha as células por conta
própria e repassa apenas as sequências de escape que entende. Por isso, o que funciona no
terminal sozinho ([Escolhendo um terminal](terminals.md)) pode não funcionar dentro do tmux,
zellij, screen ou herdr. Esta página diz o que passa e qual configuração acrescentar. Os
resultados vêm da mesma rodada de testes de novembro de 2026 das tabelas de terminais: tmux 3.8
(MSYS2: 3.7c), zellij 0.45.1 e 0.46.0, GNU screen 5.0 (o macOS traz a 4.00.03), herdr 0.9.5,
psmux 3.4.0.

O GitPoulpi detecta um multiplexador por `TMUX`, `ZELLIJ`, `STY`, `TERM=screen*` ou `tmux*`
e `HERDR_*`. Dentro de um deles, as imagens ficam em caracteres por padrão, exceto onde a
tabela abaixo diz que o GitPoulpi detecta um caminho que funciona. `GITPOULPI_TERMINAL_IMAGE=off`
desativa todas as imagens, e `ui.image_protocol` força um protocolo.

## Em resumo

| | Imagens | Formato do ponteiro (OSC 22) | Área de transferência (OSC 52) | Teclas |
|---|---|---|---|---|
| **tmux** | sixel quando o terminal por trás o aceita (detectado); imagens do kitty com `allow-passthrough on` e `ui.image_protocol = "kitty"` | com `allow-passthrough on`, no kitty e no foot; não no Ghostty | com `set-clipboard on` | sem teclado do kitty; `Ctrl+Enter` e `Ctrl+,` nunca chegam |
| **zellij** | caracteres | não | sim | o teclado do kitty funciona |
| **GNU screen** | caracteres | não | não | teclas legadas |
| **herdr** | imagens do kitty (detectadas com `kitty_graphics = true`) | não | sim | não medido |
| **psmux** (Windows) | caracteres (de propósito) | não (o Windows Terminal não tem) | a área de transferência do sistema funciona | não medido |
| **tmux do MSYS2** (Windows) | caracteres (de propósito) | não | a área de transferência do sistema funciona | não medido; a tela fica embaralhada |
| **Painéis do WezTerm**, **painéis do Windows Terminal**, **divisões do kitty e do Ghostty**, **painéis divididos do iTerm2** | como o próprio terminal | como o terminal | como o terminal | como o terminal |
| **`tmux -CC` do iTerm2** | caracteres | não | não (o iTerm2 nega por padrão) | não medido |

## tmux

**Imagens.**

- O **sixel** é desenhado pelo próprio tmux, recodificado para um cliente que o suporte. O
  GitPoulpi pergunta ao tmux uma vez na inicialização (`tmux display-message -p '#{client_termfeatures}'`,
  limitado a 400 ms) e usa sixel quando o cliente o lista. É o caso por trás do foot,
  WezTerm, Rio e Tabby; não é o caso por trás do kitty e do Ghostty (o tmux então imprime
  o texto "SIXEL IMAGE"). Exige um tmux compilado com sixel (o do Arch e o do Homebrew são;
  confira com `tmux -V` e um teste de sixel). Sem configuração. A própria resposta do tmux a
  uma consulta de atributos do dispositivo diz ter sixel seja o que for que esteja por trás,
  e por isso o GitPoulpi pergunta pelos recursos do cliente.
- O **iTerm2 por trás do tmux** anuncia sixel mas pinta de opaco os pixels não definidos, então
  o grafo, quase todo transparente, viraria um retângulo sólido. O GitPoulpi reconhece o iTerm2
  (`LC_TERMINAL`, `ITERM_SESSION_ID`, o tipo do cliente) e desenha caracteres.
- As **imagens do kitty** passam pela passagem direta DCS do tmux (kitty, Ghostty; vistas também
  no iTerm2 e no Rio em uma sondagem bruta). Elas exigem `set -g allow-passthrough on` (tmux
  3.3a ou mais recente) **e** `ui.image_protocol = "kitty"` no GitPoulpi, que então embrulha os
  envios e as exclusões. Sem a opção do tmux, o embrulho é descartado e o grafo fica em branco.
- As imagens inline do iTerm2 passam pelo tmux, mas o tmux não consegue posicioná-las: o
  GitPoulpi nunca as usa ali.

**Formato do ponteiro.** O GitPoulpi embrulha o `OSC 22` para a passagem direta. Ele chega à
janela no kitty e no foot com `allow-passthrough on`. O Ghostty não o mostra quando embrulhado
(a seta continua). WezTerm, Rio, Alacritty, Tabby e iTerm2 não têm `OSC 22` para mostrar.

**Área de transferência.** `set -g set-clipboard on`. O GitPoulpi também envia a forma embrulhada
para a passagem direta (funciona com `allow-passthrough on`). Por trás do kitty, Ghostty,
Alacritty e Tabby ela chega; o iTerm2 a nega até a permissão de área de transferência dele
ser ativada.

**Teclas.** O tmux não responde à consulta do teclado do kitty, e `extended-keys` não faz
`Ctrl+Enter` nem `Ctrl+,` passarem: medido com o tmux 3.8 dentro do kitty, os dois chegam como
`Enter` e `,` simples com `extended-keys` em off, on ou always. Atalhos de letras como
`Ctrl+Shift+Z` só chegam com `extended-keys` em on ou always **e**
`extended-keys-format csi-u` (tmux 3.5 ou mais recente) **e** um aplicativo que peça teclas
estendidas; o GitPoulpi nunca pede, então a configuração não muda nada aqui. Use as variantes
com `Alt` dentro do tmux
([Atalhos de teclado](keybindings.md#teclas-que-o-terminal-não-consegue-enviar-e-o-protocolo-de-teclado-do-kitty)).

**Configuração** (`~/.tmux.conf`):

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a ou posterior: formatos de ponteiro, imagens do kitty forçadas
```

**tmux dentro do WSL.** Não medido.

## zellij

Imagens: o zellij aceita o protocolo do kitty mas não os placeholders Unicode dele, e não leva
imagens sixel nem iTerm2, então o GitPoulpi desenha caracteres (um veto deliberado). Formato do
ponteiro: engolido, então o GitPoulpi não o envia. Área de transferência: o `OSC 52` é repassado
(funciona por trás do kitty, Ghostty, Alacritty, Tabby; o iTerm2 e o Terminal.app o negam ou o
ignoram). Teclas: o zellij responde à consulta do teclado do kitty, então o teclado completo
funciona.

O zellij 0.46 abre um popup "Welcome to Zellij" em uma partida nova: pressione `Esc`. Uma sessão
que terminou guarda o nome, e a próxima partida com o mesmo nome pergunta o que fazer.
Não é preciso configurar nada.

## GNU screen

Nada passa: as imagens e o `OSC 22` são impressos como caracteres ou descartados, o `OSC 52` é
ignorado e o teclado é o legado. O GitPoulpi desenha caracteres. O screen 4.00.03 (a versão que
o macOS traz) também embaralha cores e glifos largos na interface; o screen 5 foi testado só no
Linux. Use o tmux ou o zellij para ter uma sessão utilizável.

## herdr

O herdr embute um terminal que reemite **imagens do kitty**. O GitPoulpi as usa quando o
`config.toml` do herdr diz `kitty_graphics = true` (em `[terminal]`, ou em `[experimental]` nas
versões mais antigas) e o terminal em volta do herdr as desenha (kitty, Ghostty, WezTerm):
detectado, sem configuração. Se a configuração estiver em outro lugar (por exemplo sob
`HERDR_HOME`, que o GitPoulpi não lê), defina `ui.image_protocol = "kitty"`. Por SSH
(`herdr --remote`) o GitPoulpi não enxerga o cliente local, então desenha caracteres; veja
[Solução de problemas](troubleshooting.md#o-grafo-é-desenhado-com-caracteres-não-como-imagem-herdr-tmux-outros-terminais).
O `OSC 22` não é repassado pelo herdr 0.9.5, então o GitPoulpi não o envia. O `OSC 52` chega por
trás do kitty, Ghostty, Alacritty e Tabby. Na sondagem bruta, o herdr desenhou imagens do kitty
no kitty, no Ghostty e no iTerm2, também com a configuração padrão. Teclas: não medidas.

## psmux e tmux do MSYS2 (Windows)

- O **psmux** (Rust, ConPTY, a linguagem de comandos do tmux; `winget install psmux`) define
  `TMUX`, então o GitPoulpi desenha caracteres ali de propósito. Dentro do Windows Terminal ele
  roda normalmente e copiar funciona pela área de transferência do sistema; o ponteiro não muda
  (o Windows Terminal não tem `OSC 22`). Os outros terminais hospedeiros não foram medidos.
- O **tmux do MSYS2 ou do Cygwin** também define `TMUX` (caracteres de propósito), e no teste a
  tela ficou embaralhada: quadros antigos desenhados uns sobre os outros no mintty e no Windows
  Terminal (com e sem `-u`; sem ele, o desenho de caixas saiu como `qqqq`). Um programa nativo do
  Windows dentro de um pseudoterminal do MSYS2 não é redesenhado corretamente pelo tmux. Isso não
  tem conserto no GitPoulpi: use o psmux, os painéis do Windows Terminal ou os do WezTerm.
- zellij e screen não têm versão para Windows. O herdr 0.9.5 foi instalado mas não executado.

## Painéis de terminal

Os painéis que pertencem ao próprio terminal não são multiplexadores para o GitPoulpi:

- **Painéis do WezTerm** (`wezterm cli split-pane`, `wezterm connect`, domínios SSH): o WezTerm
  desenha tudo, então as imagens funcionam (grafo e avatares no painel) e o ponteiro se comporta
  como no próprio WezTerm (os formatos funcionam em uma versão com o PR upstream #8092 do
  WezTerm; o divisor do painel dá o ponteiro de redimensionamento). Não medido no Linux nem no
  macOS.
- **Painéis do Windows Terminal** (`wt ; split-pane`): o grafo e os avatares em sixel funcionam
  no painel, copiar funciona, sem formatos de ponteiro.
- **Divisões do kitty, do Ghostty e do iTerm2**: nativas, como o terminal. Não medidas no macOS
  para kitty e Ghostty (precisam de GPU).

## `tmux -CC` do iTerm2 (modo de controle)

O iTerm2 abre uma janela nativa para cada janela do tmux e mantém uma janela de gateway
"[tmux bash]". `TMUX` está definido, então o GitPoulpi desenha caracteres na janela nativa. As
imagens chegam ao iTerm2 pela conexão de controle (o diálogo "Allow Terminal-Initiated Display?"
apareceu em um teste de `OSC 1337`), mas o GitPoulpi não as usa ali. A carga do kitty de um
`ui.image_protocol = "kitty"` forçado é impressa na janela de gateway, então deixe em `auto`.
`OSC 22` e `OSC 52` não chegam.

## Configurações que vale conhecer

| Configuração | Uso |
|---|---|
| `ui.image_protocol` | `kitty` força imagens do kitty por um multiplexador que as repassa (tmux com `allow-passthrough on`, herdr com uma configuração que o GitPoulpi não lê); `none` para caracteres. |
| `ui.graph_renderer` | `text` está sempre correto. |
| `ui.kitty_keyboard` | `off` se um multiplexador se comportar mal com a consulta de teclado. |

Veja [Escolhendo um terminal](terminals.md) para o terminal por trás do multiplexador, e
[Solução de problemas](troubleshooting.md) quando o grafo é desenhado com caracteres.
