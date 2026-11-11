[English](../installation.md) · [Čeština](../cs/installation.md) · [Deutsch](../de/installation.md) · [Español](../es/installation.md) · [Français](../fr/installation.md) · [Italiano](../it/installation.md) · [日本語](../ja/installation.md) · [한국어](../ko/installation.md) · [Nederlands](../nl/installation.md) · [Polski](../pl/installation.md) · [Português (Brasil)](../pt-BR/installation.md) · **Português (Portugal)** · [Русский](../ru/installation.md) · [Svenska](../sv/installation.md) · [Türkçe](../tr/installation.md) · [Українська](../uk/installation.md) · [简体中文](../zh-CN/installation.md)

# Instalação

O GitPoulpi tem três executáveis:

| Executável | O que é |
|---|---|
| `gitpoulpi-tui` | A interface de terminal. |
| `gitpoulpi` | A interface de linha de comandos (`--json` para scripts). Sem subcomando, abre a interface. |
| `gitpoulpi-mcp` | O servidor MCP para agentes de IA. Veja [MCP e agentes](mcp-and-agents.md). |

Ao lado deles, o arquivo contém `gitpoulpi-launch`, um programa minúsculo que os atalhos executam para mostrar uma janela do GitPoulpi já aberta (`gitpoulpi launch`). No Windows não tem janela de consola, pelo que um clique nunca faz piscar uma. É instalado, atualizado e removido juntamente com os outros.

## O que precisa

- Um **`git` do sistema** no seu `PATH`. O GitPoulpi usa a libgit2 para a maior
  parte das operações e o seu `git` para as restantes.
  É necessária a versão **2.34 ou mais recente**. No macOS, o `git` das Command Line Tools da Apple funciona, tal como o do Homebrew.
- Um **terminal** com suporte de rato. O kitty, o Ghostty, o WezTerm e o iTerm2 (e também Windows Terminal e foot, através de sixel)
  desenham o grafo de commits como imagens. Os outros terminais (a consola
  do Windows, o tmux e a maioria dos restantes) mostram um grafo de carateres. Veja
  [Escolher um terminal](terminals.md).
- Uma **Nerd Font** ou um tipo de letra com emoji, para os ícones da barra de
  ferramentas e da barra lateral.
- Opcional: **`git-lfs`** 3.0 ou posterior, para repositórios Git LFS. Sem ele,
  os ficheiros LFS aparecem como pequenos ficheiros de ponteiro e o GitPoulpi
  diz-lhe porquê. Veja [Git LFS](lfs.md) para saber como o instalar.
- Opcional: um **credential helper** (Git Credential Manager, libsecret, `pass`,
  Keychain), para guardar tokens de alojamento e chaves de API de IA. Veja
  [Integrações de alojamento e de gestores de issues](hosting-integration.md).

## A partir de um arquivo de lançamento

Um arquivo de lançamento é `gitpoulpi-<version>-<target>.zip` (Windows) ou
`.tar.gz` (Linux, macOS), com um ficheiro `.sha256` ao lado. Verifique o arquivo
com o ficheiro `.sha256`, descompacte-o e coloque os três executáveis numa pasta
do seu `PATH` (mantenha o `gitpoulpi-updater` e o marcador `.gitpoulpi-dist` ao lado deles:
o marcador diz `channel=archive`, para que a compilação se possa atualizar
sozinha, veja [Atualizações](updates.md)). O `INSTALL.txt` dentro do arquivo dá os mesmos passos.

### Verificar uma transferência

