[English](../image-view.md) · [Čeština](../cs/image-view.md) · [Deutsch](../de/image-view.md) · [Español](../es/image-view.md) · [Français](../fr/image-view.md) · [Italiano](../it/image-view.md) · [日本語](../ja/image-view.md) · [한국어](../ko/image-view.md) · [Nederlands](../nl/image-view.md) · [Polski](../pl/image-view.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/image-view.md) · [Русский](../ru/image-view.md) · [Svenska](../sv/image-view.md) · [Türkçe](../tr/image-view.md) · [Українська](../uk/image-view.md) · [简体中文](../zh-CN/image-view.md)

# Imagens na visualização de diff e na Visualização de arquivo

Um arquivo binário que é uma imagem é mostrado como imagem, e não como o aviso
“(no textual changes - binary content...)”. A visualização de Diff mostra o
**antes** e o **depois**; a Visualização de arquivo mostra o arquivo como ele
é; o editor de merge mostra os lados de uma imagem em conflito. Tudo aqui é
somente leitura: olhar uma imagem nunca altera o repositório, então não há nada
a desfazer.

Desative em **Configurações > Imagens > Pré-visualizar imagens**
(`ui.image_view`): o aviso binário simples volta. Uma compilação personalizada
sem a feature `image-view` não tem nenhum visualizador e a página Imagens fica
só com a lista de editores (veja
[Compilações personalizadas](custom-builds.md)).

## Quais arquivos

PNG e JPEG sempre; GIF, WebP (o primeiro quadro), BMP e ICO/CUR quando a build
tem `image-view`. TIFF, AVIF, HEIC, PSD e RAW são reconhecidos mas não
decodificados: uma ficha de informações dá o formato, o tamanho e as dimensões.
SVG é texto e mantém o diff de texto. Uma imagem acima de **Tamanho máx. da
pré-visualização (MiB)** (32 MiB por padrão) ou acima de 64 megapixels mostra
uma ficha de informações. Um arquivo guardado com Git LFS é mostrado quando o
objeto dele está no armazenamento local; caso contrário, a ficha diz para fazer
`LFS > Pull` antes. A transparência é desenhada sobre um tabuleiro de xadrez.

## Como é desenhada

Em um terminal com protocolo de imagem (Kitty e Ghostty, WezTerm e iTerm2, Rio,
mintty) a imagem é uma imagem de verdade. Em qualquer outro lugar (Windows
Terminal simples, tmux, um terminal que o GitPoulpi não conhece) ela é desenhada
com caracteres de meio bloco coloridos, com metade da resolução vertical mas
legível em todo lugar; **Sem protocolo de imagem** pode ser definido como *Só a
ficha de informações*. **Protocolo de imagem** na página **Log e grafo** força um.

## A visualização de Diff

Abra o diff de um arquivo de imagem (o de um commit, a metade unstaged ou staged
do WIP). O cabeçalho nomeia os dois arquivos
(`PNG 512x512 . 34 KiB -> PNG 640x480 . 41 KiB`), depois vêm os botões de zoom,
**Ajustar**, **1:1** e **Editar em**.

- **Dividida | Em linha** à direita do cabeçalho, ou `m`: lado a lado, ou uma
  imagem de cada vez. **Tab**, ou um clique em *Antes | Depois*, mostra a outra
  na visualização em linha. Uma divisão estreita demais para dois painéis é
  desenhada em linha sem alterar a sua configuração (`ui.diff_layout`).
- O **divisor** de uma divisão se move arrastando-o ou com `Alt+Right` /
  `Alt+Left` (o painel Antes ganha / cede espaço) e volta ao meio com um clique
  duplo ou `Alt+0`. A proporção é lembrada durante a sessão.
- Um arquivo novo mostra um painel com a legenda *Arquivo novo*, um excluído
  *Arquivo excluído*; ambos ocultam o alternador. Uma imagem untracked
  selecionada nas alterações em andamento é pré-visualizada como arquivo novo.
