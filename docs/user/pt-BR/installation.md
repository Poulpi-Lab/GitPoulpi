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
`PATH` (mantenha o `gitpoulpi-updater` e o marcador `.gitpoulpi-dist` ao lado deles:
o marcador diz `channel=archive`, então a build pode se atualizar sozinha, veja
[Atualizações](updates.md)). O `INSTALL.txt` dentro do arquivo traz os mesmos passos.

### Verificar um download

Todo arquivo de release é assinado com a chave [minisign](https://jedisct1.github.io/minisign/) do projeto, e o GitPoulpi confere essa assinatura por conta própria antes de instalar uma atualização. Para um arquivo que você baixou à mão, o `.minisig` ao lado dele é a assinatura. A chave pública (id da chave `B461FFCBF6F6A4F2`) está no repositório como `packaging/minisign.pub`:

```
untrusted comment: minisign public key B461FFCBF6F6A4F2
RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

```
minisign -Vm gitpoulpi-<version>-<target>.tar.gz -P RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

Leia o comentário confiável que o comando imprime: ele precisa ser `file:<o nome do arquivo> version:<a versão que você espera>`. Uma assinatura válida que nomeia outro arquivo ou outra versão não é uma assinatura para o seu.

Quando a release foi construída com atestados de procedência de build (a variável `ATTESTATIONS` do repositório), a GitHub CLI confere onde um arquivo foi construído:

```
gh attestation verify gitpoulpi-<version>-<target>.tar.gz --repo <owner>/gitpoulpi
```

O arquivo `.sha256` só detecta um download danificado. A assinatura é o que mostra que o arquivo é do projeto.

## Com um instalador

Cada release também traz um instalador por sistema operacional. Os dois fazem as mesmas
escolhas (atalhos, PATH, um alias de comando como `gp`) chamando o próprio `gitpoulpi`
instalado, o código por trás de Configurações > Geral > Lançador, então o que o
instalador configura e o que as Configurações mostram sempre coincidem.

### Windows: `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

O primeiro diálogo pergunta para quem é a instalação:

| Escolha | Onde | Direitos de administrador | O que significa para as atualizações |
|---|---|---|---|
| **Só para mim** (padrão) | `%LOCALAPPDATA%\Programs\GitPoulpi` | nenhum | O GitPoulpi substitui os próprios arquivos: **Atualizar e reiniciar** no [painel de atualização](updates.md), sem perguntas. |
| **Para todos os usuários** | `Program Files\GitPoulpi` | uma pergunta na instalação | Cada atualização pede aprovação de administrador (uma pergunta do Windows), e as sessões do GitPoulpi de outros usuários no computador são fechadas antes. |

A mesma escolha funciona em modo silencioso: `gitpoulpi-<version>-...-setup.exe /CURRENTUSER` ou `/ALLUSERS`.

Caixas de seleção: atalho no menu Iniciar, atalho na Área de trabalho, adicionar ao PATH. Uma página pergunta o **alias de comando**: `gp` por padrão, qualquer nome que você quiser, vazio para nenhum (`/ALIAS=nome` ou `/ALIAS=` na linha de comando). O PowerShell já usa `gp` para `Get-ItemProperty`, então escolha outro nome lá ou use `gitpoulpi`.

O instalador copia os executáveis (inclusive o `gitpoulpi-updater`, o pequeno programa que troca arquivos durante uma atualização) e o marcador `.gitpoulpi-dist`. O marcador registra como o GitPoulpi foi instalado (`inno-user` ou `inno-machine`), que é o que o painel de atualização mostra como **Instalado com**. Executar o instalador de novo mantém suas escolhas (escopo, atalhos, alias).

Desinstale em Configurações > Aplicativos do Windows: ele executa `gitpoulpi uninstall` (alias, entradas do PATH, atalhos) e exclui os arquivos, inclusive as sobras `.old` do atualizador. Suas configurações e repositórios ficam.

### Linux e macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

Ele escolhe o arquivo para o seu sistema e a sua CPU, confere o `.sha256`, instala (canal `archive`: o GitPoulpi se atualiza no lugar) em
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

## Com um gerenciador de pacotes

Cada canal abaixo grava o mesmo marcador `.gitpoulpi-dist` ao lado dos programas, com o seu próprio nome de canal. O GitPoulpi o lê para saber quem é dono das atualizações: o gerenciador de pacotes é, então o GitPoulpi nunca troca arquivos pelas costas dele. O painel de atualização (clique na versão na barra de título) então executa o comando do próprio gerenciador de pacotes, ou o mostra para você copiar. Veja [Atualizações](updates.md).

| Canal | Instalar | Observações |
|---|---|---|
| **Chocolatey** (Windows) | `choco install gitpoulpi` | Instala para todos os usuários, então as atualizações pedem aprovação de administrador. |
| **winget** (Windows) | `winget install --id GitPoulpi.GitPoulpi` | Acrescente `--scope user` ou `--scope machine` para escolher; o escopo machine pede aprovação de administrador na instalação e em cada atualização. |
| **Scoop** (Windows) | `scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi` e depois `scoop install gitpoulpi` | Por usuário, sem direitos de administrador. |
| **Homebrew** (macOS, Linux) | `brew install Poulpi-Lab/gitpoulpi/gitpoulpi` | O tap é `Poulpi-Lab/homebrew-gitpoulpi`. |
| **apt** (Debian, Ubuntu) | veja [o repositório apt](#apt-debian-e-ubuntu) | Repositório assinado; um `.deb` também é anexado a toda release. |
| **MSI** (Windows, gerenciado) | `msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn` | Para administradores: todos os usuários, sem atualizações dentro do aplicativo. Veja [Distribuição corporativa](enterprise-distribution.md#msi-com-intune-ou-política-de-grupo). |
| **AUR** (Arch Linux) | `yay -S gitpoulpi-bin` (ou `paru -S gitpoulpi-bin`) | Sem um helper: `git clone https://aur.archlinux.org/gitpoulpi-bin.git`, depois `cd gitpoulpi-bin && makepkg -si`. |
| **cargo** | `cargo binstall gitpoulpi-cli` | Baixa o arquivo de release em vez de compilar. |
| **cargo, a partir do código-fonte** | `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli` | Compila; precisa de uma toolchain Rust. |
| **Docker** | `docker pull poulpilab/gitpoulpi` | Veja [Executando no Docker](docker.md). A mesma imagem está em `ghcr.io/poulpi-lab/gitpoulpi`. |

Os nomes acima são aqueles em que o processo de release publica (veja [Releasing](../../developer/releasing.md)): um canal que ainda não está no ar responde "não encontrado", e o arquivo de release ou o instalador sempre funciona.

Vale o tratamento de PATH do próprio gerenciador de pacotes: Chocolatey, winget, Scoop e Homebrew colocam os programas no PATH sozinhos, e nenhum deles cria o alias `gp` (execute `gitpoulpi alias create gp`, ou Configurações > Geral > Lançador).

### apt (Debian e Ubuntu)

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

O repositório oferece suporte a `amd64` e `arm64` e é assinado com uma chave que pertence ao projeto. Os programas vão para `/usr/lib/gitpoulpi`, com links em `/usr/bin`. O pacote depende do `git` e recomenda o `git-lfs`. Sem o repositório, instale o `.deb` de uma release com `sudo apt install ./gitpoulpi_<version>_<arch>.deb`: o GitPoulpi então diz para você baixar o próximo `.deb` por conta própria, porque só oferece comandos `apt` quando encontra o repositório em `/etc/apt/sources.list.d/gitpoulpi.*`.

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

A barra de título mostra a versão em execução (`v1.0.0`) entre o nome do aplicativo e o caminho do repositório. Clique nela, ou pressione `Ctrl+N` e depois `u`, ou execute **Atualizações: mostrar painel de atualização** na paleta de comandos (`Ctrl+P`): o painel de atualização abre com a versão, como o GitPoulpi foi instalado e o que fazer diante de uma release mais nova. **Ajuda: Notas da versão** abre o changelog.

Uma vez por dia, em segundo plano, o GitPoulpi verifica no lugar onde as suas releases são publicadas (o GitHub, por padrão) se há uma mais nova (`updates.mode`, [Configurações > Sincronização](settings.md#sincronização)). Quando há, o chip passa a dizer `v0.2.0 disponível` na cor de destaque.

O que o painel faz depende de como o GitPoulpi foi instalado: um arquivo de release ou o instalador do Windows substitui os próprios arquivos, um gerenciador de pacotes executa o próprio comando, uma instalação gerenciada ou em contêiner diz como as atualizações chegam a ela. Uma build feita com `cargo build` nunca se atualiza. A descrição completa, com o voltar à versão anterior, `auto` contra `notify` e o que um administrador pode travar, está em [Atualizações](updates.md).

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
