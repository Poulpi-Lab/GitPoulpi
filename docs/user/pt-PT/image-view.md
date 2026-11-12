[English](../image-view.md) · [Čeština](../cs/image-view.md) · [Deutsch](../de/image-view.md) · [Español](../es/image-view.md) · [Français](../fr/image-view.md) · [Italiano](../it/image-view.md) · [日本語](../ja/image-view.md) · [한국어](../ko/image-view.md) · [Nederlands](../nl/image-view.md) · [Polski](../pl/image-view.md) · [Português (Brasil)](../pt-BR/image-view.md) · **Português (Portugal)** · [Русский](../ru/image-view.md) · [Svenska](../sv/image-view.md) · [Türkçe](../tr/image-view.md) · [Українська](../uk/image-view.md) · [简体中文](../zh-CN/image-view.md)

# Imagens na vista Diff e na vista de ficheiro

Um ficheiro binário que seja uma imagem é mostrado como imagem, e não com o
aviso "(no textual changes - binary content...)". A vista Diff mostra o
**antes** e o **depois**; a vista de ficheiro mostra o ficheiro tal como está; o
editor de merge mostra os lados de uma imagem em conflito. Tudo aqui é só de
leitura: ver uma imagem nunca altera o repositório, por isso não há nada a
desfazer.

Desative-o em **Definições > Imagens > Pré-visualizar imagens**
(`ui.image_view`): volta o aviso binário habitual. Uma compilação personalizada
sem a funcionalidade `image-view` não tem visualizador nenhum e a página Imagens
fica apenas com a lista de editores (veja
[Compilações personalizadas](custom-builds.md)).

## Que ficheiros

PNG e JPEG sempre; GIF, WebP (a sua primeira imagem), BMP e ICO/CUR quando a
compilação tem `image-view`. TIFF, AVIF, HEIC, PSD e RAW são reconhecidos mas
não descodificados: uma ficha de informações indica o formato, o tamanho e as
dimensões. O SVG é texto e mantém o diff de texto. Uma imagem acima do
**Tamanho máx. da pré-visualização (MiB)** (32 MiB por predefinição) ou acima de
64 megapíxeis mostra antes uma ficha de informações. Um ficheiro guardado com
Git LFS é mostrado quando o seu objeto está no armazém local; caso contrário, a
ficha diz para fazer primeiro `LFS > Pull`. A transparência é desenhada sobre um
tabuleiro de xadrez.

## Como é desenhada

Num terminal com protocolo de imagem (Kitty e Ghostty, WezTerm e iTerm2, Rio,
mintty, e os terminais sixel Windows Terminal, foot e Tabby) a imagem é uma imagem a sério. Em qualquer outro (a consola simples do Windows,
o zellij, um terminal que o GitPoulpi não conheça) é desenhada com
caracteres de meio bloco coloridos, com metade da resolução vertical mas legível
em todo o lado; **Sem protocolo de imagem** pode ser definido como *Só a ficha
de informações*. **Protocolo de imagem** na página **Log e grafo** força um.

## A vista Diff

Abra o diff de um ficheiro de imagem (o de um commit, a metade unstaged ou
staged do WIP). O cabeçalho indica os dois ficheiros
(`PNG 512x512 . 34 KiB -> PNG 640x480 . 41 KiB`), depois os botões de zoom,
**Ajustar**, **1:1** e **Editar em**.

- **Dividida | Em linha** à direita do cabeçalho, ou `m`: lado a lado, ou uma
  imagem de cada vez. **Tab**, ou um clique em *Antes | Depois*, mostra a outra
  na vista em linha. Uma divisão demasiado estreita para dois painéis é
  desenhada em linha sem alterar a sua definição (`ui.diff_layout`).
- O **divisor** de uma divisão move-se arrastando-o ou com `Alt+Right` /
  `Alt+Left` (o painel do antes ganha / cede espaço) e volta ao meio com um duplo
  clique ou `Alt+0`. A proporção é recordada durante a sessão.
- Um ficheiro novo mostra um painel com a legenda *Ficheiro novo*, um eliminado
  *Ficheiro eliminado*; ambos escondem o seletor. Uma imagem não rastreada
  selecionada nas alterações em curso é pré-visualizada como ficheiro novo.