- O lado Antes é o commit pai (diff de commit), `HEAD` (staged) ou o índice
  (unstaged); o lado Depois é o commit, o índice ou o arquivo em disco. Se o
  arquivo em disco muda enquanto o diff está aberto (você o salvou em um editor,
  uma ferramenta o reescreveu), a imagem acompanha em cerca de meio segundo.

## A Visualização de arquivo

Abra uma imagem com **Visualização de arquivo** no menu de um arquivo ou na
barra do diff: um painel, com o arquivo na revisão (ou em disco, para as
alterações em andamento).

## Zoom e rolagem

| | |
|---|---|
| `+` / `-` (ou `=`) | Aumentar / diminuir o zoom, em torno do centro do painel |
| `0` | **Ajustar** (o padrão): a imagem inteira, e ela continua ajustada quando a janela ou o divisor é redimensionado |
| `1` | Tamanho real (1:1) |
| Setas, `h j k l` | Rolar um décimo do painel; com `Shift` um painel inteiro; `PageUp`/`PageDown`, `Home`/`End` |
| Roda | Rolar na vertical; `Shift`+roda na horizontal; `Ctrl`+roda dá zoom em torno do ponteiro |
| Arrastar | Agarrar a imagem e movê-la; as barras de rolagem nas laterais funcionam como em todo lugar |

Uma visualização com zoom mantém a escala e o ponto no seu centro quando o
painel é redimensionado; Ajustar continua Ajustar. Na visualização de Diff os
dois painéis compartilham um zoom e uma posição de rolagem (**Sincronizar zoom e
rolagem antes/depois**, ativado por padrão), de modo que a mesma parte das duas
imagens fica à vista; quando elas diferem em tamanho, o zoom é mantido em
relação a cada uma. A partir de **Zoom pixelado a partir de** (2x por padrão) a
imagem é ampliada em pixels quadrados em vez de suavizada, o que serve para
ícones e pixel art.

## Editar em

O botão **Editar em** do cabeçalho lista os editores de imagem encontrados nesta
máquina (GIMP, Krita, Inkscape, Photoshop, Paint.NET, Preview, ... e o aplicativo
padrão do sistema); `o` executa o padrão e `Shift+O` abre a lista. Ele também
está no menu de botão direito de uma imagem nas listas de arquivos, e na paleta
de comandos (*Imagem: editar em...*). A lista e o padrão são definidos em
**Configurações > Imagens**, do mesmo jeito que funciona [Abrir em](open-in.md).

Editar o arquivo em disco (o lado das alterações em andamento) é direto: o editor
salva no lugar e a imagem é atualizada. Qualquer outra versão (a de um commit, o
índice, um arquivo excluído) é gravada em uma **cópia temporária somente
leitura**, com um aviso na linha de status, para que um editor não possa alterar
o repositório.

Os aplicativos que o sistema registra para o tipo da imagem (sua lista *Abrir com*) vêm depois dos seus editores na lista, sem repetir um programa que já está lá. Os outros arquivos binários têm a mesma lista em [Abrir em](open-in.md).

## No editor de merge

Uma imagem em conflito mostra seu cartão com as imagens **A**, **B** e (quando
há) **Base** nas caixas, lado a lado, cada uma ajustada à sua caixa e somente
leitura (sem zoom ali; as imagens servem só para ajudar a escolher). Os botões e
as teclas não mudam: **Manter A** (`a`), **Aceitar B** (`b`), **Aceitar base** (`o`),
**Abrir na ferramenta de merge** (`m`). As pré-visualizações exigem uma janela
larga o bastante (cerca de 70 colunas) e algumas linhas livres; mais estreito, o
cartão é o de texto simples. Veja
[Resolvendo conflitos de merge](merge-conflicts.md).

## Teclas e configurações

As teclas estão em [Atalhos de teclado](keybindings.md#visualização-de-imagem); todas podem
ser remapeadas e alcançadas pela paleta (`Ctrl+P`, digite *Imagem*). As opções
estão na página **Imagens** das [Configurações](settings.md#imagens) e na
[referência de configuração](config-reference.md) (`ui.image_view*`,
`ui.image_editors`).
