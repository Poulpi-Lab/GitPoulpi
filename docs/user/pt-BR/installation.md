[English](../installation.md) · [Čeština](../cs/installation.md) · [Deutsch](../de/installation.md) · [Español](../es/installation.md) · [Français](../fr/installation.md) · [Italiano](../it/installation.md) · [日本語](../ja/installation.md) · [한국어](../ko/installation.md) · [Nederlands](../nl/installation.md) · [Polski](../pl/installation.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/installation.md) · [Русский](../ru/installation.md) · [Svenska](../sv/installation.md) · [Türkçe](../tr/installation.md) · [Українська](../uk/installation.md) · [简体中文](../zh-CN/installation.md)

# Instalação

O GitPoulpi tem três executáveis:

| Executável | O que é |
|---|---|
| `gitpoulpi-tui` | A interface de terminal. |
| `gitpoulpi` | A interface de linha de comando (`--json` para scripts). Sem subcomando, abre a interface. |
| `gitpoulpi-mcp` | O servidor MCP para agentes de IA. Veja [MCP e agentes](mcp-and-agents.md). |

Ao lado deles, o arquivo contém `gitpoulpi-launch`, um programa minúsculo que os atalhos executam para mostrar uma janela do GitPoulpi já aberta (`gitpoulpi launch`). No Windows ele não tem janela de console, então um clique nunca faz uma piscar. É instalado, atualizado e removido junto com os outros.

## O que você precisa

- Um **`git` do sistema** no seu `PATH`. O GitPoulpi usa a libgit2 para a maioria
  das operações e o seu `git` para o resto.
  É necessária a versão **2.34 ou mais recente**. No macOS, o `git` das Command Line Tools da Apple funciona, e o do Homebrew também.
- Um **terminal** com suporte a mouse. kitty, Ghostty, WezTerm e iTerm2 (e também Windows Terminal e foot, via sixel)
  desenham o grafo de commits como imagens. Os outros terminais (o
  console do Windows, tmux e a maioria dos demais) mostram um grafo de
  caracteres. Veja
  [Escolhendo um terminal](terminals.md).
- Uma **Nerd Font** ou uma fonte com emojis, para os ícones da barra de
  ferramentas e da barra lateral.
- Opcional: **`git-lfs`** 3.0 ou mais recente, para repositórios com Git LFS.
  Sem ele, os arquivos LFS aparecem como pequenos arquivos de ponteiro e o
  GitPoulpi explica o motivo. Veja [Git LFS](lfs.md) para saber como instalá-lo.
- Opcional: um **auxiliar de credenciais** (Git Credential Manager, libsecret, `pass`,
  Keychain), para guardar tokens de hospedagem e chaves de API de IA. Veja
  [Integrações de hospedagem e de rastreadores de issues](hosting-integration.md).

## A partir de um arquivo de release

Um arquivo de release é `gitpoulpi-<version>-<target>.zip` (Windows) ou
`.tar.gz` (Linux, macOS), com um arquivo `.sha256` ao lado. Confira o arquivo
com o `.sha256`, descompacte-o e coloque os três executáveis em uma pasta do seu
`PATH`. O `INSTALL.txt` dentro do arquivo traz os mesmos passos.

## Com um instalador

Cada release também traz um instalador por sistema operacional. Os dois fazem as mesmas
escolhas (atalhos, PATH, um alias de comando como `gp`) chamando o próprio `gitpoulpi`
instalado, o código por trás de Configurações > Geral > Lançador, então o que o
instalador configura e o que as Configurações mostram sempre coincidem.

### Windows: `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

Ele instala para o usuário atual, sem pedir administrador, em
`%LOCALAPPDATA%\Programs\GitPoulpi` (o primeiro diálogo oferece uma instalação para todos os
usuários e a escolha da pasta). Caixas de seleção: atalho no menu Iniciar, atalho na
Área de trabalho, adicionar ao PATH. Uma página pergunta o **alias de comando**: `gp` por padrão,
qualquer nome que você quiser, vazio para nenhum. O PowerShell já usa `gp` para
`Get-ItemProperty`, então escolha outro nome lá ou use `gitpoulpi`.

