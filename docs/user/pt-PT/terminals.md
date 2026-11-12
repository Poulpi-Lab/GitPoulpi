[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · **Português (Portugal)** · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# Escolher um terminal

O GitPoulpi funciona em qualquer terminal com suporte de rato, mas fica melhor e mais
agradável num que desenhe imagens, mostre cor verdadeira e envie o teclado completo. Esta
página diz o que cada terminal oferece em cada SO, e que definição mudar.

**Como ler.** As tabelas vêm de uma ronda de testes de **novembro de 2026**: cada terminal
foi acionado com sequências de escape em bruto e, onde foi possível executá-lo, com o próprio
GitPoulpi, em Linux, Windows 11 e macOS 15. As versões estão listadas
[abaixo](#versões-testadas). Cada célula é uma destas:

- **funciona**;
- **parcial** (e o que falta);
- **não funciona** (e porquê: o *terminal*, a *camada do SO* ou o *GitPoulpi*);
- **n/d** (o terminal não existe nesse SO);
- **não medido** (e porquê).

Os multiplexadores (tmux, zellij, screen, herdr, psmux, painéis) têm a sua própria página:
[Multiplexadores](multiplexers.md).

## Melhores escolhas

| SO | Melhores escolhas, por ordem | Porquê |
|---|---|---|
| **Windows** | **Windows Terminal**, depois **WezTerm** e **Rio** | O Windows Terminal desenha o grafo e os avatares como imagens sixel, bem ancoradas, e é a predefinição do lançador. O WezTerm e o Rio desenham-nos como imagens do iTerm2, bem ancoradas desde a correção do ConPTY de novembro de 2026 (veja [Windows](#windows)). |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio** | O kitty e o Ghostty dão imagens, as formas do ponteiro e o teclado completo. O foot desenha imagens sixel e tem o teclado completo e as formas do ponteiro. |
| **macOS** | **iTerm2**, depois **kitty** e **Ghostty** | O iTerm2 3.7.7 desenha imagens através do protocolo kitty. O Terminal.app funciona mas não tem imagens nem cor verdadeira. O kitty e o Ghostty seguem o comportamento que têm em Linux (não medido em macOS). |

Em qualquer SO, o **screen** e o **zellij** escondem do GitPoulpi o protocolo de imagens do
terminal, pelo que ele desenha carateres; o tmux deixa passar sixel. Veja
[Multiplexadores](multiplexers.md).

## O que o GitPoulpi usa

- **Imagens** (coluna do grafo, avatares dos autores, diff de imagem e Vista de ficheiro): o
  **protocolo gráfico do kitty** com marcadores Unicode, as **imagens em linha do iTerm2**
  (`OSC 1337`), ou **sixel**. A deteção nega por predefinição, a partir de `TERM`,
  `TERM_PROGRAM`, `KITTY_WINDOW_ID`, `GHOSTTY_RESOURCES_DIR`, `WEZTERM_EXECUTABLE`,
  `WT_SESSION`, `KONSOLE_VERSION` e das variáveis dos multiplexadores. Quando o ambiente não
  nomeia ninguém (um xterm iniciado com `-ti vt340`, um terminal do VS Code com imagens
  ligadas, qualquer terminal alcançado por SSH, onde o `TERM_PROGRAM` não é reencaminhado),
  decidem as respostas do próprio terminal: o GitPoulpi pergunta uma vez no arranque (veja
  «A consulta de arranque» abaixo) e usa o nome XTVERSION do terminal (kitty, Ghostty,
  WezTerm, iTerm2, mintty, foot) ou sixel nos seus atributos de dispositivo. Dentro de um
  multiplexador as respostas são ignoradas, porque o tmux afirma sixel seja o que for que
  esteja por trás.
- **Que protocolo onde.** kitty, Ghostty e iTerm2 3.7.7 ou posterior: o protocolo kitty.
  WezTerm e Rio: o protocolo iTerm2, misturando a transparência do PNG. mintty e iTerm2
  anterior à 3.7.7: o protocolo iTerm2 com um fundo opaco. Windows Terminal, foot, Tabby,
  Konsole e mlterm: **sixel**. Tudo o resto: carateres, e o avatar passa a ser o crachá de
  iniciais. `ui.image_protocol` força um.
- **iTerm2 3.7.7.** Cada imagem em linha `OSC 1337` abre uma janela modal «Allow
  Terminal-Initiated Display?» que bloqueia a janela até assinalar «Remember my choice». As
  mesmas imagens enviadas pelo protocolo kitty, marcadores Unicode incluídos, desenham-se sem
  pergunta, pelo que o GitPoulpi usa aí o protocolo kitty. `ui.image_protocol = "iterm2"`
  continua a dar as imagens em linha.
- **Sixel** é desenhado para a **coluna do grafo** (faixas, nós e, com
  `ui.avatars = "graph"`, os avatares dentro dos círculos dos commits) e para os
  **avatares** noutros sítios (Log, detalhe do commit, Histórico do ficheiro, Blame, a vista
  de imagem). Um sixel não tem alfa, ids nem escala, pelo que a imagem tem exatamente
  `colunas x largura da célula` por `linhas x altura da célula` píxeis, desenhada só onde há
  algo, sobre células em branco que mantêm o fundo; só é recodificada quando as faixas, a
  posição de deslocação ou o zoom mudam. Um sixel não tem apagar: o GitPoulpi escreve texto
  sobre as suas células para o remover. O sixel precisa do tamanho real da célula em píxeis:
  o Windows Terminal, o Tabby e o mintty respondem a `CSI 16 t` uma vez no arranque e depois
  de um zoom (sem nunca bloquear se ficarem em silêncio); nos outros vem do tamanho da janela
  do terminal, ou de `GITPOULPI_TERMINAL_CELL_PX=10x20`. Um terminal que desenha sixel sem o
  dizer precisa de `ui.image_protocol = "sixel"`. O Windows Terminal anterior à 1.22 não
  desenha sixel; defina aí `ui.image_protocol = "none"`.
- **Cor verdadeira**: os temas são RGB. `ui.color_depth = "auto"` desce para 256 cores num
  `TERM` `*-256color` sem `COLORTERM=truecolor` (e no Terminal.app, que não tem cor
  verdadeira), e para 16 cores na consola Linux.
- **Rato**: relato de rato SGR com botões, arrastar e movimento (realce ao passar, arrastar e
  largar, clique direito, duplo clique).
- **A consulta de arranque.** Saem quatro perguntas numa só escrita: as flags de teclado
  kitty (`CSI ? u`), a cor de fundo (`OSC 11`, só usada com `ui.theme = system`), o nome do
  terminal (XTVERSION, `CSI > 0 q`) e os seus atributos de dispositivo (`CSI c`, por último:
  todos os terminais lhe respondem, pelo que fecha a leitura). A leitura é um ciclo `poll`
  com limite de 100 ms, terminado antes de o leitor de teclado arrancar. Um terminal que não
  responde nada custa uma vez os 100 ms e tudo fica como o ambiente diz. Só em Unix: no
  Windows a consola relata as teclas por si e decide o ambiente. As respostas estão no
  relatório de ambiente do Registo de atividade (linha `start-up query:`).
- **Teclado**: o **protocolo de teclado kitty** (apenas a flag de desambiguação) é pedido no
  arranque. Faz chegar `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` e `Ctrl+Shift+Z`. Sem ele
  ficam as teclas antigas (`Ctrl+Y` refaz, as variantes `Alt` substituem algumas
  combinações). Veja
  [Atalhos de teclado](keybindings.md#teclas-que-o-terminal-não-consegue-enviar-e-o-protocolo-de-teclado-do-kitty).
- **Área de transferência**: cada ação de copiar segue uma só política. Localmente, a **área
  de transferência do sistema** da máquina onde o GitPoulpi corre, mais `OSC 52` para o
  terminal quando se sabe que o terminal o respeita (kitty, WezTerm, Ghostty, foot,
  Alacritty, Windows Terminal, Rio, Tabby, iTerm2); o `OSC 52` é também a alternativa quando
  a área de transferência do sistema falha. **Por SSH, ou sem ecrã** (uma consola Linux, um
  contentor), só se envia `OSC 52`, e dentro do tmux envia-se embrulhado para a passagem
  além de simples. A linha de estado só diz «copiado» quando a área de transferência do
  sistema aceitou o texto; o que foi para o terminal diz «enviado ... para a área de
  transferência do terminal (OSC 52, não confirmado)», porque nada confirma que o terminal
  obedeceu. O relatório de ambiente do Registo de atividade nomeia o caminho na sua linha
  `clipboard:`.
- **Forma do ponteiro do rato** (ponteiro de redimensionar sobre os divisores, mão sobre os
  itens clicáveis, cursor de texto sobre os campos de texto): `OSC 22`, um extra onde o
  terminal o implementa. O GitPoulpi envia-o sempre (exceto na consola Linux e por trás do
  screen, zellij e herdr, que o engolem ou o imprimem; por trás do tmux é embrulhado para a
  passagem). Sob `TERM_PROGRAM=mintty` envia os nomes de cursor próprios do mintty (`hand`,
  `ibeam`, `sizewe`, ...), os únicos que o mintty lê. **O Rio não implementa `OSC 22` em
  nenhum SO**: o seu código analisa a sequência e descarta-a, pelo que nenhuma definição de
  um lado ou do outro o muda.
- **Cursor de texto**: com `ui.text_cursor = "terminal"` (a predefinição) o cursor do próprio
  terminal é posto na posição do cursor de texto do campo em foco, como uma barra
  intermitente (`DECSCUSR`), e oculto no resto; o seu estilo de cursor volta ao sair.
  `drawn` desenha em vez disso o cursor próprio do GitPoulpi.
- Os **eventos de foco** estão ativados (as dicas fecham quando a janela perde o foco). A
  **colagem entre parênteses** é pedida no arranque e libertada em todas as saídas (Unix),
  pelo que uma colagem de várias linhas chega como uma só colagem: as suas quebras de linha
  nunca submetem um campo e as suas letras nunca disparam atalhos. No Windows a consola não
  tem parênteses de colagem; veja [Windows](#windows) para o que o GitPoulpi faz em vez
  disso. O GitPoulpi não pede hiperligações OSC 8 nem sublinhado ondulado. Apenas no
  caminho das imagens, cada fotograma é embrulhado numa atualização sincronizada (modo 2026,
  ignorado pelos terminais que o não conhecem) para que uma imagem nunca pisque sobre uma
  janela.
- **Emoji e símbolos**: o GitPoulpi só usa glifos com apresentação emoji predefinida ou
  formas BMP de largura simples, porque os terminais e os tipos de letra discordam quanto aos
  seletores de variação.

## A matriz

As linhas são terminais, as colunas são o SO. «GitPoulpi» numa célula significa que o
próprio GitPoulpi foi executado aí; «sonda» significa que só foram executadas sequências de
escape em bruto. Células **n/d**: o kitty não tem versão nativa para Windows (as versões
trazem Linux e macOS); o Ghostty não tem versão para Windows (só a sua biblioteca libghostty
é multiplataforma); o foot é um terminal Wayland (só Linux); o xterm é um cliente X11 (Linux,
e macOS através do XQuartz); o iTerm2 e o Terminal.app são só para macOS; o Windows
Terminal, o conhost e o mintty são só para Windows (o mintty também sob Cygwin e MSYS2).

**Não medido em macOS: kitty, Ghostty, WezTerm, Rio e Warp.** Precisam de uma GPU (Metal ou
OpenGL 3.3). Os seus resultados em Linux são a referência do que o mesmo programa faz;
repita-os num Mac com GPU antes de confiar numa célula de macOS.

### Imagens: que protocolo o GitPoulpi escolhe, e se aparece

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **funciona**: protocolo kitty (GitPoulpi) | n/d | não medido (precisa de GPU) |
| **Ghostty** | **funciona**: protocolo kitty (GitPoulpi) | n/d | não medido (precisa de GPU) |
| **WezTerm** | **funciona**: protocolo iTerm2 com transparência (GitPoulpi). Os marcadores desenham-se esmagados, pelo que o protocolo kitty não é usado | **funciona**: protocolo iTerm2 com transparência, grafo e avatares nas suas linhas (oficial 20240203 e a compilação do PR #8092 do upstream). A oficial 20240203 punha as imagens nas linhas erradas antes de o GitPoulpi mostrar o cursor à volta delas, veja «O limite do ConPTY». | não medido (precisa de GPU) |
| **foot** | **funciona**: sixel com transparência (GitPoulpi) | n/d | n/d |
| **Rio** | **funciona**: protocolo iTerm2 com transparência (GitPoulpi) | **funciona**: protocolo iTerm2 com transparência, exato à célula em todos os tamanhos de letra experimentados (0.5.28; GitPoulpi). Sixel e kitty nunca chegam (o ConPTY descarta-os), pelo que não há nada a escolher | não medido (precisa de GPU) |
| **Alacritty** | texto: **funciona** (o terminal não tem suporte de imagens) | texto: **parcial** (sem imagens; diff de imagem em blocos) | texto: **funciona** |
| **Tabby** | **funciona**: sixel com transparência (sonda; GitPoulpi não executado no Tabby em Linux) | **funciona**: sixel, alinhado (GitPoulpi) | **funciona**: sixel, transparência mantida (GitPoulpi) |
| **Warp** | não medido (precisa de XWayland e descodifica mal teclas sintéticas, pelo que nada se pôde escrever) | **não funciona** (terminal: sem imagens sixel, iTerm2 nem kitty); GitPoulpi não medido (não se conseguiu iniciar a partir da caixa de entrada do Warp) | não medido (precisa de GPU) |
| **xterm** | sonda: sixel **funciona** com `-ti vt340`; o GitPoulpi escolhe-o a partir dos atributos de dispositivo (a consulta de arranque veio depois da execução), ou defina `ui.image_protocol = "sixel"` | n/d | texto: **funciona**; o sixel desenha-se (sonda) e precisa de `ui.image_protocol = "sixel"` |
| **iTerm2** | n/d | n/d | **funciona**: protocolo kitty na 3.7.7 e posteriores, sem janela (GitPoulpi). As versões mais antigas recebem as imagens em linha com um fundo opaco. O sixel desenha-se mas pinta de forma opaca os píxeis não definidos, pelo que `ui.image_protocol = "sixel"` dá um retângulo sólido |
| **Terminal.app** | n/d | n/d | texto: **funciona** (sem imagens) |
| **Windows Terminal** | n/d | **funciona**: sixel (GitPoulpi, 1.25) | n/d |
| **conhost** | n/d | texto: **parcial** (grafo de carateres, diff de imagem em meios blocos) | n/d |
| **mintty** | n/d | **não funciona** (terminal por trás do ConPTY: não se desenha imagem nenhuma, nem sequer uma `OSC 1337` simples posta com o cursor visível; a coluna do grafo fica em branco; texto e paleta estão bem) | n/d |

### Forma do ponteiro do rato (OSC 22)

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **funciona** | n/d | não medido (precisa de GPU) |
| **Ghostty** | **funciona** | n/d | não medido (precisa de GPU) |
| **WezTerm** | **não funciona** (terminal: não implementado na compilação testada) | oficial: **não funciona** (terminal). Compilação com o PR #8092 do WezTerm upstream, que acrescenta formas de ponteiro: **funciona** (mão, cursor de texto, setas de redimensionar) | não medido (precisa de GPU) |
| **foot** | **funciona** | n/d | n/d |
| **Rio** | **não funciona** (terminal: analisado e descartado) | **não funciona** (terminal, mesmo motivo; não o ConPTY) | **não funciona** (terminal, mesmo código; só executado em Linux e Windows) |
| **Alacritty** | **não funciona** (terminal) | **não funciona** (terminal) | **não funciona** (terminal) |
| **Tabby** | **não funciona** (terminal: xterm.js) | **não funciona** (terminal) | **não funciona** (terminal) |
| **Warp** | não medido | **não funciona** (terminal) | não medido |
| **xterm** | **não funciona** (terminal: nomes CSS desconhecidos) | n/d | não medido (o ponteiro estava fora da janela) |
| **iTerm2** | n/d | n/d | **não funciona** (terminal) |
| **Terminal.app** | n/d | n/d | **não funciona** (terminal) |
| **Windows Terminal** | n/d | **não funciona** (terminal) | n/d |
| **conhost** | n/d | **não funciona** (terminal) | n/d |
| **mintty** | n/d | **funciona** com os nomes próprios do mintty; o GitPoulpi envia-os | n/d |

Onde a célula diz «não funciona», o divisor ou o botão sob o rato continua a acender-se.

### Protocolo de teclado

O GitPoulpi pede o protocolo de teclado kitty em Linux e macOS. No Windows não o pede: a
consola relata os modificadores por si, e as combinações abaixo ficam limitadas pelo que o
terminal guarda para si.

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **funciona** | n/d | não medido (precisa de GPU) |
| **Ghostty** | **funciona** | n/d | não medido (precisa de GPU) |
| **WezTerm** | **parcial**: sem resposta até `enable_kitty_keyboard = true` em `wezterm.lua` (predefinição `false`) | **parcial**: o WezTerm guarda `Ctrl+Shift+N` (abre a sua janela); `Ctrl+,` não é entregue (compilação oficial). A compilação do PR #8092 não guarda nenhuma das combinações, mas `Ctrl+,` continua a não abrir as Definições (sem resposta de teclado kitty através do ConPTY) | não medido (precisa de GPU) |
| **foot** | **funciona** | n/d | n/d |
| **Rio** | **funciona** (ligado por predefinição) | **parcial**: o Rio guarda `Ctrl+Shift+N` (abre a sua janela) | não medido (precisa de GPU) |
| **Alacritty** | **funciona** (0.13 e posteriores) | **parcial**: `Ctrl+Shift+N` chega como `Ctrl+N`, `Ctrl+,` não é entregue | **funciona** |
| **Tabby** | **não funciona** (terminal: sem resposta; guarda `Ctrl+Shift+N` e `Ctrl+,` para si) | **parcial**: o Tabby guarda `Ctrl+Shift+N` e `Ctrl+,` | **não funciona** (terminal: sem resposta); algumas combinações não chegaram |
| **Warp** | não medido | não medido | não medido (precisa de GPU) |
| **xterm** | **não funciona** (terminal) | n/d | **não funciona** (terminal) |
| **iTerm2** | n/d | n/d | **funciona** (3.7.7; se não, verifique *Profiles > Keys*) |
| **Terminal.app** | n/d | n/d | **não funciona** (terminal: sem resposta; teclas antigas) |
| **Windows Terminal** | n/d | **parcial**: o WT guarda `Ctrl+Shift+N` (nova janela) e `Ctrl+,` (as suas definições) | n/d |
| **conhost** | n/d | não medido | n/d |
| **mintty** | n/d | **parcial**: nenhuma das combinações teve efeito visível | n/d |

Onde uma combinação não chega, use as variantes `Alt`
([Atalhos de teclado](keybindings.md#teclas-que-o-terminal-não-consegue-enviar-e-o-protocolo-de-teclado-do-kitty)),
ou reatribua a combinação em *Definições > Teclado e rato*.

### Área de transferência (ações de copiar)

Uma cópia usa sempre a área de transferência do sistema quando há um ecrã local, pelo que
estas células descrevem o caminho `OSC 52` do terminal, que conta por SSH, em contentores e
como alternativa. Todos os testes de cópia do GitPoulpi que usaram a área de transferência
do sistema funcionaram.

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **funciona** | n/d | não medido (precisa de GPU) |
| **Ghostty** | **funciona** (visto por trás do tmux, zellij, herdr); sem nada: não medido (uma escrita na área de transferência precisa do foco do teclado, que o teste não lhe deu) | n/d | não medido (precisa de GPU) |
| **WezTerm** | **funciona** | **funciona** | não medido (precisa de GPU) |
| **foot** | não medido (uma escrita precisa do foco do teclado, que o teste não lhe deu) | n/d | n/d |
| **Rio** | não medido (mesmo motivo) | **funciona** | não medido (precisa de GPU) |
| **Alacritty** | não medido (mesmo motivo) | **não funciona** (área de transferência inalterada com a configuração predefinida; não investigado) | **funciona** |
| **Tabby** | **funciona** | **funciona** | **funciona** |
| **Warp** | não medido | **não funciona** (área de transferência inalterada) | não medido (precisa de GPU) |
| **xterm** | não medido | n/d | **não funciona** (área de transferência inalterada) |
| **iTerm2** | n/d | n/d | **não funciona** até *Settings > General > Selection > Applications in terminal may access clipboard* estar ligado |
| **Terminal.app** | n/d | n/d | **não funciona** (terminal) |
| **Windows Terminal** | n/d | **funciona** (com foco) | n/d |
| **conhost** | n/d | não medido (nenhuma sonda executada) | n/d |
| **mintty** | n/d | **não funciona** (área de transferência inalterada com as suas definições predefinidas) | n/d |

### Colar

Em Linux e macOS o GitPoulpi pede colagem entre parênteses, pelo que uma colagem de várias
linhas é uma só colagem. Isto não foi executado terminal a terminal (**não medido** por
terminal). No Windows a consola não tem parênteses de colagem, pelo que um terminal que não
pergunte primeiro entrega o texto como pressões de teclas e cada quebra de linha seria um
`Enter`. O GitPoulpi trata por isso as pressões de teclas que se seguem umas às outras em
30 ms (um `Enter` interior, ou oito ou mais carateres) como uma só colagem. Teclas mantidas
premidas e «abc, `Enter`» escritos à mão não são colagens.

| Terminal | Windows (colagem de duas linhas no pedido de nova branch) |
|---|---|
| **Windows Terminal** | **funciona**: o WT pergunta primeiro «colar várias linhas?» (o seu próprio aviso), e depois nenhuma branch é criada |
| **WezTerm**, **Rio**, **Alacritty**, **Tabby**, **mintty** | **funciona** (nenhuma branch criada) |
| **Warp** | não medido (o GitPoulpi não se conseguiu iniciar dentro dele) |
| **conhost** | não medido (`Ctrl+V` e `Shift+Insert` enviados pelo teste não colaram na consola clássica) |

### Largura de emoji e de ícones

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio**, **Alacritty** | **funciona** (células largas alinhadas; o kitty desenha a engrenagem U+2699 em branco, uma questão conhecida do tipo de letra) | WezTerm, Rio, Tabby, Windows Terminal: **funciona**. Alacritty: **parcial** (emoji como contornos monocromáticos) | Alacritty, Tabby: **funciona**. Os outros: não medido (precisa de GPU) |
| **Tabby** | **funciona** | **funciona** | **funciona** |
| **Warp** | não medido | **funciona** | não medido |
| **xterm** | **parcial** (caixas: o tipo de letra) | n/d | **parcial** (caixas: o tipo de letra) |
| **iTerm2**, **Terminal.app** | n/d | n/d | **funciona** |
| **conhost** | n/d | **parcial**: emoji a cores desenhados, células largas alinhadas | n/d |
| **mintty** | n/d | **funciona** | n/d |

### Versões testadas

kitty 0.49.2 (a 0.50.0 de macOS não arrancou), Ghostty 1.4.0, WezTerm 20260716 em Linux e
20240203 (a última versão etiquetada) em Windows, foot 1.28.0, Rio 0.5.28 (Linux, Windows) e
0.5.29 (macOS), Alacritty 0.17.0, Tabby 1.0.238, Warp 0.2026.11.04, xterm 411 (Linux) e 378
(XQuartz), iTerm2 3.7.7, Terminal.app 455.1 (macOS 15.8), Windows Terminal 1.25.2733, mintty
3.8.3. Os terminais mudam depressa: uma versão mais recente do que estas pode diferir.

## Terminal a terminal

### Windows

- **Windows Terminal.** A predefinição do lançador e o melhor suporte de imagens no Windows:
  grafo e avatares sixel, OSC 52, bons emoji, e responde à consulta do tamanho da célula.
  Escolha uma Nerd Font em *Settings > Profiles > Defaults > Appearance > Font face*.
  Mantenha `Shift` premido para selecionar texto enquanto o rato está capturado. Guarda
  `Ctrl+Shift+N` e `Ctrl+,`: use as variantes `Alt` ou reatribua. A forma do ponteiro não
  muda.
- **WezTerm.** Em `wezterm.lua`, defina `config.enable_kitty_keyboard = true` (compilações
  Linux e macOS) e um tipo de letra com emoji de reserva, por exemplo
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  As imagens usam o protocolo iTerm2 e nada precisa de ser definido: a última versão
  etiquetada, 20240203-110809 (fev. de 2024), ancora-as nas linhas certas (reverificado em
  novembro de 2026 com o GitPoulpi atual; antes da correção do ConPTY descrita abaixo não o
  fazia). O PR #8092 do WezTerm upstream acrescenta formas de ponteiro.
- **Rio** (MSI, precisa de UAC). As imagens (grafo, avatares, diff de imagem) usam o
  protocolo iTerm2 com transparência e caem exatamente nas suas células: medido na 0.5.28 com
  o tipo de letra predefinido e a 18 e 22,5 px (células de 8x16, 11x21 e 13x26 píxeis,
  sempre píxeis inteiros), uma imagem de 20x10 células e outra de 4x27 células saíram ambas
  exatas à célula. Nada precisa de ser definido. A pseudo-consola engole a consulta do
  tamanho da célula, pelo que o GitPoulpi assume que as células do Rio têm o dobro da altura
  da largura (verdade nos três tamanhos); com um tipo de letra de outra proporção defina
  `GITPOULPI_TERMINAL_CELL_PX=WxH` (por exemplo `10x24`) para obter marcadores de commit
  redondos. O Rio não tem formas de ponteiro em nenhum SO.
- **Tabby.** Grafo e avatares sixel, copiar e colar funcionam. O Tabby guarda
  `Ctrl+Shift+N` e `Ctrl+,` para si.
- **Alacritty.** Precisa de OpenGL 3.3. Grafo de carateres; a cópia por `OSC 52` não estava
  ligada com a configuração predefinida.
- **mintty** (Git Bash, MSYS2, Cygwin). As formas de ponteiro funcionam (nomes próprios do
  mintty). As imagens não aparecem (a coluna do grafo fica em branco; o mintty 3.8.3 não
  desenha nenhuma imagem `OSC 1337` por trás do ConPTY, mesmo posta por uma sonda simples) e
  aí não responde a nenhuma consulta de tamanho de célula: defina
  `ui.graph_renderer = "text"`.
- **Warp.** Não medido com o GitPoulpi: não se conseguiu iniciá-lo a partir da caixa de
  entrada do Warp. O terminal não tem imagens sixel, iTerm2 nem kitty.
- **conhost, cmd, uma janela do PowerShell.** Grafo de carateres, diff de imagem em meios
  blocos. Por predefinição, o GitPoulpi iniciado a partir de uma janela dessas reabre-se no
  terminal que escolheu; veja «A regra do relançamento».

**O limite do ConPTY.** O Windows executa o programa de cada terminal por trás de uma
pseudo-consola (ConPTY). O Windows Terminal e as compilações do WezTerm que trazem uma recente
deixam passar as sequências de imagem; a que vem dentro do Windows (usada pelo Rio e pelo
mintty), e a que vem com o WezTerm 20240203, não. O que se perde: as sequências DCS (sixel) e
APC (gráficos kitty) são descartadas, pelo que só chegam as imagens iTerm2 (`OSC 1337`), e é
por isso que o Rio e o WezTerm as usam; `CSI 16 t` (tamanho da célula) só tem resposta no
Windows Terminal e no Tabby. O que foi corrigido: uma `OSC 1337` atravessa o ConPTY logo e cai
no cursor que o ConPTY pintou por último, e o ConPTY só pinta o cursor enquanto este está
visível. O GitPoulpi mantém o cursor oculto, pelo que as suas imagens caíam em linhas
antigas (o grafo sobre o rodapé, um ecrã rasgado após deslocar). Agora mostra o cursor à volta
de cada imagem que escreve no Windows, dentro da mesma escrita, e volta a ocultá-lo; o Rio e o
WezTerm 20240203 ancoram cada imagem na sua célula (`pty_conpty_picture_anchor` lê de volta o
movimento do cursor por trás do ConPTY). Nada muda fora do Windows nem para sixel.

**Colar no Windows.** Veja a tabela «Colar» acima: o GitPoulpi junta uma rajada de pressões
de teclas numa só colagem, pelo que uma colagem de várias linhas nunca submete um pedido nos
terminais testados.

**A regra do relançamento.** Um duplo clique no exe inicia-o numa janela de consola clássica
(conhost) própria. Quando `ui.launch_terminal` nomeia outro terminal (predefinição: Windows
Terminal), o GitPoulpi reabre-se aí e a primeira janela fecha
(`ui.relaunch_in_terminal`). Fá-lo **apenas** quando está sozinho numa janela de consola clássica.
Um terminal que executa o exe diretamente (`wt.exe gitpoulpi.exe`, WezTerm, Rio, Alacritty
`-e`, Tabby `run`) é onde quer estar, e nada é relançado; a partir de uma shell não está
sozinho, pelo que também nada acontece. Desligue-o com `ui.relaunch_in_terminal = false`.

### Linux

- **kitty.** Tudo funciona sem nenhuma definição. `Shift+clique` é do próprio kitty, pelo que
  use `Alt+clique` para alargar uma seleção ([Rato](mouse-and-gestures.md)).
- **Ghostty.** Tudo funciona sem nenhuma definição. Se os ícones aparecerem como buracos,
  acrescente um tipo de letra emoji ou Nerd Font a `font-family` (repita a chave para
  acrescentar alternativas).
- **WezTerm.** Imagens pelo protocolo iTerm2, transparência misturada. Defina
  `enable_kitty_keyboard = true` para as combinações completas. Sem formas de ponteiro.
- **foot.** Grafo e avatares sixel, formas de ponteiro, teclado completo, rápido. Defina
  `font=JetBrainsMono Nerd Font:size=11` em `foot.ini`. A cópia por `OSC 52` é governada por
  `security.osc52`.
- **Rio.** Imagens pelo protocolo iTerm2 com transparência, teclado completo. Sem formas de
  ponteiro (o Rio descarta `OSC 22`).
- **Alacritty.** Grafo de carateres, teclado kitty desde a 0.13. Sem suporte de imagens.
- **Tabby.** Imagens sixel; sem teclado kitty, sem formas de ponteiro (Electron, xterm.js).
  Guarda `Ctrl+Shift+N` e `Ctrl+,`.
- **xterm.** Inicie-o como `xterm -u8 -ti vt340` para sixel (o UTF-8 é preciso para o
  desenho de caixas). Caixas em vez de emoji: o tipo de letra.
- **Warp.** Não medido: precisa de XWayland e, no teste, descodificou mal teclas sintéticas.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Não fazem parte da ronda de novembro de 2026.
  Funcionam com carateres e teclas antigas: use as variantes `Alt` e `Ctrl+Y` para refazer.
  O VTE ignora `OSC 52`, pelo que copiar por SSH não chega à área de transferência. Um método
  de entrada (IBus, Fcitx) pode apanhar a alternância de preparação `Ctrl+Espaço`; reatribua-a
  em *Definições > Teclado e rato*. O Konsole é reconhecido para sixel pela sua própria
  documentação.
- **Wayland, Hyprland, Omarchy.** A dica de ponteiro (`OSC 22`) funciona no kitty, Ghostty e
  foot. A entrada do lançador do Omarchy passa por `xdg-terminal-exec`.

### macOS

- **iTerm2.** A 3.7.7 e posteriores desenham imagens pelo protocolo kitty, sem perguntas.
  Definições que ajudam:
  - *Settings > General > Selection > Applications in terminal may access clipboard*:
    necessária para copiar por `OSC 52` (caso contrário o iTerm2 mostra «the terminal
    attempted to access the clipboard but it was denied»).
  - *Profiles > Keys > Left Option key: Esc+*: necessária para as combinações `Alt`.
  - Se definir `ui.image_protocol = "iterm2"` (ou usar uma versão anterior à 3.7.7),
    assinale «Remember my choice» na janela «Allow Terminal-Initiated Display?». As imagens
    passam então a ter um fundo opaco (um retângulo visível se o seu fundo diferir do do
    tema; `ui.graph_renderer = "text"` remove-o).
  - O protocolo de teclado kitty está ligado por predefinição (3.6.9 e posteriores).
- **Terminal.app.** Só 256 cores: o GitPoulpi escolhe `ui.color_depth = 256` por si e os
  temas ficam mais baços. Sem imagens, sem teclado kitty, sem área de transferência por
  `OSC 52`. Assinale *Use Option as Meta key* para as combinações `Alt`.
- **Alacritty, Tabby.** Alacritty: grafo de carateres, teclado kitty, cópia por `OSC 52`;
  defina `option_as_alt = "Both"` para as combinações `Alt`. Tabby: grafo e avatares sixel.
- **Option como Meta.** A tecla Option compõe carateres (Option+B escreve um sinal de
  integral) a menos que o terminal a transforme em Meta: iTerm2 *Left Option key: Esc+*,
  Terminal.app *Use Option as Meta key*, Alacritty `option_as_alt`. As combinações `Alt` do
  GitPoulpi precisam disso, e em lado nenhum vem ligado por predefinição.
- **kitty, Ghostty, WezTerm, Rio, Warp**: não medidos em macOS (precisam de GPU). Espere o
  comportamento de Linux.
- **xterm** (XQuartz): `xterm -u8 -ti vt340`; o sixel precisa de `ui.image_protocol = "sixel"`.

### VS Code e Warp

O terminal do VS Code (`TERM_PROGRAM=vscode`) não faz parte da ronda. É reconhecido pelos seus
atributos de dispositivo quando `terminal.integrated.enableImages` está ligado; caso contrário o
GitPoulpi desenha carateres; `ui.image_protocol = "sixel"` força as imagens. Warp: veja as notas
por SO acima.

## Multiplexadores

O tmux, o zellij, o screen, o herdr, o psmux e os painéis próprios dos terminais ficam entre o
GitPoulpi e o terminal e mudam o que passa. A versão curta: o tmux deixa passar sixel e, com
`allow-passthrough on`, formas de ponteiro e imagens kitty; o zellij e o screen não deixam
passar imagens nem formas de ponteiro; o herdr deixa passar imagens kitty. Todos os detalhes e a
configuração a acrescentar estão em [Multiplexadores](multiplexers.md).

## O que se degrada, e como

| Em falta | O que obtém em vez disso |
|---|---|
| Protocolo de imagens | Grafo de carateres. O avatar passa a ser o crachá colorido de iniciais (uma letra) seja o que for que `ui.avatar_fallback` diga: um identicon espremido em células de texto é irreconhecível. O diff de imagem e a Vista de ficheiro usam meios blocos coloridos, ou uma ficha de informações. |
| Cor verdadeira | As cores são mapeadas para a paleta de 256 ou 16 cores (`ui.color_depth`). |
| Protocolo de teclado kitty | Teclas antigas; algumas combinações chegam como outra tecla. As Definições avisam, e reatribui ou usa as variantes `Alt`. |
| OSC 52 | Localmente nada muda (usa-se a área de transferência do sistema). Por SSH ou sem ecrã, as ações de copiar não põem nada na área de transferência: selecione com o rato (mantenha `Shift`) e use a cópia do próprio terminal. |
| OSC 22 | O ponteiro não muda; o divisor ou o botão sob o rato continua a acender-se. |
| Tipo de letra emoji | Caixas ou buracos em vez de ícones; o grafo e o texto continuam a funcionar. Instale uma Nerd Font ou um tipo de letra emoji e indique-o como alternativa. |

## Definições a conhecer

| Definição | Uso |
|---|---|
| `ui.image_protocol` | `auto` (predefinição), `kitty`, `iterm2`, `sixel`, `none`. Defina `none` se vir lixo onde deviam estar imagens. Reiniciar. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` está sempre correto (use-o onde se aplica o limite do ConPTY). Reiniciar. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | `auto` ou `off`, se um terminal ou multiplexador se portar mal. Reiniciar. |
| `ui.text_cursor` | `terminal` (predefinição) ou `drawn`. |
| `ui.mouse_enabled` | Desligado deixa o rato ao terminal. Reiniciar. |
| `ui.launch_terminal`, `ui.relaunch_in_terminal` | O terminal que os atalhos do SO abrem, e se um duplo clique reabre o GitPoulpi aí ([Definições](settings.md)). |

Veja também [Multiplexadores](multiplexers.md), [Imagens](image-view.md),
[Atalhos de teclado](keybindings.md) e [Resolução de problemas](troubleshooting.md).

## Fontes

Documentação verificada em outubro de 2026, comportamento testado em novembro de 2026. Fontes primárias salvo indicação em contrário.

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [pointer shapes](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sequences](https://wezterm.org/escape-sequences.html), [OSC 22 pull request](https://github.com/wezterm/wezterm/pull/6292), [releases](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [releases](https://github.com/alacritty/alacritty/releases)
- Konsole: [OSC 52 merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [kitty keyboard merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [OSC 52 issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [kitty keyboard issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [downloads](https://iterm2.com/downloads.html); teclado kitty a partir da [matriz terminfo.dev](https://terminfo.dev/extensions/kitty-keyboard-protocol) (secundária)
- Windows Terminal: [1.22 announcement, sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (secundária), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [escape sequence support](https://rioterm.com/docs/escape-sequence-support) (sem linha `OSC 22`), [features](https://rioterm.com/docs/features), [kitty keyboard](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [full-screen apps](https://docs.warp.dev/terminal/more-features/full-screen-apps); OSC 52 a partir de um [artigo sobre o aviso de OSC 52 no Warp](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) (secundária)
- VS Code: [advanced terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [control sequences](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tips](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (sem cor verdadeira): [discussão da Apple](https://discussions.apple.com/thread/255315820) (secundária)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES); zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Sites de comparação (secundários, usados só para confirmar): [Terminal Trove](https://terminaltrove.com/compare/terminals/)
