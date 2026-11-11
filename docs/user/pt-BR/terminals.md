[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# Escolhendo um terminal

O GitPoulpi roda em qualquer terminal com suporte a mouse, mas fica melhor e mais
agradável de usar em um que desenhe imagens, mostre true color e envie o teclado
completo. Esta página classifica os terminais por sistema operacional, diz o que cada
um oferece e qual configuração mudar. Os fatos foram conferidos na documentação, no
changelog ou no rastreador de issues de cada projeto em outubro de 2026 (veja
[Fontes](#fontes)); nada aqui foi testado em todos os terminais. Tudo o
que está marcado como **(não verificado)** não pôde ser confirmado em uma fonte
primária.

## Melhores opções

| SO | Melhores opções, em ordem | Por quê |
|---|---|---|
| **Windows** | **WezTerm**, depois **Windows Terminal** | O WezTerm desenha o grafo e os avatares como imagens, com transparência. O Windows Terminal tem a melhor integração com o SO (é o padrão do lançador), ele desenha o grafo e os avatares como imagens sixel (sem transparência). |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, depois **foot** | kitty e Ghostty oferecem imagens e o teclado completo. O WezTerm também oferece imagens. O foot desenha o grafo e os avatares como imagens sixel, é rápido e tem o teclado completo. No Omarchy, escolha Ghostty ou kitty para ter imagens. |
| **macOS** | **kitty**, **Ghostty**, **WezTerm**, depois **iTerm2** | Mesmo raciocínio. O Terminal.app funciona, mas não tem imagens nem true color. |

Em qualquer SO, **tmux, zellij e screen** escondem do GitPoulpi o protocolo de
imagens do terminal, então ele desenha caracteres (veja
[Multiplexadores](#multiplexadores)).

## O que o GitPoulpi usa

- **Imagens** (coluna do grafo, avatares dos autores, diff de imagem e Visualização
  de arquivo): o **protocolo gráfico do kitty** com placeholders Unicode, as
  **imagens inline do iTerm2** (`OSC 1337`) ou **sixel**, nessa ordem de preferência. A
  detecção é negar por padrão, a partir de `TERM`, `TERM_PROGRAM`,
  `KITTY_WINDOW_ID`, `GHOSTTY_RESOURCES_DIR`, `WEZTERM_EXECUTABLE`, `WT_SESSION`,
  `KONSOLE_VERSION` e das variáveis dos multiplexadores, e nunca lê o teclado (então
  também não pergunta ao terminal pelos atributos do dispositivo). kitty e Ghostty
  usam o protocolo do kitty. O WezTerm usa o protocolo do iTerm2 (de propósito) e
  mistura a transparência do PNG. iTerm2, Rio e mintty usam o protocolo do iTerm2 com
  um fundo opaco. Windows Terminal, foot, Konsole e mlterm usam **sixel**. Sem nenhum
  deles, os avatares são o selo com as iniciais.
- **Sixel** é desenhado para a **coluna do grafo** (faixas, nós e, com `ui.avatars = "graph"`, os avatares dentro dos círculos dos commits) e, nos demais lugares, para os **avatares** (Log, detalhe do commit, Histórico do arquivo, Blame, a visualização de imagem). Um sixel não tem alfa, ids nem escala, então a imagem tem exatamente `colunas x largura da célula` por `linhas x altura da célula` pixels, desenhada só onde há algo (bordas suavizadas abaixo de meia opacidade são descartadas), sobre células vazias que mantêm o fundo; a imagem codificada fica em cache, é recodificada só quando faixas, rolagem ou zoom mudam e é reenviada só quando uma célula sob ela foi repintada. Um sixel não tem como ser apagado: o GitPoulpi escreve texto sobre as suas células para removê-lo. O sixel precisa do tamanho real da célula em pixels: o Windows Terminal é consultado uma vez na partida e após um zoom (`CSI 16 t`, resposta em cerca de 10 ms, nunca bloqueia se ficar calado); em outros lugares vem do tamanho da janela ou de `GITPOULPI_TERMINAL_CELL_PX=10x20`. Um terminal que desenha sixel mas não é reconhecido (um terminal do VS Code com `terminal.integrated.enableImages`, um xterm iniciado com `-ti vt340`) precisa de `ui.image_protocol = "sixel"`. O Windows Terminal anterior à 1.22 não desenha sixel; defina `ui.image_protocol = "none"` nele (o grafo então fica em caracteres, como em todo terminal sem imagens).
- **True color**: os temas são RGB. `ui.color_depth = "auto"` cai para 256 cores em
  um `TERM` `*-256color` sem `COLORTERM=truecolor` (e no Terminal.app), e para 16
  cores no console do Linux. kitty, WezTerm e Windows Terminal contam como true color.
- **Mouse**: relatório de mouse SGR com botões, arrastar e movimento (feedback ao
  passar o mouse, arrastar e soltar, clique com o botão direito, clique duplo).
- **Teclado**: o **protocolo de teclado do kitty** (somente a flag de desambiguação)
  é solicitado na inicialização, com um timeout de 100 ms. Ele faz `Ctrl+,`,
  `Ctrl+Enter`, `Ctrl+Shift+N` e `Ctrl+Shift+Z` chegarem. Sem ele, as teclas
  legadas continuam (`Ctrl+Y` refaz, variantes com `Alt` substituem alguns atalhos).
  Veja
  [Atalhos de teclado](keybindings.md#teclas-que-o-terminal-não-consegue-enviar-e-o-protocolo-de-teclado-do-kitty).
- **Área de transferência**: as ações de copiar escrevem `OSC 52`.
- **Formato do ponteiro do mouse** (ponteiro de redimensionamento sobre os divisores,
  mão sobre itens clicáveis): `OSC 22`, um bônus onde está implementado.
- **Eventos de foco** estão ativados (as dicas fecham quando a janela perde o foco).
  O GitPoulpi não solicita colar entre colchetes (bracketed paste), hiperlinks OSC 8
  nem sublinhado ondulado. Somente no caminho das imagens, cada quadro é envolvido em
  uma atualização sincronizada (modo 2026, ignorado pelos terminais que não o
  conhecem) para que uma imagem nunca pisque sobre um popup.
- **Emojis e símbolos**: o GitPoulpi só usa glifos com apresentação de emoji padrão
  ou formas BMP de largura simples, porque terminais e fontes discordam sobre os
  seletores de variação.

## Tabela de recursos

“Imagens” é o que o GitPoulpi desenha nesse terminal hoje (protocolo **kitty**, **iTerm2**
ou **sixel**, ou **texto**), não o que o terminal poderia fazer.

| Terminal | SO | Imagens (grafo, avatares, diff de imagem) | True color | Teclado kitty (atalhos completos) | Cópia OSC 52 | Ponteiro OSC 22 |
|---|---|---|---|---|---|---|
| **kitty** (0.49) | Linux, macOS | kitty | sim | sim | sim | sim |
| **Ghostty** (1.3) | Linux, macOS (sem versão para Windows) | kitty | sim | sim | sim | sim, desde a 1.0 |
| **WezTerm** | Windows, Linux, macOS | iTerm2, com transparência | sim | só com `enable_kitty_keyboard = true` (padrão `false`) | sim (somente escrita) | não (o pull request ainda está aberto) |
| **foot** (1.28) | Linux (Wayland) | sixel (grafo e avatares; sem gráficos do kitty) | sim | sim, desde a 1.10.3 | sim (`security.osc52`) | sim, desde a 1.12 |
| **Alacritty** (0.17) | todos | texto (sem suporte a imagens no projeto) | sim | sim, desde a 0.13 | sim (escrita; colar desativado por padrão) | não (não implementado) |
| **Konsole** | Linux | sixel (grafo e avatares) | sim | sendo adicionado em 2026 (merge request !1188); versão (não verificado) | somente escrita, integrado em meados de 2024 | (não verificado) |
| **GNOME Terminal, Ptyxis** (VTE) | Linux | texto | sim | não (não está em uma versão lançada do VTE) | não (o VTE ignora) | não |
| **iTerm2** (3.7) | macOS | iTerm2, fundo opaco | sim | sim, desde a 3.6.9 (confira Profiles > Keys) | desativado por padrão: ative “Applications in terminal may access clipboard” | (não verificado) |
| **Terminal.app** | macOS | texto | não (256 cores) | não | (não verificado) | (não verificado) |
| **Windows Terminal** (1.25) | Windows | sixel (grafo e avatares) | sim | sim, estável desde a 1.25 (out. 2026); Preview a partir da 1.25 (mar. 2026) | sim (escrita; só com foco) | não (não implementado) |
| **Rio** | Windows, Linux, macOS | iTerm2, fundo opaco (também fala os gráficos do kitty) | sim | sim (ativado por padrão) | sim | não (não implementado) |
| **Warp** | macOS, Linux, Windows | texto (não detectado) | sim | sim | sim (respeita a sequência) | (não verificado) |
| **Terminal do VS Code** (xterm.js) | todos | texto (imagens sixel e iTerm2 existem atrás de `terminal.integrated.enableImages`; `ui.image_protocol = "sixel"` usa o sixel) | sim | (não verificado) | (não verificado) | não |
| **mintty** (Cygwin, Git Bash) | Windows | iTerm2, fundo opaco | sim | (não verificado; não documentado) | (não verificado; não documentado) | sim |
| **Console do Windows** (conhost, cmd, janela do PowerShell) | Windows | texto | só em versões recentes do Windows 10/11 | desnecessário: ele informa os modificadores sozinho | não | não |
| **ConEmu** | Windows | texto | (não verificado) | (não verificado) | (não verificado) | (não verificado) |
| **tmux, zellij, screen** | todos | texto, sempre | precisa de configuração | precisa de configuração | precisa de configuração | não |

As células “não (não implementado)” do OSC 22 vêm da ausência no changelog e na
documentação de um projeto, somados a relatos de usuários, e não de um teste; trate-as
como “não documentado”. A página não tabula a largura de emojis e de seletores de
variação: ela difere por terminal e por fonte, e é por isso que o GitPoulpi evita
glifos VS16 (veja [O que o GitPoulpi usa](#o-que-o-gitpoulpi-usa)).

## Por terminal

### Windows

- **WezTerm.** O melhor suporte a imagens no Windows. Deixe `ui.image_protocol` em
  `auto`. No `wezterm.lua`, defina `config.enable_kitty_keyboard = true` para ter os
  atalhos completos e uma fonte com fallback de emoji, por exemplo
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  A última versão estável com tag do WezTerm é a 20240203-110809-5046fc22 (fev. de
  2024); correções mais novas saem nas nightlies. O WezTerm ainda não tem o formato
  de ponteiro OSC 22.
- **Windows Terminal.** True color, mouse, OSC 52 e bons emojis. O GitPoulpi desenha o
  grafo e os avatares como imagens sixel (Windows Terminal 1.22 ou mais recente). Escolha uma Nerd Font em *Settings > Profiles >
  Defaults > Appearance > Font face*. Segure `Shift` para selecionar texto enquanto o
  mouse está capturado. O formato do ponteiro não muda. Em uma versão anterior à 1.25
  (sem o protocolo de teclado do kitty), use as variantes com `Alt`.
- **Rio** funciona com um fundo opaco para as imagens; menos testado.
- **mintty** (Git Bash, MSYS2, Cygwin) desenha imagens pelo protocolo do iTerm2 com
  um fundo opaco; menos testado.
- **Console do Windows, ConEmu, cmd**: evite. As imagens são desenhadas mal ou nem
  aparecem. Um clique duplo no exe reabre o GitPoulpi no terminal que você escolheu
  (*Configurações > Lançador*).

### Linux

- **kitty.** Tudo funciona sem nenhuma configuração. `Shift+clique` é do próprio
  kitty, então use `Alt+clique` para estender uma seleção
  ([Mouse](mouse-and-gestures.md)).
- **Ghostty.** Tudo funciona sem nenhuma configuração. Se os ícones aparecerem como
  buracos, adicione uma fonte de emoji ou uma Nerd Font a `font-family` (repita a
  chave para adicionar fallbacks).
- **WezTerm.** Imagens pelo protocolo do iTerm2, com a transparência misturada. A
  mesma observação sobre `enable_kitty_keyboard` do Windows.
- **foot.** Grafo e avatares em sixel, teclado completo, rápido. Defina `font=JetBrainsMono Nerd Font:size=11`
  no `foot.ini`.
- **Alacritty.** Grafo de caracteres, teclado do kitty desde a 0.13. O Alacritty não
  tem suporte a imagens.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Funcionam com caracteres e teclas
  legadas: use as variantes com `Alt` e `Ctrl+Y` para refazer (um Konsole recente pode
  adicionar o protocolo de teclado do kitty). As ações de copiar do VTE (`OSC 52`) não
  colocam nada na área de transferência. Um método de entrada (IBus, Fcitx) pode
  capturar o `Ctrl+Space` que alterna o stage - **(não verificado)**; reatribua-o em
  *Configurações > Teclado e mouse*.
- **Wayland, Hyprland, Omarchy.** A dica do ponteiro do mouse (`OSC 22`) funciona no
  kitty, no Ghostty e no foot. A entrada do lançador no Omarchy passa por
  `xdg-terminal-exec`.

### macOS

- **kitty, Ghostty, WezTerm**: como no Linux.
- **iTerm2.** Imagens com um fundo opaco (um retângulo visível se o seu fundo for
  diferente do do tema; `ui.graph_renderer = "text"` o remove). Ative
  *Settings > General > Selection > Applications in terminal may access clipboard*
  para copiar, e *Profiles > Keys > Left Option key: Esc+* para os atalhos com `Alt`.
- **Terminal.app.** Somente 256 cores: o GitPoulpi escolhe `ui.color_depth = 256`
  sozinho e os temas ficam mais sem graça. Sem imagens. Marque *Use Option as Meta
  key* para os atalhos com `Alt`.

### VS Code e Warp

O terminal do VS Code (`TERM_PROGRAM=vscode`) e o Warp não são reconhecidos como
terminais de imagens, então o GitPoulpi desenha caracteres. Os dois são utilizáveis.
No VS Code, ative `terminal.integrated.enableImages` e defina
`ui.image_protocol = "sixel"` para ter imagens nos avatares. O Warp documenta o suporte ao protocolo de teclado do kitty; o suporte do terminal do
VS Code é **(não verificado)**.

## Multiplexadores

O GitPoulpi detecta tmux, zellij, screen e herdr (`TMUX`, `ZELLIJ`, `STY`,
`TERM=screen*` ou `tmux*`, `HERDR_*`) e desenha **caracteres**, porque um
multiplexador repassa células, não sequências de imagem. Para os outros recursos, no
`~/.tmux.conf`:

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g extended-keys on                # acordes completos (tmux 3.2 ou posterior)
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a ou posterior; só se você forçar imagens, veja abaixo
```

Para forçar imagens por um multiplexador que as repassa, defina `ui.image_protocol`
como `kitty` ou `iterm2`. O herdr com
`kitty_graphics = true` dentro do kitty, do Ghostty ou do WezTerm é detectado
automaticamente; veja
[Solução de problemas](troubleshooting.md#o-grafo-é-desenhado-com-caracteres-não-como-imagem-herdr-tmux-outros-terminais).
`GITPOULPI_TERMINAL_IMAGE=off` desativa todas as imagens.

O tmux não repassa sozinho o protocolo de teclado do kitty: `extended-keys` o faz
enviar as teclas com modificadores como sequências `CSI u` ou `xterm`. O zellij 0.41 e
mais recentes fala o protocolo de teclado do kitty e repassa o OSC 52, segundo o
changelog dele.

## O que se perde, e como

| Falta | O que você recebe no lugar |
|---|---|
| Protocolo de imagens | Grafo de caracteres. O avatar vira o selo colorido de iniciais (uma letra), seja qual for o valor de `ui.avatar_fallback`: um identicon espremido em células de texto fica irreconhecível. O diff de imagem e a Visualização de arquivo usam meios-blocos coloridos, ou um cartão de informações. |
| True color | As cores são mapeadas para a paleta de 256 ou de 16 cores (`ui.color_depth`). |
| Protocolo de teclado do kitty | Teclas legadas; alguns atalhos chegam como outra tecla. As Configurações avisam, e você reatribui ou usa as variantes com `Alt`. |
| OSC 52 | As ações de copiar não colocam nada na área de transferência: selecione com o mouse (segurando `Shift`) e use o copiar do próprio terminal. |
| OSC 22 | O ponteiro não muda; o divisor ou botão sob o mouse ainda se acende. |
| Fonte de emoji | Caixas ou buracos no lugar dos ícones; o grafo e o texto continuam funcionando. Instale uma Nerd Font ou uma fonte de emoji e liste-a como fallback. |

## Configurações que vale conhecer

| Configuração | Uso |
|---|---|
| `ui.image_protocol` | `auto` (padrão), `kitty`, `iterm2`, `sixel`, `none`. Use `none` se aparecer lixo onde deveriam estar as imagens. Reiniciar. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` está sempre correto. Reiniciar. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | `auto` ou `off`, se um terminal ou multiplexador se comportar mal. Reiniciar. |
| `ui.mouse_enabled` | Desativado, deixa o mouse para o terminal. Reiniciar. |
| `ui.launch_terminal` | O terminal que os atalhos do SO abrem ([Configurações](settings.md)). |

Veja também [Imagens](image-view.md), [Atalhos de teclado](keybindings.md) e
[Solução de problemas](troubleshooting.md).

## Fontes

Conferidas em outubro de 2026. Fontes primárias, salvo indicação em contrário.

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [formatos de ponteiro](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [sequências de escape](https://wezterm.org/escape-sequences.html), [pull request do OSC 22](https://github.com/wezterm/wezterm/pull/6292), [versões](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [versões](https://github.com/alacritty/alacritty/releases)
- Konsole: [merge request do OSC 52](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [merge request do teclado kitty](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [issue do OSC 52](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [issue do teclado kitty](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [downloads](https://iterm2.com/downloads.html); teclado kitty a partir da [matriz do terminfo.dev](https://terminfo.dev/extensions/kitty-keyboard-protocol) (secundária)
- Windows Terminal: [anúncio da 1.22](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (secundária), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [recursos](https://rioterm.com/docs/features), [teclado kitty](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [aplicativos em tela cheia](https://docs.warp.dev/terminal/more-features/full-screen-apps); OSC 52 a partir de uma [análise do aviso de segurança do OSC 52 no Warp](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) (secundária)
- VS Code: [terminal avançado](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [sequências de controle](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [dicas](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (sem true color): [discussão da Apple](https://discussions.apple.com/thread/255315820) (secundária)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES); zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Sites de comparação (secundários, usados só para conferência cruzada): [Terminal Trove](https://terminaltrove.com/compare/terminals/)

Ainda não verificados: o sixel no Konsole e no mlterm (tirado das listas de recursos dos próprios projetos, não reconferido contra uma versão, e não experimentado em uma janela real em nenhum terminal); o OSC 22 no Konsole, iTerm2, Terminal.app e Warp; o OSC 52 no
Terminal.app, VS Code e mintty; o protocolo de teclado do kitty no VS Code, mintty e
ConEmu; todas as células do ConEmu; e a largura dos emojis por terminal. A versão em
que o Konsole traz o protocolo de teclado do kitty não foi confirmada.