- O lado do antes é o commit pai (diff de commit), `HEAD` (staged) ou o índice
  (unstaged); o lado do depois é o commit, o índice ou o ficheiro em disco. Se o
  ficheiro em disco mudar enquanto o diff está aberto (guardou-o a partir de um
  editor, uma ferramenta reescreveu-o), a imagem acompanha em cerca de meio
  segundo.

## A vista de ficheiro

Abra uma imagem com **Vista de ficheiro** a partir do menu de um ficheiro ou da
barra do diff: um só painel, o ficheiro na revisão (ou em disco, no caso das
alterações em curso).

## Zoom e deslocação

| | |
|---|---|
| `+` / `-` (ou `=`) | Aproximar / afastar, em torno do centro do painel |
| `0` | **Ajustar** (a predefinição): a imagem inteira, e continua ajustada quando a janela ou o divisor são redimensionados |
| `1` | Tamanho real (1:1) |
| Setas, `h j k l` | Deslocar um décimo do painel; com `Shift` um painel inteiro; `PageUp`/`PageDown`, `Home`/`End` |
| Roda | Desloca na vertical; `Shift`+roda na horizontal; `Ctrl`+roda faz zoom em torno do ponteiro |
| Arrastar | Agarra a imagem e move-a; as barras de deslocação nos lados funcionam como em todo o lado |

Uma vista com zoom mantém a escala e o ponto do seu centro quando o painel é
redimensionado; Ajustar continua Ajustar. Na vista Diff os dois painéis partilham
um só zoom e uma só posição de deslocação (**Sincronizar zoom e deslocação
antes/depois**, ativado por predefinição), por isso a mesma parte de ambas as
imagens está à vista; quando diferem no tamanho, o zoom mantém-se relativo a
cada uma. A partir de **Zoom pixelizado a partir de** (2x por predefinição) a
imagem é ampliada em píxeis quadrados em vez de suavizada, o que convém a ícones
e pixel art.

## Editar em

O botão **Editar em** do cabeçalho lista os editores de imagem encontrados nesta
máquina (GIMP, Krita, Inkscape, Photoshop, Paint.NET, Preview, ... e a
aplicação predefinida do sistema); `o` executa o predefinido e `Shift+O` abre a
lista. Também está no menu de clique com o botão direito de uma imagem nas
listas de ficheiros e na paleta de comandos (*Imagem: editar em...*). A lista e o
predefinido definem-se em **Definições > Imagens**, tal como funciona
[Abrir em](open-in.md).

Editar o ficheiro em disco (o lado das alterações em curso) é direto: o editor
guarda no próprio ficheiro e a imagem atualiza. Qualquer outra versão (a de um
commit, o índice, um ficheiro eliminado) é escrita numa **cópia temporária só de
leitura**, com um aviso na barra de estado, para que um editor não possa alterar
o repositório.

As aplicações que o sistema regista para o tipo da imagem (a sua lista *Abrir com*) seguem os seus editores na lista, sem repetir um programa que já lá está. Os outros ficheiros binários têm a mesma lista em [Abrir em](open-in.md).

## No editor de merge

Uma imagem em conflito mostra a sua ficha com as imagens **A**, **B** e (quando
existe) **Base** nas caixas, lado a lado, cada uma ajustada à sua caixa e só de
leitura (sem zoom aí; as imagens servem apenas para o ajudar a escolher). Os
botões e as teclas não mudam: **Manter A** (`a`), **Aceitar B** (`b`),
**Aceitar Base** (`o`), **Abrir na ferramenta de merge** (`m`). As
pré-visualizações precisam de uma janela suficientemente larga (cerca de 70
colunas) e de algumas linhas livres; mais estreita, a ficha é a de texto simples.
Veja [Resolver conflitos de merge](merge-conflicts.md).

## Teclas e definições

As teclas estão nos [Atalhos de teclado](keybindings.md#vista-de-imagem); todas são
reatribuíveis e acessíveis a partir da paleta (`Ctrl+P`, escreva *Imagem*). As
opções estão na página **Imagens** das [Definições](settings.md#imagens) e na
[referência de configuração](config-reference.md) (`ui.image_view*`,
`ui.image_editors`).
