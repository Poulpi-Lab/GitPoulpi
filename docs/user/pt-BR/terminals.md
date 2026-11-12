[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# Escolhendo um terminal

O GitPoulpi roda em qualquer terminal com suporte a mouse, mas fica melhor e mais
agradável de usar em um que desenhe imagens, mostre true color e envie o teclado
completo. Esta página diz o que cada terminal oferece em cada sistema operacional e
qual configuração mudar.

**Como ler.** As tabelas vêm de uma rodada de testes de **novembro de 2026**: cada
terminal foi acionado com sequências de escape brutas e, onde foi possível executá-lo,
com o próprio GitPoulpi, no Linux, no Windows 11 e no macOS 15. As versões estão
[abaixo](#versões-testadas). Cada célula é uma destas:

- **funciona**;
- **parcial** (e o que falta);
- **não funciona** (e por quê: o *terminal*, a *camada do SO* ou o *GitPoulpi*);
- **n/d** (o terminal não existe nesse SO);
- **não medido** (e por quê).

Os multiplexadores (tmux, zellij, screen, herdr, psmux, painéis) têm a sua própria página:
[Multiplexadores](multiplexers.md).

## Melhores opções

| SO | Melhores opções, em ordem | Por quê |
|---|---|---|
| **Windows** | **Windows Terminal**, depois **WezTerm** e **Rio** | O Windows Terminal desenha o grafo e os avatares como imagens sixel, ancoradas no lugar certo, e é o padrão do lançador. O WezTerm e o Rio os desenham como imagens do iTerm2, ancoradas no lugar certo desde a correção do ConPTY de novembro de 2026 (veja [Windows](#windows)). |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio** | kitty e Ghostty oferecem imagens, os formatos de ponteiro e o teclado completo. O foot desenha imagens sixel e tem o teclado completo e os formatos de ponteiro. |
| **macOS** | **iTerm2**, depois **kitty** e **Ghostty** | O iTerm2 3.7.7 desenha imagens pelo protocolo do kitty. O Terminal.app funciona, mas não tem imagens nem true color. kitty e Ghostty seguem o comportamento que têm no Linux (não medido no macOS). |

Em qualquer SO, **screen** e **zellij** escondem do GitPoulpi o protocolo de imagens do
terminal, então ele desenha caracteres; o tmux repassa sixel. Veja
[Multiplexadores](multiplexers.md).

## O que o GitPoulpi usa

- **Imagens** (coluna do grafo, avatares dos autores, diff de imagem e Visualização de
  arquivo): o **protocolo gráfico do kitty** com placeholders Unicode, as **imagens inline
  do iTerm2** (`OSC 1337`) ou **sixel**. A detecção é negar por padrão, a partir de `TERM`,
  `TERM_PROGRAM`, `KITTY_WINDOW_ID`, `GHOSTTY_RESOURCES_DIR`, `WEZTERM_EXECUTABLE`,
  `WT_SESSION`, `KONSOLE_VERSION` e das variáveis dos multiplexadores. Quando o ambiente
  não nomeia ninguém (um xterm iniciado com `-ti vt340`, um terminal do VS Code com imagens
  ativadas, qualquer terminal alcançado por SSH, onde `TERM_PROGRAM` não é repassado), as
  respostas do próprio terminal decidem: o GitPoulpi pergunta uma vez na inicialização
  (veja "A consulta de inicialização" abaixo) e usa o nome XTVERSION do terminal (kitty,
  Ghostty, WezTerm, iTerm2, mintty, foot) ou o sixel nos atributos do dispositivo. Dentro de
  um multiplexador as respostas são ignoradas, porque o tmux afirma ter sixel seja o que for
  que esteja por trás dele.
- **Qual protocolo onde.** kitty, Ghostty e iTerm2 3.7.7 ou mais recente: o protocolo do
  kitty. WezTerm e Rio: o protocolo do iTerm2, misturando a transparência do PNG. mintty e
  iTerm2 anterior à 3.7.7: o protocolo do iTerm2 com um fundo opaco. Windows Terminal, foot,
  Tabby, Konsole e mlterm: **sixel**. Todo o resto: caracteres, e o avatar vira o selo com as
  iniciais. `ui.image_protocol` força um.
- **iTerm2 3.7.7.** Cada imagem inline `OSC 1337` abre um diálogo modal "Allow
  Terminal-Initiated Display?" que bloqueia a janela até você marcar "Remember my choice". As
  mesmas imagens enviadas pelo protocolo do kitty, placeholders Unicode incluídos, são
  desenhadas sem pergunta, então o GitPoulpi usa o protocolo do kitty ali.
  `ui.image_protocol = "iterm2"` ainda dá as imagens inline.
- **Sixel** é desenhado para a **coluna do grafo** (faixas, nós e, com `ui.avatars = "graph"`,
  os avatares dentro dos círculos dos commits) e, nos demais lugares, para os **avatares**
  (Log, detalhe do commit, Histórico do arquivo, Blame, a visualização de imagem). Um sixel
  não tem alfa, ids nem escala, então a imagem tem exatamente `colunas x largura da célula`
  por `linhas x altura da célula` pixels, desenhada só onde há algo, sobre células vazias que
  mantêm o fundo; ela é recodificada só quando as faixas, a rolagem ou o zoom mudam. Um sixel
  não tem como ser apagado: o GitPoulpi escreve texto sobre as suas células para removê-lo. O
  sixel precisa do tamanho real da célula em pixels: o Windows Terminal, o Tabby e o mintty
  respondem a `CSI 16 t` uma vez na partida e após um zoom (nunca bloqueia se ficarem
  calados); em outros lugares vem do tamanho da janela do terminal, ou de
  `GITPOULPI_TERMINAL_CELL_PX=10x20`. Um terminal que desenha sixel sem dizê-lo precisa de
  `ui.image_protocol = "sixel"`. O Windows Terminal anterior à 1.22 não desenha sixel; defina
  `ui.image_protocol = "none"` nele.
- **True color**: os temas são RGB. `ui.color_depth = "auto"` cai para 256 cores em um `TERM`
  `*-256color` sem `COLORTERM=truecolor` (e no Terminal.app, que não tem true color), e para
  16 cores no console do Linux.
- **Mouse**: relatório de mouse SGR com botões, arrastar e movimento (feedback ao passar o
  mouse, arrastar e soltar, clique com o botão direito, clique duplo).
- **A consulta de inicialização.** Quatro perguntas saem em uma só escrita: as flags do teclado
  do kitty (`CSI ? u`), a cor de fundo (`OSC 11`, usada só com `ui.theme = system`), o nome do
  terminal (XTVERSION, `CSI > 0 q`) e os atributos do dispositivo (`CSI c`, por último: todo
  terminal responde, então ela encerra a leitura). A leitura é um só laço de `poll` com prazo
  de 100 ms, terminado antes de o leitor de teclado começar. Um terminal que não responde nada
  custa os 100 ms uma vez e tudo continua como o ambiente diz. Só em Unix: no Windows o console
  informa as teclas sozinho e o ambiente decide. As respostas estão no relatório de ambiente do
  Registro de atividade (linha `start-up query:`).
- **Teclado**: o **protocolo de teclado do kitty** (somente a flag de desambiguação) é
  solicitado na inicialização. Ele faz `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` e `Ctrl+Shift+Z`
  chegarem. Sem ele, as teclas legadas continuam (`Ctrl+Y` refaz, as variantes com `Alt`
  substituem alguns atalhos). Veja
  [Atalhos de teclado](keybindings.md#teclas-que-o-terminal-não-consegue-enviar-e-o-protocolo-de-teclado-do-kitty).
- **Área de transferência**: toda ação de copiar segue uma só política. Localmente, a **área de
  transferência do sistema** da máquina em que o GitPoulpi roda, mais `OSC 52` para o terminal
  quando ele é conhecido por respeitá-lo (kitty, WezTerm, Ghostty, foot, Alacritty, Windows
  Terminal, Rio, Tabby, iTerm2); o `OSC 52` também é o recurso alternativo quando a área de
  transferência do sistema falha. **Por SSH, ou sem tela** (um console do Linux, um contêiner),
  só o `OSC 52` é enviado, e dentro do tmux ele é enviado embrulhado para a passagem direta e
  também sem embrulho. A linha de status diz "copiado" só quando a área de transferência do
  sistema aceitou o texto; o que foi para o terminal diz "enviado ... para a área de
  transferência do terminal (OSC 52, não confirmado)", porque nada confirma que o terminal
  obedeceu. O relatório de ambiente do Registro de atividade nomeia o caminho na linha
  `clipboard:`.
- **Formato do ponteiro do mouse** (ponteiro de redimensionamento sobre os divisores, mão sobre
  itens clicáveis, I-beam sobre os campos de texto): `OSC 22`, um bônus onde o terminal o
  implementa. O GitPoulpi sempre o envia (exceto no console do Linux e atrás do screen, zellij e
  herdr, que o engolem ou o imprimem; atrás do tmux ele é embrulhado para a passagem direta).
  Com `TERM_PROGRAM=mintty` ele envia os nomes de cursor do próprio mintty (`hand`, `ibeam`,
  `sizewe`, ...), os únicos que o mintty lê. **O Rio não implementa o `OSC 22` em nenhum SO**: o
  código dele interpreta a sequência e a descarta, então nenhuma configuração de nenhum dos
  lados muda isso.
- **Cursor de texto**: com `ui.text_cursor = "terminal"` (o padrão), o cursor do próprio terminal
  é colocado no ponto de inserção do campo de texto em foco, como uma barra piscante
  (`DECSCUSR`), e ocultado nos demais lugares; o seu próprio estilo de cursor volta ao sair.
  `drawn` desenha o cursor do próprio GitPoulpi.
- Os **eventos de foco** estão ativados (as dicas fecham quando a janela perde o foco). O
  **colar entre colchetes** (bracketed paste) é solicitado na inicialização e liberado em toda
  saída (Unix), então uma colagem de várias linhas chega como uma só colagem: as quebras de linha
  nunca confirmam um campo e as letras nunca disparam atalhos. No Windows o console não tem
  colchetes de colagem; veja [Windows](#windows) para o que o GitPoulpi faz em vez disso. O
  GitPoulpi não solicita hiperlinks OSC 8 nem sublinhado ondulado. Somente no caminho das
  imagens, cada quadro é envolvido em uma atualização sincronizada (modo 2026, ignorado pelos
  terminais que não o conhecem) para que uma imagem nunca pisque sobre um popup.
- **Emojis e símbolos**: o GitPoulpi só usa glifos com apresentação de emoji padrão ou formas
  BMP de largura simples, porque terminais e fontes discordam sobre os seletores de variação.

## A tabela

As linhas são terminais, as colunas são o SO. "GitPoulpi" em uma célula significa que o próprio
GitPoulpi foi executado ali; "sondagem" significa que só sequências de escape brutas foram. Células
**n/d**: o kitty não tem versão nativa para Windows (as versões trazem Linux e macOS); o Ghostty
não tem versão para Windows (só a biblioteca libghostty é multiplataforma); o foot é um terminal
Wayland (só Linux); o xterm é um cliente X11 (Linux, e macOS pelo XQuartz); iTerm2 e Terminal.app
são só do macOS; Windows Terminal, conhost e mintty são só do Windows (o mintty também sob Cygwin e
MSYS2).

**Não medidos no macOS: kitty, Ghostty, WezTerm, Rio e Warp.** Eles precisam de GPU (Metal ou
OpenGL 3.3). Os resultados deles no Linux são a referência do que o mesmo programa faz; rode-os de
novo em um Mac com GPU antes de confiar em uma célula do macOS.

### Imagens: qual protocolo o GitPoulpi escolhe, e se aparece

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **funciona**: protocolo do kitty (GitPoulpi) | n/d | não medido (precisa de GPU) |
| **Ghostty** | **funciona**: protocolo do kitty (GitPoulpi) | n/d | não medido (precisa de GPU) |
| **WezTerm** | **funciona**: protocolo do iTerm2 com transparência (GitPoulpi). Os placeholders são desenhados achatados, então o protocolo do kitty não é usado | **funciona**: protocolo do iTerm2 com transparência, grafo e avatares nas suas linhas (o oficial 20240203 e a versão com o PR upstream #8092). O oficial 20240203 colocava as imagens nas linhas erradas antes de o GitPoulpi mostrar o cursor em volta delas, veja "O limite do ConPTY". | não medido (precisa de GPU) |
| **foot** | **funciona**: sixel com transparência (GitPoulpi) | n/d | n/d |
| **Rio** | **funciona**: protocolo do iTerm2 com transparência (GitPoulpi) | **funciona**: protocolo do iTerm2 com transparência, exato por célula em todo tamanho de fonte testado (0.5.28; GitPoulpi). Sixel e kitty nunca chegam (o ConPTY os descarta), então não há o que escolher | não medido (precisa de GPU) |
| **Alacritty** | texto: **funciona** (o terminal não tem suporte a imagens) | texto: **parcial** (sem imagens; diff de imagem em blocos) | texto: **funciona** |
| **Tabby** | **funciona**: sixel com transparência (sondagem; o GitPoulpi não foi executado no Tabby no Linux) | **funciona**: sixel, alinhado (GitPoulpi) | **funciona**: sixel, transparência mantida (GitPoulpi) |
| **Warp** | não medido (precisa do XWayland e decodifica errado teclas sintéticas, então nada pôde ser digitado) | **não funciona** (terminal: sem imagens sixel, iTerm2 nem kitty); GitPoulpi não medido (não foi possível iniciá-lo pela caixa de entrada do Warp) | não medido (precisa de GPU) |
| **xterm** | sondagem: o sixel **funciona** com `-ti vt340`; o GitPoulpi o escolhe pelos atributos do dispositivo (a consulta de inicialização veio depois da execução), ou defina `ui.image_protocol = "sixel"` | n/d | texto: **funciona**; o sixel é desenhado (sondagem) e precisa de `ui.image_protocol = "sixel"` |
| **iTerm2** | n/d | n/d | **funciona**: protocolo do kitty na 3.7.7 e mais recentes, sem diálogo (GitPoulpi). Versões mais antigas recebem as imagens inline com um fundo opaco. O sixel é desenhado mas pinta de opaco os pixels não definidos, então `ui.image_protocol = "sixel"` dá um retângulo sólido |
| **Terminal.app** | n/d | n/d | texto: **funciona** (sem imagens) |
| **Windows Terminal** | n/d | **funciona**: sixel (GitPoulpi, 1.25) | n/d |
| **conhost** | n/d | texto: **parcial** (grafo de caracteres, diff de imagem em meios-blocos) | n/d |
| **mintty** | n/d | **não funciona** (terminal atrás do ConPTY: nenhuma imagem é desenhada, nem mesmo uma `OSC 1337` simples posicionada com o cursor visível; a coluna do grafo fica em branco; texto e paleta estão bem) | n/d |

### Formato do ponteiro do mouse (OSC 22)

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **funciona** | n/d | não medido (precisa de GPU) |
| **Ghostty** | **funciona** | n/d | não medido (precisa de GPU) |
| **WezTerm** | **não funciona** (terminal: não implementado na versão testada) | oficial: **não funciona** (terminal). Versão com o PR upstream #8092 do WezTerm, que acrescenta formatos de ponteiro: **funciona** (mão, I-beam, setas de redimensionamento) | não medido (precisa de GPU) |
| **foot** | **funciona** | n/d | n/d |
| **Rio** | **não funciona** (terminal: interpretado e descartado) | **não funciona** (terminal, mesmo motivo; não é o ConPTY) | **não funciona** (terminal, mesmo código; executado só no Linux e no Windows) |
| **Alacritty** | **não funciona** (terminal) | **não funciona** (terminal) | **não funciona** (terminal) |
| **Tabby** | **não funciona** (terminal: xterm.js) | **não funciona** (terminal) | **não funciona** (terminal) |
| **Warp** | não medido | **não funciona** (terminal) | não medido |
| **xterm** | **não funciona** (terminal: nomes CSS desconhecidos) | n/d | não medido (o ponteiro estava fora da janela) |
| **iTerm2** | n/d | n/d | **não funciona** (terminal) |
| **Terminal.app** | n/d | n/d | **não funciona** (terminal) |
| **Windows Terminal** | n/d | **não funciona** (terminal) | n/d |
| **conhost** | n/d | **não funciona** (terminal) | n/d |
| **mintty** | n/d | **funciona** com os nomes do próprio mintty; o GitPoulpi os envia | n/d |

Onde a célula diz "não funciona", o divisor ou botão sob o mouse ainda se acende.

### Protocolo de teclado

O GitPoulpi pede o protocolo de teclado do kitty no Linux e no macOS. No Windows ele não pede: o
console informa os modificadores sozinho, e os atalhos abaixo são limitados pelo que o terminal
guarda para si.

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **funciona** | n/d | não medido (precisa de GPU) |
| **Ghostty** | **funciona** | n/d | não medido (precisa de GPU) |
| **WezTerm** | **parcial**: sem resposta até `enable_kitty_keyboard = true` no `wezterm.lua` (padrão `false`) | **parcial**: o WezTerm guarda `Ctrl+Shift+N` (abre a janela dele); `Ctrl+,` não é entregue (versão oficial). A versão do PR #8092 não guarda nenhum dos dois, mas `Ctrl+,` ainda não abre as Configurações (sem resposta do teclado do kitty pelo ConPTY) | não medido (precisa de GPU) |
| **foot** | **funciona** | n/d | n/d |
| **Rio** | **funciona** (ativado por padrão) | **parcial**: o Rio guarda `Ctrl+Shift+N` (abre a janela dele) | não medido (precisa de GPU) |
| **Alacritty** | **funciona** (0.13 e mais recentes) | **parcial**: `Ctrl+Shift+N` chega como `Ctrl+N`, `Ctrl+,` não é entregue | **funciona** |
| **Tabby** | **não funciona** (terminal: sem resposta; ele guarda `Ctrl+Shift+N` e `Ctrl+,` para si) | **parcial**: o Tabby guarda `Ctrl+Shift+N` e `Ctrl+,` | **não funciona** (terminal: sem resposta); alguns atalhos não chegaram |
| **Warp** | não medido | não medido | não medido (precisa de GPU) |
| **xterm** | **não funciona** (terminal) | n/d | **não funciona** (terminal) |
| **iTerm2** | n/d | n/d | **funciona** (3.7.7; confira *Profiles > Keys* se não funcionar) |
| **Terminal.app** | n/d | n/d | **não funciona** (terminal: sem resposta; teclas legadas) |
| **Windows Terminal** | n/d | **parcial**: o WT guarda `Ctrl+Shift+N` (nova janela) e `Ctrl+,` (as configurações dele) | n/d |
| **conhost** | n/d | não medido | n/d |
| **mintty** | n/d | **parcial**: nenhum dos dois atalhos teve efeito visível | n/d |

Onde um atalho não chega, use as variantes com `Alt`
([Atalhos de teclado](keybindings.md#teclas-que-o-terminal-não-consegue-enviar-e-o-protocolo-de-teclado-do-kitty)),
ou reatribua o atalho em *Configurações > Teclado e mouse*.

### Área de transferência (ações de copiar)

Uma cópia sempre usa a área de transferência do sistema quando há uma tela local, então estas
células descrevem o caminho `OSC 52` do terminal, que importa por SSH, em contêineres e como
recurso alternativo. Todo teste de cópia do GitPoulpi que usou a área de transferência do sistema
funcionou.

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **funciona** | n/d | não medido (precisa de GPU) |
| **Ghostty** | **funciona** (visto atrás do tmux, zellij, herdr); sem multiplexador: não medido (uma escrita na área de transferência precisa do foco do teclado, que o teste não deu) | n/d | não medido (precisa de GPU) |
| **WezTerm** | **funciona** | **funciona** | não medido (precisa de GPU) |
| **foot** | não medido (uma escrita precisa do foco do teclado, que o teste não deu) | n/d | n/d |
| **Rio** | não medido (mesmo motivo) | **funciona** | não medido (precisa de GPU) |
| **Alacritty** | não medido (mesmo motivo) | **não funciona** (área de transferência inalterada com a configuração padrão; não investigado) | **funciona** |
| **Tabby** | **funciona** | **funciona** | **funciona** |
| **Warp** | não medido | **não funciona** (área de transferência inalterada) | não medido (precisa de GPU) |
| **xterm** | não medido | n/d | **não funciona** (área de transferência inalterada) |
| **iTerm2** | n/d | n/d | **não funciona** até *Settings > General > Selection > Applications in terminal may access clipboard* estar ativado |
| **Terminal.app** | n/d | n/d | **não funciona** (terminal) |
| **Windows Terminal** | n/d | **funciona** (com foco) | n/d |
| **conhost** | n/d | não medido (nenhuma sondagem executada) | n/d |
| **mintty** | n/d | **não funciona** (área de transferência inalterada com as configurações padrão) | n/d |

### Colar

No Linux e no macOS o GitPoulpi solicita o colar entre colchetes, então uma colagem de várias
linhas é uma só colagem. Isso não foi executado terminal por terminal (**não medido** por
terminal). No Windows o console não tem colchetes de colagem, então um terminal que não pergunta
antes entrega o texto como pressionamentos de tecla e cada quebra de linha seria um `Enter`. O
GitPoulpi, portanto, trata como uma só colagem os pressionamentos que se seguem em até 30 ms (um
`Enter` interno, ou oito ou mais caracteres). Teclas seguradas e "abc, `Enter`" digitado à mão não
são colagens.

| Terminal | Windows (colagem de duas linhas no prompt de nova branch) |
|---|---|
| **Windows Terminal** | **funciona**: o WT pergunta "colar várias linhas?" primeiro (o aviso dele), e depois nenhuma branch é criada |
| **WezTerm**, **Rio**, **Alacritty**, **Tabby**, **mintty** | **funciona** (nenhuma branch criada) |
| **Warp** | não medido (não foi possível iniciar o GitPoulpi dentro dele) |
| **conhost** | não medido (`Ctrl+V` e `Shift+Insert` enviados pelo teste não colaram no console clássico) |

### Largura de emojis e ícones

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio**, **Alacritty** | **funciona** (células largas alinhadas; o kitty desenha a engrenagem U+2699 em branco, uma questão conhecida de fonte) | WezTerm, Rio, Tabby, Windows Terminal: **funciona**. Alacritty: **parcial** (emojis como contornos monocromáticos) | Alacritty, Tabby: **funciona**. Os demais: não medido (precisa de GPU) |
| **Tabby** | **funciona** | **funciona** | **funciona** |
| **Warp** | não medido | **funciona** | não medido |
| **xterm** | **parcial** (caixas: a fonte) | n/d | **parcial** (caixas: a fonte) |
| **iTerm2**, **Terminal.app** | n/d | n/d | **funciona** |
| **conhost** | n/d | **parcial**: emojis coloridos desenhados, células largas alinhadas | n/d |
| **mintty** | n/d | **funciona** | n/d |

### Versões testadas

kitty 0.49.2 (a 0.50.0 do macOS não iniciou), Ghostty 1.4.0, WezTerm 20260716 no Linux e
20240203 (a última versão com tag) no Windows, foot 1.28.0, Rio 0.5.28 (Linux, Windows) e
0.5.29 (macOS), Alacritty 0.17.0, Tabby 1.0.238, Warp 0.2026.11.04, xterm 411 (Linux) e 378
(XQuartz), iTerm2 3.7.7, Terminal.app 455.1 (macOS 15.8), Windows Terminal 1.25.2733, mintty
3.8.3. Os terminais mudam rápido: uma versão mais nova que estas pode diferir.

## Por terminal

### Windows

- **Windows Terminal.** O padrão do lançador e o melhor suporte a imagens no Windows: grafo e
  avatares em sixel, OSC 52, bons emojis, e ele responde à consulta de tamanho da célula. Escolha
  uma Nerd Font em *Settings > Profiles > Defaults > Appearance > Font face*. Segure `Shift` para
  selecionar texto enquanto o mouse está capturado. Ele guarda `Ctrl+Shift+N` e `Ctrl+,`: use as
  variantes com `Alt` ou reatribua. O formato do ponteiro não muda.
- **WezTerm.** No `wezterm.lua`, defina `config.enable_kitty_keyboard = true` (versões para Linux
  e macOS) e uma fonte com fallback de emoji, por exemplo
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  As imagens usam o protocolo do iTerm2 e nada precisa ser definido: a última versão com tag,
  20240203-110809 (fev. de 2024), as ancora nas linhas certas (reconferido em novembro de 2026
  com o GitPoulpi atual; antes da correção do ConPTY descrita abaixo, não ancorava). O PR #8092
  do WezTerm upstream acrescenta formatos de ponteiro.
- **Rio** (MSI, exige UAC). As imagens (grafo, avatares, diff de imagem) usam o protocolo do
  iTerm2 com transparência e caem exatamente nas suas células: medido na 0.5.28 com a fonte padrão
  e a 18 e 22,5 px (células de 8x16, 11x21 e 13x26 pixels, sempre pixels inteiros), uma imagem de
  20x10 células e uma de 4x27 células saíram exatas por célula. Nada precisa ser definido. O
  pseudoconsole engole a consulta de tamanho da célula, então o GitPoulpi presume que as células
  do Rio têm o dobro da altura da largura (verdade nos três tamanhos); com uma fonte de outra
  proporção, defina `GITPOULPI_TERMINAL_CELL_PX=WxH` (por exemplo `10x24`) para ter marcadores de
  commit redondos. O Rio não tem formatos de ponteiro em nenhum SO.
- **Tabby.** Grafo e avatares em sixel, copiar e colar funcionam. O Tabby guarda `Ctrl+Shift+N` e
  `Ctrl+,` para si.
- **Alacritty.** Precisa de OpenGL 3.3. Grafo de caracteres; copiar pelo `OSC 52` não estava
  ativado com a configuração padrão.
- **mintty** (Git Bash, MSYS2, Cygwin). Os formatos de ponteiro funcionam (os nomes do próprio
  mintty). As imagens não aparecem (a coluna do grafo fica em branco; o mintty 3.8.3 não desenha
  nenhuma imagem `OSC 1337` atrás do ConPTY, mesmo posicionada por uma sondagem simples) e ele não
  responde a nenhuma consulta de tamanho da célula ali: defina `ui.graph_renderer = "text"`.
- **Warp.** Não medido com o GitPoulpi: não foi possível iniciá-lo pela caixa de entrada do Warp. O
  terminal não tem imagens sixel, iTerm2 nem kitty.
- **conhost, cmd, uma janela do PowerShell.** Grafo de caracteres, diff de imagem em meios-blocos.
  Por padrão, o GitPoulpi iniciado de uma janela assim se reabre no terminal que você escolheu;
  veja "A regra de reabertura".

**O limite do ConPTY.** O Windows executa o programa de todo terminal atrás de um pseudoconsole
(ConPTY). O Windows Terminal e as versões do WezTerm que trazem um ConPTY recente repassam as
sequências de imagem; o que vem dentro do Windows (usado pelo Rio e pelo mintty), e o que vem com
o WezTerm 20240203, não. O que se perde: as sequências DCS (sixel) e APC (gráficos do kitty) são
descartadas, então só as imagens do iTerm2 (`OSC 1337`) chegam, e é por isso que o Rio e o
WezTerm as usam; `CSI 16 t` (tamanho da célula) só é respondida no Windows Terminal e no Tabby. O
que foi corrigido: um `OSC 1337` atravessa o ConPTY na hora e cai no cursor que o ConPTY pintou
por último, e o ConPTY só pinta o cursor enquanto ele está visível. O GitPoulpi mantém o cursor
oculto, então as imagens caíam em linhas antigas (o grafo sobre o rodapé, uma tela rasgada após
rolar). Agora ele mostra o cursor em volta de cada imagem que escreve no Windows, dentro da
mesma escrita, e o oculta de novo; o Rio e o WezTerm 20240203 ancoram toda imagem na sua célula
(`pty_conpty_picture_anchor` lê de volta o movimento do cursor atrás do ConPTY). Nada muda fora
do Windows nem para o sixel.

**Colar no Windows.** Veja a tabela "Colar" acima: o GitPoulpi junta uma rajada de pressionamentos
de tecla em uma só colagem, então uma colagem de várias linhas nunca confirma um prompt nos
terminais testados.

**A regra de reabertura.** Um clique duplo no exe o inicia em uma janela de console clássica
(conhost) só dele. Quando `ui.launch_terminal` nomeia outro terminal (padrão: Windows Terminal), o
GitPoulpi se reabre ali e a primeira janela fecha (`ui.relaunch_in_terminal`). Ele só faz isso
quando está **sozinho** em uma janela de console clássica. Um terminal que executa o exe
diretamente (`wt.exe gitpoulpi.exe`, WezTerm, Rio, Alacritty `-e`, Tabby `run`) é onde você quer
estar, e nada é reaberto; a partir de um shell ele não está sozinho, então também nada acontece.
Desative com `ui.relaunch_in_terminal = false`.

### Linux

- **kitty.** Tudo funciona sem nenhuma configuração. `Shift+clique` é do próprio kitty, então use
  `Alt+clique` para estender uma seleção ([Mouse](mouse-and-gestures.md)).
- **Ghostty.** Tudo funciona sem nenhuma configuração. Se os ícones aparecerem como buracos,
  adicione uma fonte de emoji ou uma Nerd Font a `font-family` (repita a chave para adicionar
  fallbacks).
- **WezTerm.** Imagens pelo protocolo do iTerm2, com a transparência misturada. Defina
  `enable_kitty_keyboard = true` para ter os atalhos completos. Sem formatos de ponteiro.
- **foot.** Grafo e avatares em sixel, formatos de ponteiro, teclado completo, rápido. Defina
  `font=JetBrainsMono Nerd Font:size=11` no `foot.ini`. Copiar pelo `OSC 52` é governado por
  `security.osc52`.
- **Rio.** Imagens pelo protocolo do iTerm2 com transparência, teclado completo. Sem formatos de
  ponteiro (o Rio descarta o `OSC 22`).
- **Alacritty.** Grafo de caracteres, teclado do kitty desde a 0.13. Sem suporte a imagens.
- **Tabby.** Imagens sixel; sem teclado do kitty, sem formatos de ponteiro (Electron, xterm.js).
  Ele guarda `Ctrl+Shift+N` e `Ctrl+,`.
- **xterm.** Inicie-o como `xterm -u8 -ti vt340` para ter sixel (o UTF-8 é necessário para o
  desenho de caixas). Caixas no lugar de emojis: a fonte.
- **Warp.** Não medido: precisa do XWayland e, no teste, decodificou errado teclas sintéticas.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Não fizeram parte da rodada de novembro de 2026.
  Funcionam com caracteres e teclas legadas: use as variantes com `Alt` e `Ctrl+Y` para refazer.
  O VTE ignora o `OSC 52`, então copiar por SSH não chega à área de transferência. Um método de
  entrada (IBus, Fcitx) pode capturar o `Ctrl+Space` que alterna o stage; reatribua-o em
  *Configurações > Teclado e mouse*. O Konsole é reconhecido para sixel pela documentação dele.
- **Wayland, Hyprland, Omarchy.** A dica do ponteiro (`OSC 22`) funciona no kitty, no Ghostty e no
  foot. A entrada do lançador no Omarchy passa por `xdg-terminal-exec`.

### macOS

- **iTerm2.** A 3.7.7 e as mais recentes desenham imagens pelo protocolo do kitty, sem pergunta.
  Configurações que ajudam:
  - *Settings > General > Selection > Applications in terminal may access clipboard*: necessária
    para copiar pelo `OSC 52` (senão o iTerm2 mostra "the terminal attempted to access the
    clipboard but it was denied").
  - *Profiles > Keys > Left Option key: Esc+*: necessária para os atalhos com `Alt`.
  - Se você definir `ui.image_protocol = "iterm2"` (ou usar uma versão anterior à 3.7.7), marque
    "Remember my choice" no diálogo "Allow Terminal-Initiated Display?". As imagens então têm um
    fundo opaco (um retângulo visível se o seu fundo for diferente do do tema;
    `ui.graph_renderer = "text"` o remove).
  - O protocolo de teclado do kitty vem ativado por padrão (3.6.9 e mais recentes).
- **Terminal.app.** Somente 256 cores: o GitPoulpi escolhe `ui.color_depth = 256` sozinho e os
  temas ficam mais sem graça. Sem imagens, sem teclado do kitty, sem área de transferência pelo
  `OSC 52`. Marque *Use Option as Meta key* para os atalhos com `Alt`.
- **Alacritty, Tabby.** Alacritty: grafo de caracteres, teclado do kitty, cópia por `OSC 52`;
  defina `option_as_alt = "Both"` para os atalhos com `Alt`. Tabby: grafo e avatares em sixel.
- **Option como Meta.** O Option compõe caracteres (Option+B digita um sinal de integral), a menos
  que o terminal o transforme em Meta: iTerm2 *Left Option key: Esc+*, Terminal.app *Use Option as
  Meta key*, Alacritty `option_as_alt`. Os atalhos com `Alt` do GitPoulpi precisam disso, e em
  lugar nenhum vem ativado por padrão.
- **kitty, Ghostty, WezTerm, Rio, Warp**: não medidos no macOS (precisam de GPU). Espere o
  comportamento do Linux.
- **xterm** (XQuartz): `xterm -u8 -ti vt340`; o sixel precisa de `ui.image_protocol = "sixel"`.

### VS Code e Warp

O terminal do VS Code (`TERM_PROGRAM=vscode`) não fez parte da rodada. Ele é reconhecido pelos
atributos do dispositivo quando `terminal.integrated.enableImages` está ativado; senão o GitPoulpi
desenha caracteres; `ui.image_protocol = "sixel"` força as imagens. Warp: veja as notas por SO
acima.

## Multiplexadores

tmux, zellij, screen, herdr, psmux e os painéis dos próprios terminais ficam entre o GitPoulpi e o
terminal e mudam o que passa. Em resumo: o tmux repassa sixel e, com `allow-passthrough on`,
formatos de ponteiro e imagens do kitty; zellij e screen não repassam nem imagens nem formatos de
ponteiro; o herdr repassa imagens do kitty. Todos os detalhes e a configuração a acrescentar estão
em [Multiplexadores](multiplexers.md).

## O que se perde, e como

| Falta | O que você recebe no lugar |
|---|---|
| Protocolo de imagens | Grafo de caracteres. O avatar vira o selo colorido de iniciais (uma letra), seja qual for o valor de `ui.avatar_fallback`: um identicon espremido em células de texto fica irreconhecível. O diff de imagem e a Visualização de arquivo usam meios-blocos coloridos, ou um cartão de informações. |
| True color | As cores são mapeadas para a paleta de 256 ou de 16 cores (`ui.color_depth`). |
| Protocolo de teclado do kitty | Teclas legadas; alguns atalhos chegam como outra tecla. As Configurações avisam, e você reatribui ou usa as variantes com `Alt`. |
| OSC 52 | Localmente nada muda (a área de transferência do sistema é usada). Por SSH ou sem tela, as ações de copiar não colocam nada na área de transferência: selecione com o mouse (segurando `Shift`) e use o copiar do próprio terminal. |
| OSC 22 | O ponteiro não muda; o divisor ou botão sob o mouse ainda se acende. |
| Fonte de emoji | Caixas ou buracos no lugar dos ícones; o grafo e o texto continuam funcionando. Instale uma Nerd Font ou uma fonte de emoji e liste-a como fallback. |

## Configurações que vale conhecer

| Configuração | Uso |
|---|---|
| `ui.image_protocol` | `auto` (padrão), `kitty`, `iterm2`, `sixel`, `none`. Use `none` se aparecer lixo onde deveriam estar as imagens. Reiniciar. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` está sempre correto (use-o onde vale o limite do ConPTY). Reiniciar. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | `auto` ou `off`, se um terminal ou multiplexador se comportar mal. Reiniciar. |
| `ui.text_cursor` | `terminal` (padrão) ou `drawn`. |
| `ui.mouse_enabled` | Desativado, deixa o mouse para o terminal. Reiniciar. |
| `ui.launch_terminal`, `ui.relaunch_in_terminal` | O terminal que os atalhos do SO abrem, e se um clique duplo reabre o GitPoulpi ali ([Configurações](settings.md)). |

Veja também [Multiplexadores](multiplexers.md), [Imagens](image-view.md),
[Atalhos de teclado](keybindings.md) e [Solução de problemas](troubleshooting.md).

## Fontes

Documentação conferida em outubro de 2026, comportamento testado em novembro de 2026. Fontes primárias, salvo indicação em contrário.

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [formatos de ponteiro](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [sequências de escape](https://wezterm.org/escape-sequences.html), [pull request do OSC 22](https://github.com/wezterm/wezterm/pull/6292), [versões](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [versões](https://github.com/alacritty/alacritty/releases)
- Konsole: [merge request do OSC 52](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [merge request do teclado kitty](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [issue do OSC 52](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [issue do teclado kitty](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [downloads](https://iterm2.com/downloads.html); teclado kitty a partir da [matriz do terminfo.dev](https://terminfo.dev/extensions/kitty-keyboard-protocol) (secundária)
- Windows Terminal: [anúncio da 1.22](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (secundária), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [suporte a sequências de escape](https://rioterm.com/docs/escape-sequence-support) (sem linha de `OSC 22`), [recursos](https://rioterm.com/docs/features), [teclado kitty](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [aplicativos em tela cheia](https://docs.warp.dev/terminal/more-features/full-screen-apps); OSC 52 a partir de uma [análise do aviso de segurança do OSC 52 no Warp](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) (secundária)
- VS Code: [terminal avançado](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [sequências de controle](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [dicas](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (sem true color): [discussão da Apple](https://discussions.apple.com/thread/255315820) (secundária)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES); zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Sites de comparação (secundários, usados só para conferência cruzada): [Terminal Trove](https://terminaltrove.com/compare/terminals/)