O instalador copia os três executáveis e o marcador `.gitpoulpi-dist`, então
[Atualizar agora](#atualizando-no-lugar) funciona exatamente como com o zip. Uma instalação por
usuário tem permissão de escrita, então pode se atualizar sozinha; uma instalação para todos os usuários em
Program Files precisa de elevação para ser substituída, então reinstale-a com o novo instalador.
Desinstale em Configurações > Aplicativos do Windows: ele executa `gitpoulpi uninstall` (alias,
entradas do PATH, atalhos) e exclui os arquivos, inclusive as sobras `.old` do atualizador.
Suas configurações e repositórios ficam.

### Linux e macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

Ele escolhe o arquivo para o seu sistema e a sua CPU, confere o `.sha256`, instala em
`~/.local/opt/gitpoulpi` e cria links de `gitpoulpi` e `gitpoulpi-mcp` em
`~/.local/bin`. Não precisa de root e nada fora da sua pasta pessoal é tocado.
Sem `--yes` ele pergunta sobre o alias (padrão `gp`), o PATH e os atalhos.

| Opção | Efeito |
|---|---|
| `--prefix DIR` | Instala os arquivos do programa em `DIR` (absoluto). |
| `--version X.Y.Z` | Instala esta release em vez da mais recente. |
| `--archive FILE` | Usa um arquivo local; `FILE.sha256` precisa estar ao lado dele (`--no-verify` dispensa isso). Nenhuma rede é usada. |
| `--alias NAME` / `--no-alias` | Faz `NAME` iniciar o GitPoulpi (`gitpoulpi alias create`), ou nenhum. |
| `--no-path` | Não adiciona o GitPoulpi ao PATH dos novos shells (senão, `gitpoulpi path add`). |
| `--desktop` | Também cria os atalhos do menu de aplicativos e da Área de trabalho. |
| `--yes` | Nunca pergunta: valem as opções e os padrões (alias `gp`, PATH sim, atalhos não). |
| `--uninstall` | Executa `gitpoulpi uninstall`, depois remove os links e a pasta. |

A consulta da release mais recente exige que o repositório seja público; para um privado,
baixe o arquivo em um navegador e use `--archive`.

## A partir do código-fonte

Você precisa de uma toolchain Rust estável recente (<https://rustup.rs>).

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

Os executáveis ficam em `target/release/`. O perfil `release` compila
rapidamente. Para os executáveis menores e mais rápidos, use o perfil `dist`.
Ele demora mais para compilar:

```
cargo build --profile dist
```

Os scripts `build-opti` na raiz do repositório executam essa compilação para os
três executáveis e mostram os tamanhos deles:

```
./build-opti.sh                  # Linux, macOS
build-opti                       # Windows (cmd ou PowerShell)
```

Para compilar apenas os idiomas e as funções que você usa, veja
[Compilações personalizadas](custom-builds.md).

## Gerar um arquivo de release você mesmo

```
cargo run -p xtask -- dist
```

Isso compila o perfil `dist` e grava
`target/dist/gitpoulpi-<version>-<target>.zip` (Windows) ou `.tar.gz` (outros
sistemas). O arquivo contém os três executáveis, `INSTALL.txt`, `README.md`,
`LICENSE` e `CHANGELOG.md`. Um arquivo `.sha256` é gravado ao lado dele.

## Atualizações

A barra de título mostra a versão em execução (`v1.0.0`) entre o nome do aplicativo
e o caminho do repositório. Clique nela, ou execute **Ajuda: Notas da versão** na paleta de comandos (`Ctrl+P`): uma aba Notas da versão abre com o changelog, rolado
até a versão em execução.

Uma vez por dia, em segundo plano, o GitPoulpi pergunta ao GitHub se existe uma release
mais nova (`updates.mode`, [Configurações > Sincronização](settings.md#sincronização)).
Quando existe, o chip passa a dizer `v0.2.0 disponível` na cor de destaque e o clique
abre a aba Notas da versão com as notas dessa release no topo. A aba tem
três botões, cada um com uma tecla e um comando da paleta:

- **Baixar** (`d`, *Atualizações: baixar*) abre a página da release no navegador.
- **Ignorar esta versão** (`s`, *Atualizações: ignorar esta versão*) oculta o aviso
  até aparecer uma release mais nova.
- **Verificar agora** (`r`, *Atualizações: verificar agora*) pergunta de novo na hora.

`updates.mode = "off"` não faz requisição nenhuma. Sem conexão de rede,
ou enquanto o repositório é privado e nenhum token do github.com está guardado, a verificação
falha em silêncio (uma linha no registro do aplicativo, nada na interface).
`GITPOULPI_UPDATE_API` aponta a verificação para outra raiz de API (usado pelos testes).

### Atualizando no lugar

Uma build instalada a partir de um arquivo de release pode se substituir sozinha. O arquivo traz
um pequeno arquivo `.gitpoulpi-dist` ao lado dos executáveis; uma build em execução só é
"atualizável" quando esse arquivo está ao lado do seu executável (e nomeia a sua
plataforma). Uma build feita com `cargo`, uma build personalizada copiada para outro lugar, ou um
pacote que deixou o arquivo de fora nunca se atualiza sozinha: a aba Notas da versão diz isso
e oferece só **Baixar**.

Para uma build atualizável, a aba acrescenta:

- **Atualizar agora** (`u`, *Atualizações: atualizar agora*) baixa o arquivo e o seu
  `.sha256` em segundo plano (uma tarefa no indicador da linha de status, com a sua
  porcentagem, cancelável como qualquer tarefa), confere o hash, descompacta o arquivo
  e coloca cada arquivo novo ao lado do antigo como `<name>.new`. Nada que esteja rodando
  é tocado. No modo `auto`, o mesmo acontece sozinho assim que uma verificação encontra uma
  release mais nova.
- **Reiniciar para atualizar** (`R`, *Atualizações: reiniciar para atualizar*; o chip diz o mesmo
  quando o download está preparado) é sempre uma ação sua, nunca automática. Ela
  para o daemon independente da sessão (um embutido sai junto com a janela),
  troca os arquivos e inicia a nova versão com os mesmos argumentos e a mesma
  sessão.
- **Voltar para a versão anterior** (`v`, *Atualizações: voltar à versão anterior*) troca de volta os
  arquivos anteriores e reinicia. Os arquivos da versão anterior ficam como
  `<name>.old` até a nova versão rodar por um minuto, depois são
  excluídos, então o botão existe para essa primeira execução. Ele não faz parte do
  histórico de desfazer do git: altera os arquivos do próprio GitPoulpi, não um repositório.

Como os arquivos são substituídos: no Linux e no macOS cada arquivo novo é renomeado sobre o
antigo (atômico; um programa em execução mantém a cópia antiga até sair). No
Windows um `.exe` em execução não pode ser sobrescrito, então o antigo é renomeado para
`<name>.old` e o novo é colocado no lugar; um `.old` que ainda esteja rodando é excluído
em uma inicialização posterior. A troca é tudo ou nada: se outro programa mantém um dos
executáveis aberto (um cliente MCP executando o `gitpoulpi-mcp`, por exemplo), o GitPoulpi cita o
arquivo, não altera nada e deixa o download preparado para uma nova tentativa.

O que é recusado, cada caso com uma mensagem: uma release sem arquivo `.sha256`, um hash
que não confere, um arquivo cujo nome não é o desta plataforma
(`gitpoulpi-<version>-<target>.<zip|tar.gz>`), um arquivo cujo marcador cita
outra versão ou outra plataforma, e qualquer link de download fora destes hosts: os do GitHub,
`github.com`, `api.github.com`, `objects.githubusercontent.com`,
`release-assets.githubusercontent.com` e `github-releases.githubusercontent.com`
(a primeira requisição e cada redirecionamento; um redirecionamento para qualquer outro lugar interrompe o
download). Os arquivos de um repositório privado exigem uma sessão de navegador, então o
download falha ali até o repositório ficar público.

## Verificar se funciona

```
gitpoulpi --version
cd path/to/a/git/repository
gitpoulpi-tui
```

`--version` mostra a versão e o commit a partir do qual a build foi feita. Depois
siga os [Primeiros passos](getting-started.md).

## Desinstalar

`gitpoulpi uninstall` (o que os dois instaladores executam) remove os aliases de comando,
as entradas do PATH e os atalhos que o GitPoulpi criou, depois os arquivos do programa de uma
instalação de release; ele recusa uma build sem o marcador `.gitpoulpi-dist`
(uma build do cargo), e `--keep-files` deixa os arquivos do programa. Ele não está
disponível para agentes MCP. Sem um instalador, exclua os três executáveis.
O GitPoulpi também guarda estes arquivos:

| O quê | Onde |
|---|---|
| Configurações (`config.toml`) e sessões | A pasta de configuração do seu sistema. `GITPOULPI_CONFIG_DIR` a altera. |
| Estado por repositório | `.gitpoulpi/` dentro de cada repositório. |
| Caches (avatares) | A pasta de cache do seu sistema. |
| Tokens e chaves de API | Seu auxiliar de credenciais. **Configurações > SSH e credenciais > Esquecer todos os usuários e senhas** os remove. |

Veja a [Referência de configuração](config-reference.md) para os caminhos exatos.