Todos os ficheiros de uma versão são assinados com a chave [minisign](https://jedisct1.github.io/minisign/) do projeto, e o GitPoulpi verifica essa assinatura ele próprio antes de instalar uma atualização. Para um ficheiro que transferiu à mão, o `.minisig` ao lado dele é a assinatura. A chave pública (id de chave `B461FFCBF6F6A4F2`) está no repositório como `packaging/minisign.pub`:

```
untrusted comment: minisign public key B461FFCBF6F6A4F2
RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

```
minisign -Vm gitpoulpi-<version>-<target>.tar.gz -P RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

Leia o comentário de confiança que o comando mostra: tem de ser `file:<nome do ficheiro> version:<a versão que espera>`. Uma assinatura válida mas que indica outro ficheiro ou outra versão não é uma assinatura para o seu.

Quando a versão foi compilada com atestados de proveniência de compilação (a variável `ATTESTATIONS` do repositório), a CLI do GitHub verifica onde um ficheiro foi compilado:

```
gh attestation verify gitpoulpi-<version>-<target>.tar.gz --repo <owner>/gitpoulpi
```

O ficheiro `.sha256` só deteta uma transferência danificada. A assinatura é o que mostra que o ficheiro é do projeto.

## Com um instalador

Cada versão traz também um instalador por SO. Ambos fazem as mesmas escolhas
(atalhos, PATH, um alias de comando como `gp`) chamando o próprio `gitpoulpi`
instalado, o código por trás de Definições > Lançador, pelo que o que o
instalador configura e o que as Definições mostram estão sempre de acordo.

### Windows: `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

A primeira caixa de diálogo pergunta para quem é a instalação:

| Escolha | Onde | Direitos de administrador | O que significa para as atualizações |
|---|---|---|---|
| **Só para mim** (predefinição) | `%LOCALAPPDATA%\Programs\GitPoulpi` | nenhum | O GitPoulpi substitui os seus próprios ficheiros: **Atualizar e reiniciar** no [painel de atualizações](updates.md), sem pedido de confirmação. |
| **Para todos os utilizadores** | `Program Files\GitPoulpi` | um pedido na instalação | Cada atualização pede aprovação de administrador (um pedido do Windows), e as sessões do GitPoulpi de outros utilizadores no computador são fechadas primeiro. |

A mesma escolha funciona em modo silencioso: `gitpoulpi-<version>-...-setup.exe /CURRENTUSER` ou `/ALLUSERS`.

Caixas de seleção: atalho no menu Iniciar, atalho no ambiente de trabalho, adicionar ao PATH. Uma página pergunta o **alias de comando**: `gp` por predefinição, o nome que quiser, ou vazio para nenhum (`/ALIAS=nome` ou `/ALIAS=` na linha de comandos). O PowerShell já usa `gp` para `Get-ItemProperty`, por isso escolha outro nome aí ou use `gitpoulpi`.

O instalador copia os executáveis (incluindo o `gitpoulpi-updater`, o pequeno programa que troca os ficheiros durante uma atualização) e o marcador `.gitpoulpi-dist`. O marcador regista como o GitPoulpi foi instalado (`inno-user` ou `inno-machine`), que é o que o painel de atualizações mostra como **Instalado com**. Executar o instalador de novo mantém as suas escolhas (âmbito, atalhos, alias).

Desinstale em Definições do Windows > Aplicações: executa `gitpoulpi uninstall` (alias, entradas do PATH, atalhos) e elimina os ficheiros, incluindo os restos `.old` do atualizador. As suas definições e os seus repositórios ficam.

### Linux e macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

Escolhe o arquivo para o seu SO e CPU, verifica o respetivo `.sha256`, instala
(canal `archive`: o GitPoulpi atualiza-se a si próprio no sítio) em `~/.local/opt/gitpoulpi` e liga `gitpoulpi` e `gitpoulpi-mcp` em
`~/.local/bin`. Não precisa de root e nada fora da sua pasta pessoal é tocado.
Sem `--yes`, pergunta pelo alias (predefinição `gp`), pelo PATH e pelos atalhos.

| Opção | Efeito |
|---|---|
| `--prefix DIR` | Instala os ficheiros do programa em `DIR` (absoluto). |
| `--version X.Y.Z` | Instala esta versão em vez da mais recente. |
| `--archive FILE` | Usa um arquivo local; `FILE.sha256` tem de estar ao lado dele (`--no-verify` dispensa-o). Não usa a rede. |
| `--alias NAME` / `--no-alias` | Faz com que `NAME` inicie o GitPoulpi (`gitpoulpi alias create`), ou nenhum. |
| `--no-path` | Não adiciona o GitPoulpi ao PATH das novas shells (caso contrário, `gitpoulpi path add`). |
| `--desktop` | Cria também os atalhos do menu de aplicações e do ambiente de trabalho. |
| `--yes` | Nunca pergunta: aplicam-se as opções e as predefinições (alias `gp`, PATH sim, atalhos não). |
| `--uninstall` | Executa `gitpoulpi uninstall` e depois remove as ligações e a pasta. |

Procurar a versão mais recente exige que o repositório seja público; num
repositório privado, transfira o arquivo num navegador e use `--archive`.

## Com um gestor de pacotes

Cada canal abaixo escreve o mesmo marcador `.gitpoulpi-dist` ao lado dos programas, com o seu próprio nome de canal. O GitPoulpi lê-o para saber quem é dono das atualizações: o gestor de pacotes, pelo que o GitPoulpi nunca troca ficheiros nas suas costas. O painel de atualizações (clique na versão na barra de título) executa então o comando do próprio gestor de pacotes, ou mostra-o para o copiar. Veja [Atualizações](updates.md).

| Canal | Instalar | Notas |
|---|---|---|
| **Chocolatey** (Windows) | `choco install gitpoulpi` | Instala para todos os utilizadores, por isso as atualizações pedem aprovação de administrador. |
| **winget** (Windows) | `winget install --id GitPoulpi.GitPoulpi` | Acrescente `--scope user` ou `--scope machine` para escolher; o âmbito de máquina pede aprovação de administrador na instalação e em cada atualização. |
| **Scoop** (Windows) | `scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi` e depois `scoop install gitpoulpi` | Por utilizador, sem direitos de administrador. |
| **Homebrew** (macOS, Linux) | `brew install Poulpi-Lab/gitpoulpi/gitpoulpi` | O tap é `Poulpi-Lab/homebrew-gitpoulpi`. |
| **apt** (Debian, Ubuntu) | veja [o repositório apt](#apt-debian-e-ubuntu) | Repositório assinado; um `.deb` também vem anexado a cada versão. |
| **MSI** (Windows, gerido) | `msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn` | Para administradores: todos os utilizadores, sem atualizações na aplicação. Veja [Distribuição empresarial](enterprise-distribution.md#msi-com-intune-ou-política-de-grupo). |
| **AUR** (Arch Linux) | `yay -S gitpoulpi-bin` (ou `paru -S gitpoulpi-bin`) | Sem um auxiliar: `git clone https://aur.archlinux.org/gitpoulpi-bin.git`, depois `cd gitpoulpi-bin && makepkg -si`. |
| **cargo** | `cargo binstall gitpoulpi-cli` | Transfere o arquivo da versão em vez de compilar. |
| **cargo, a partir do código-fonte** | `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli` | Compila; precisa de uma toolchain Rust. |
| **Docker** | `docker pull poulpilab/gitpoulpi` | Veja [Executar no Docker](docker.md). A mesma imagem está em `ghcr.io/poulpi-lab/gitpoulpi`. |

Os nomes acima são aqueles para onde o processo de lançamento publica (veja [Lançamentos](../../developer/releasing.md)): um canal que ainda não está ativo responde "não encontrado", e o arquivo da versão ou o instalador funcionam sempre.

Aplica-se o tratamento do PATH do próprio gestor de pacotes: o Chocolatey, o winget, o Scoop e o Homebrew colocam eles próprios os programas no PATH, e nenhum deles cria o alias `gp` (execute `gitpoulpi alias create gp`, ou Definições > Lançador).

### apt (Debian e Ubuntu)

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

O repositório suporta `amd64` e `arm64` e é assinado com uma chave que pertence ao projeto. Os programas vão para `/usr/lib/gitpoulpi`, com ligações em `/usr/bin`. O pacote depende de `git` e recomenda `git-lfs`. Sem o repositório, instale o `.deb` de uma versão com `sudo apt install ./gitpoulpi_<version>_<arch>.deb`: o GitPoulpi diz-lhe então que transfira você próprio o `.deb` seguinte, porque só oferece comandos `apt` quando encontra o repositório em `/etc/apt/sources.list.d/gitpoulpi.*`.

## A partir do código-fonte

Precisa de uma toolchain Rust estável recente (<https://rustup.rs>).

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

Os executáveis ficam em `target/release/`. O perfil `release` compila
depressa. Para os executáveis mais pequenos e mais rápidos, use o perfil
`dist`. Demora mais a compilar:

```
cargo build --profile dist
```

Os scripts `build-opti` na raiz do repositório executam esta compilação para os
três executáveis e mostram os seus tamanhos:

```
./build-opti.sh                  # Linux, macOS
build-opti                       # Windows (cmd ou PowerShell)
```

Para compilar apenas os idiomas e as funcionalidades que usa, veja
[Compilações personalizadas](custom-builds.md).

## Criar o seu próprio arquivo de lançamento

```
cargo run -p xtask -- dist
```

Isto compila o perfil `dist` e escreve
`target/dist/gitpoulpi-<version>-<target>.zip` (Windows) ou `.tar.gz` (outros
sistemas). O arquivo contém os três executáveis, `INSTALL.txt`, `README.md`,
`LICENSE` e `CHANGELOG.md`. Um ficheiro `.sha256` é escrito ao lado.

## Atualizações

A barra de título mostra a versão em execução (`v1.0.0`) entre o nome da aplicação e o caminho do repositório. Quando se conhece uma versão mais recente (ou há uma atualização em curso), um clique nela abre o painel de atualizações com a versão, como o GitPoulpi foi instalado e o que fazer perante a nova versão; caso contrário, um clique abre o separador Notas de versão. `Ctrl+N` e depois `u`, ou **Atualizações: mostrar atualização** na paleta de comandos (`Ctrl+P`), abrem sempre o painel de atualizações, e **Ajuda: notas de versão** abre sempre o registo de alterações.

Uma vez por dia, em segundo plano, o GitPoulpi verifica no sítio onde as suas versões são publicadas (por predefinição o GitHub) se existe uma mais recente (`updates.mode`, [Definições > Sincronização](settings.md#sincronização)). Quando existe, o indicador passa a dizer `v0.2.0 disponível` na cor de destaque.

O que o painel faz depende de como o GitPoulpi foi instalado: um arquivo de lançamento ou o instalador do Windows substitui os seus próprios ficheiros, um gestor de pacotes executa o seu próprio comando, uma instalação gerida ou de contentor diz como as atualizações lhe chegam. Uma compilação feita com `cargo build` nunca se atualiza. A descrição completa, com reverter, `auto` contra `notify` e o que um administrador pode bloquear, está em [Atualizações](updates.md).

## Verificar que funciona

```
gitpoulpi --version
cd path/to/a/git/repository
gitpoulpi-tui
```

`--version` mostra a versão e o commit a partir do qual a compilação foi feita.
Depois siga os [Primeiros passos](getting-started.md).

## Desinstalar

`gitpoulpi uninstall` (o que ambos os instaladores executam) remove os alias de comando,
as entradas do PATH e os atalhos que o GitPoulpi criou e, numa instalação a partir de uma versão,
os ficheiros do programa; recusa-se a correr numa compilação sem o marcador `.gitpoulpi-dist`
(uma compilação com cargo), e `--keep-files` deixa os ficheiros do programa. Não está
disponível para os agentes MCP. Sem instalador, apague os três executáveis.
O GitPoulpi também guarda estes ficheiros:

| O quê | Onde |
|---|---|
| Definições (`config.toml`) e sessões | A pasta de configuração do seu SO. `GITPOULPI_CONFIG_DIR` altera-a. |
| Estado por repositório | `.gitpoulpi/` dentro de cada repositório. |
| Caches (avatares) | A pasta de cache do seu SO. |
| Tokens e chaves de API | O seu credential helper. **Definições > SSH e credenciais > Esquecer todos os utilizadores e palavras-passe** remove-os. |

Veja a [Referência de configuração](config-reference.md) para os caminhos exatos.
