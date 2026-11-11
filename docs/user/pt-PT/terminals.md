[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · **Português (Portugal)** · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# Escolher um terminal

O GitPoulpi corre em qualquer terminal com suporte de rato, mas fica melhor e mais
agradável de usar num que desenhe imagens, mostre cor verdadeira e envie o teclado
completo. Esta página ordena os terminais por sistema operativo, diz o que cada um
lhe dá e que definição alterar. Os factos foram confirmados na documentação, no
registo de alterações ou no gestor de problemas de cada projeto em outubro de 2026
(veja [Fontes](#fontes)); nem tudo aqui foi testado em todos os terminais.
Tudo o que está marcado como **(não verificado)** não pôde ser confirmado numa fonte
primária.

## Melhores escolhas

| SO | Melhores escolhas, por ordem | Porquê |
|---|---|---|
| **Windows** | **WezTerm**, depois **Windows Terminal** | O WezTerm desenha o grafo e os avatares como imagens, com transparência. O Windows Terminal tem a melhor integração com o SO (é a predefinição do lançador), desenha os avatares como imagens sixel e o grafo como carateres. |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, depois **foot** | O kitty e o Ghostty dão imagens e o teclado completo. O WezTerm também dá imagens. O foot desenha os avatares como imagens sixel, é rápido e tem o teclado completo. No Omarchy, escolha o Ghostty ou o kitty para ter imagens. |
| **macOS** | **kitty**, **Ghostty**, **WezTerm**, depois **iTerm2** | Pela mesma razão. O Terminal.app funciona, mas não tem imagens nem cor verdadeira. |

Em qualquer SO, o **tmux, o zellij e o screen** escondem do GitPoulpi o protocolo de
imagens do terminal, pelo que ele desenha carateres (veja
[Multiplexadores](#multiplexadores)).

## O que o GitPoulpi usa

- **Imagens** (coluna do grafo, avatares dos autores, diff de imagens e vista de
  ficheiro): o **protocolo gráfico do kitty** com marcadores Unicode, as **imagens
  em linha do iTerm2** (`OSC 1337`) ou o **sixel**, por esta ordem de preferência. A
  deteção recusa por omissão, a partir de `TERM`, `TERM_PROGRAM`, `KITTY_WINDOW_ID`,
  `GHOSTTY_RESOURCES_DIR`, `WEZTERM_EXECUTABLE`, `WT_SESSION`, `KONSOLE_VERSION` e das
  variáveis dos multiplexadores, e nunca lê o teclado (por isso também não pede ao
  terminal os seus atributos de dispositivo). O kitty e o Ghostty usam o protocolo do
  kitty. O WezTerm usa o protocolo do iTerm2 (de propósito) e mistura a transparência
  do PNG. O iTerm2, o Rio e o mintty usam o protocolo do iTerm2 com um fundo opaco. O
  Windows Terminal, o foot, o Konsole e o mlterm usam **sixel**. Sem nenhum deles, os
  avatares são o emblema com as iniciais.
- **Sixel** é desenhado para a **coluna do grafo** (faixas, nós e, com `ui.avatars = "graph"`, os avatares dentro dos círculos dos commits) e, nos outros sítios, para os **avatares** (Log, detalhe do commit, Histórico do ficheiro, Blame, vista de imagem). Um sixel não tem alfa, ids nem escala, pelo que a imagem tem exatamente `colunas x largura da célula` por `linhas x altura da célula` píxeis, desenhada só onde há algo, sobre células vazias que mantêm o fundo; a imagem codificada fica em cache, é recodificada só quando as faixas, a deslocação ou o zoom mudam e é reenviada só quando uma célula sob ela foi repintada. Um sixel não se apaga: o GitPoulpi escreve texto por cima das suas células para o remover. O sixel precisa do tamanho real da célula em píxeis: o Windows Terminal é consultado uma vez no arranque e após um zoom (`CSI 16 t`, resposta em cerca de 10 ms, nunca bloqueia se ficar calado); noutros sítios vem do tamanho da janela ou de `GITPOULPI_TERMINAL_CELL_PX=10x20`. Um terminal que desenha sixel mas não é reconhecido (um terminal do VS Code com `terminal.integrated.enableImages`, um xterm iniciado com `-ti vt340`) precisa de `ui.image_protocol = "sixel"`. O Windows Terminal anterior à 1.22 não desenha sixel; defina aí `ui.image_protocol = "none"` (o grafo continua em carateres).
- **Cor verdadeira**: os temas são RGB. `ui.color_depth = "auto"` desce para 256 cores
  num `TERM` `*-256color` sem `COLORTERM=truecolor` (e no Terminal.app), e para 16
  cores na consola do Linux. O kitty, o WezTerm e o Windows Terminal contam como cor
  verdadeira.
- **Rato**: relatório de rato SGR com botões, arrastar e movimento (realce ao passar,
  arrastar e largar, clique com o botão direito, clique duplo).
- **Teclado**: o **protocolo de teclado do kitty** (só a opção de desambiguação) é
  pedido no arranque, com um limite de 100 ms. Faz chegar `Ctrl+,`, `Ctrl+Enter`,
  `Ctrl+Shift+N` e `Ctrl+Shift+Z`. Sem ele ficam as teclas clássicas (`Ctrl+Y` refaz,
  as variantes com `Alt` substituem alguns acordes). Veja
  [Atalhos de teclado](keybindings.md#teclas-que-o-terminal-não-consegue-enviar-e-o-protocolo-de-teclado-do-kitty).
- **Área de transferência**: as ações de copiar escrevem `OSC 52`.
- **Forma do ponteiro do rato** (ponteiro de redimensionar sobre os divisores, mão
  sobre os itens clicáveis): `OSC 22`, um extra onde está implementado.
- Os **eventos de foco** estão ativados (as dicas fecham quando a janela perde o foco).
  O GitPoulpi não pede colagem entre parênteses (bracketed paste), hiperligações OSC 8
  nem sublinhado ondulado.
  Só no caminho das imagens, cada fotograma vai envolvido numa atualização sincronizada
  (modo 2026, ignorado pelos terminais que não o conhecem) para que uma imagem nunca
  pisque por cima de um popup.
- **Emoji e símbolos**: o GitPoulpi só usa glifos com apresentação de emoji por
  omissão ou formas BMP de largura simples, porque os terminais e os tipos de letra
  discordam quanto aos seletores de variação.

## Tabela de capacidades

«Imagens» é o que o GitPoulpi desenha hoje nesse terminal (protocolo **kitty**,
**iTerm2** ou **sixel**, ou **texto**), não o que o terminal poderia fazer.

| Terminal | SO | Imagens (grafo, avatares, diff de imagens) | Cor verdadeira | Teclado kitty (acordes completos) | Cópia OSC 52 | Ponteiro OSC 22 |
|---|---|---|---|---|---|---|
| **kitty** (0.49) | Linux, macOS | kitty | sim | sim | sim | sim |
| **Ghostty** (1.3) | Linux, macOS (sem versão para Windows) | kitty | sim | sim | sim | sim, desde a 1.0 |
| **WezTerm** | Windows, Linux, macOS | iTerm2, com transparência | sim | só com `enable_kitty_keyboard = true` (predefinição `false`) | sim (só escrita) | não (o pull request ainda está aberto) |
| **foot** (1.28) | Linux (Wayland) | sixel (grafo e avatares; sem gráficos kitty) | sim | sim, desde a 1.10.3 | sim (`security.osc52`) | sim, desde a 1.12 |
| **Alacritty** (0.17) | todos | texto (sem suporte de imagens no projeto original) | sim | sim, desde a 0.13 | sim (escrita; colagem desligada por predefinição) | não (não implementado) |
| **Konsole** | Linux | sixel (grafo e avatares) | sim | a ser acrescentado em 2026 (merge request !1188); versão (não verificado) | só escrita, integrado em meados de 2024 | (não verificado) |
| **GNOME Terminal, Ptyxis** (VTE) | Linux | texto | sim | não (não está numa versão publicada do VTE) | não (o VTE ignora-o) | não |
| **iTerm2** (3.7) | macOS | iTerm2, fundo opaco | sim | sim, desde a 3.6.9 (veja Profiles > Keys) | desligado por predefinição: ative «Applications in terminal may access clipboard» | (não verificado) |
| **Terminal.app** | macOS | texto | não (256 cores) | não | (não verificado) | (não verificado) |
| **Windows Terminal** (1.25) | Windows | sixel (grafo e avatares) | sim | sim, estável desde a 1.25 (out. 2026); Preview desde a 1.25 (mar. 2026) | sim (escrita; só com foco) | não (não implementado) |
| **Rio** | Windows, Linux, macOS | iTerm2, fundo opaco (também fala gráficos kitty) | sim | sim (ativado por predefinição) | sim | não (não implementado) |
| **Warp** | macOS, Linux, Windows | texto (não detetado) | sim | sim | sim (respeita a sequência) | (não verificado) |
| **Terminal do VS Code** (xterm.js) | todos | texto (sixel e imagens iTerm2 existem atrás de `terminal.integrated.enableImages`; `ui.image_protocol = "sixel"` usa o sixel) | sim | (não verificado) | (não verificado) | não |
| **mintty** (Cygwin, Git Bash) | Windows | iTerm2, fundo opaco | sim | (não verificado; não documentado) | (não verificado; não documentado) | sim |
| **Consola do Windows** (conhost, cmd, janela do PowerShell) | Windows | texto | só Windows 10/11 recentes | não é preciso: comunica os modificadores por si | não | não |
| **ConEmu** | Windows | texto | (não verificado) | (não verificado) | (não verificado) | (não verificado) |
| **tmux, zellij, screen** | todos | texto, sempre | precisa de definições | precisa de definições | precisa de definições | não |

As células «não (não implementado)» do OSC 22 vêm da ausência no registo de
alterações e na documentação de um projeto, mais relatos de utilizadores, e não de um
teste; leia-as como «não documentado». A página não tabela a largura dos emoji e dos
seletores de variação: difere consoante o terminal e o tipo de letra, e é por isso que
o GitPoulpi evita glifos VS16 (veja [O que o GitPoulpi usa](#o-que-o-gitpoulpi-usa)).

## Terminal a terminal

### Windows

- **WezTerm.** O melhor suporte de imagens no Windows. Deixe `ui.image_protocol` em
  `auto`. No `wezterm.lua`, defina `config.enable_kitty_keyboard = true` para ter os
  acordes completos e um tipo de letra com alternativa para emoji, por exemplo
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  A última versão estável com etiqueta do WezTerm é a 20240203-110809-5046fc22 (fev.
  2024); as correções mais recentes chegam nas versões noturnas. O WezTerm ainda não
  tem a forma do ponteiro OSC 22.
- **Windows Terminal.** Cor verdadeira, rato, OSC 52 e bons emoji. O GitPoulpi desenha
  os avatares como imagens sixel (Windows Terminal 1.22 ou posterior); o grafo é de
  carateres. Escolha uma Nerd Font em *Definições > Perfis >
  Predefinições > Aspeto > Tipo de letra*. Mantenha `Shift` premido para selecionar
  texto enquanto o rato está capturado. A forma do ponteiro não muda. Numa versão
  anterior à 1.25 (sem o protocolo de teclado do kitty), use as variantes com `Alt`.
- O **Rio** funciona com um fundo opaco para as imagens; menos testado.
- O **mintty** (Git Bash, MSYS2, Cygwin) desenha imagens através do protocolo do iTerm2
  com um fundo opaco; menos testado.
- **Consola do Windows, ConEmu, cmd**: evite. As imagens são mal desenhadas ou nem
  aparecem. Um clique duplo no exe volta a abrir o GitPoulpi no terminal que escolheu
  (*Definições > Lançador*).

### Linux

- **kitty.** Tudo funciona sem definições. `Shift+clique` é do próprio kitty, por isso
  use `Alt+clique` para alargar uma seleção ([Rato](mouse-and-gestures.md)).
- **Ghostty.** Tudo funciona sem definições. Se os ícones aparecerem como buracos,
  acrescente um tipo de letra com emoji ou uma Nerd Font a `font-family` (repita a
  chave para acrescentar alternativas).
- **WezTerm.** Imagens através do protocolo do iTerm2, transparência misturada. A mesma
  nota sobre `enable_kitty_keyboard` que no Windows.
- **foot.** Grafo de carateres, teclado completo, rápido. Os avatares são imagens
  sixel, o grafo é de carateres. Defina `font=JetBrainsMono Nerd Font:size=11` no
  `foot.ini`.
- **Alacritty.** Grafo de carateres, teclado kitty desde a 0.13. O Alacritty não tem
  suporte de imagens.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Funcionam com carateres e teclas
  clássicas: use as variantes com `Alt` e `Ctrl+Y` para refazer (um Konsole recente pode
  acrescentar o protocolo de teclado do kitty). As ações de copiar do VTE (`OSC 52`) não
  põem nada na área de transferência. Um método de introdução (IBus, Fcitx) pode apanhar
  o `Ctrl+Space` que alterna o stage - **(não verificado)**; reatribua-o em
  *Definições > Teclado*.
- **Wayland, Hyprland, Omarchy.** A dica do ponteiro do rato (`OSC 22`) funciona no
  kitty, no Ghostty e no foot. A entrada do lançador do Omarchy passa pelo
  `xdg-terminal-exec`.

### macOS

- **kitty, Ghostty, WezTerm**: como no Linux.
- **iTerm2.** Imagens com fundo opaco (um retângulo visível se o seu fundo for
  diferente do do tema; `ui.graph_renderer = "text"` remove-o). Ative
  *Settings > General > Selection > Applications in terminal may access clipboard* para
  copiar, e *Profiles > Keys > Left Option key: Esc+* para os acordes com `Alt`.
- **Terminal.app.** Só 256 cores: o GitPoulpi escolhe sozinho `ui.color_depth = 256` e
  os temas ficam mais baços. Sem imagens. Ative *Use Option as Meta key* para os
  acordes com `Alt`.

### VS Code e Warp

O terminal do VS Code (`TERM_PROGRAM=vscode`) e o Warp não são reconhecidos como
terminais de imagens, pelo que o GitPoulpi desenha carateres. Ambos são utilizáveis. No
VS Code, ative `terminal.integrated.enableImages` e defina
`ui.image_protocol = "sixel"` para ter imagens nos avatares. O
Warp documenta o suporte do protocolo de teclado do kitty; o do terminal do VS Code é
**(não verificado)**.

## Multiplexadores

O GitPoulpi deteta o tmux, o zellij, o screen e o herdr (`TMUX`, `ZELLIJ`, `STY`,
`TERM=screen*` ou `tmux*`, `HERDR_*`) e desenha **carateres**, porque um multiplexador
reencaminha células, não sequências de imagens. Para as outras funcionalidades, no
`~/.tmux.conf`:

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g extended-keys on                # full chords (tmux 3.2 or later)
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a ou posterior; só se forçar imagens, veja abaixo
```

Para forçar imagens através de um multiplexador que as reencaminhe, defina
`ui.image_protocol` como `kitty` ou `iterm2`. O
herdr com `kitty_graphics = true` dentro do kitty, do Ghostty ou do WezTerm é detetado
automaticamente; veja
[Resolução de problemas](troubleshooting.md#o-grafo-é-desenhado-com-carateres-não-como-imagem-herdr-tmux-outros-terminais).
`GITPOULPI_TERMINAL_IMAGE=off` desliga todas as imagens.

O tmux não reencaminha o protocolo de teclado do kitty por si: `extended-keys` faz com
que envie as teclas modificadas como sequências `CSI u` ou `xterm`. O zellij 0.41 e
posteriores fala o protocolo de teclado do kitty e reencaminha o OSC 52, segundo o seu
registo de alterações.

## O que se perde, e como

| Falta | O que obtém em vez disso |
|---|---|
| Protocolo de imagens | Grafo de carateres. O avatar passa a ser o emblema colorido com as iniciais (uma letra), diga o que disser `ui.avatar_fallback`: um identicon espremido em células de texto é irreconhecível. O diff de imagens e a vista de ficheiro usam meios-blocos coloridos, ou uma ficha de informações. |
| Cor verdadeira | As cores são mapeadas para a paleta de 256 ou de 16 cores (`ui.color_depth`). |
| Protocolo de teclado do kitty | Teclas clássicas; alguns acordes chegam como outra tecla. As Definições avisam, e você reatribui ou usa as variantes com `Alt`. |
| OSC 52 | As ações de copiar não põem nada na área de transferência: selecione com o rato (mantenha `Shift` premido) e use a cópia do próprio terminal. |
| OSC 22 | O ponteiro não muda; o divisor ou o botão sob o rato continua a acender. |
| Tipo de letra com emoji | Caixas ou buracos em vez de ícones; o grafo e o texto continuam a funcionar. Instale uma Nerd Font ou um tipo de letra com emoji e indique-o como alternativa. |

## Definições a conhecer

| Definição | Utilização |
|---|---|
| `ui.image_protocol` | `auto` (predefinição), `kitty`, `iterm2`, `sixel`, `none`. Use `none` se vir lixo onde deviam estar imagens. Reinicie. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` está sempre certo. Reinicie. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | `auto` ou `off`, se um terminal ou multiplexador se portar mal. Reinicie. |
| `ui.mouse_enabled` | Desligado, deixa o rato ao terminal. Reinicie. |
| `ui.launch_terminal` | O terminal que os atalhos do SO abrem ([Definições](settings.md)). |

Veja também [Imagens](image-view.md), [Atalhos de teclado](keybindings.md) e
[Resolução de problemas](troubleshooting.md).

## Fontes

Verificadas em outubro de 2026. Fontes primárias, salvo indicação em contrário.

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [pointer shapes](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sequences](https://wezterm.org/escape-sequences.html), [OSC 22 pull request](https://github.com/wezterm/wezterm/pull/6292), [releases](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [releases](https://github.com/alacritty/alacritty/releases)
- Konsole: [OSC 52 merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [kitty keyboard merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [OSC 52 issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [kitty keyboard issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [downloads](https://iterm2.com/downloads.html); teclado kitty a partir da [matriz do terminfo.dev](https://terminfo.dev/extensions/kitty-keyboard-protocol) (secundária)
- Windows Terminal: [anúncio da 1.22, sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (secundária), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [features](https://rioterm.com/docs/features), [kitty keyboard](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [full-screen apps](https://docs.warp.dev/terminal/more-features/full-screen-apps); OSC 52 a partir de uma [análise de um aviso de segurança do OSC 52 no Warp](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) (secundária)
- VS Code: [advanced terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [control sequences](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tips](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (sem cor verdadeira): [discussão da Apple](https://discussions.apple.com/thread/255315820) (secundária)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES); zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Sites de comparação (secundárias, usadas apenas para confirmar): [Terminal Trove](https://terminaltrove.com/compare/terminals/)

Continuam por verificar: o sixel no Konsole e no mlterm (tirado das listas de funcionalidades dos próprios projetos, não reconfirmado numa versão, e nunca experimentado numa janela real em nenhum terminal); o OSC 22 no Konsole, no iTerm2, no Terminal.app e no Warp; o
OSC 52 no Terminal.app, no VS Code e no mintty; o protocolo de teclado do kitty no VS
Code, no mintty e no ConEmu; todas as células do ConEmu; e a largura dos emoji por
terminal. A versão em que o Konsole traz o protocolo de teclado do kitty não foi
confirmada.
