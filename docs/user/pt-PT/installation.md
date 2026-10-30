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
do seu `PATH`. O `INSTALL.txt` dentro do arquivo dá os mesmos passos.

## Com um instalador

Cada versão traz também um instalador por SO. Ambos fazem as mesmas escolhas
(atalhos, PATH, um alias de comando como `gp`) chamando o próprio `gitpoulpi`
instalado, o código por trás de Definições > Geral > Lançador, pelo que o que o
instalador configura e o que as Definições mostram estão sempre de acordo.

### Windows: `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

Instala para o utilizador atual, sem pedir direitos de administrador, em
`%LOCALAPPDATA%\Programs\GitPoulpi` (a primeira caixa de diálogo oferece uma
instalação para todos os utilizadores e a escolha da pasta). Caixas de seleção:
atalho no menu Iniciar, atalho no ambiente de trabalho, adicionar ao PATH. Uma
página pergunta o **alias de comando**: `gp` por predefinição, o nome que
quiser, ou vazio para nenhum. O PowerShell já usa `gp` para `Get-ItemProperty`,
por isso escolha outro nome aí ou use `gitpoulpi`.

O instalador copia os três executáveis e o marcador `.gitpoulpi-dist`, pelo que
[Atualizar agora](#atualizar-no-próprio-sítio) funciona exatamente como no
zip. Uma instalação por utilizador tem permissão de escrita, por isso pode
atualizar-se a si própria; uma instalação para todos os utilizadores em Program
Files precisa de elevação para ser substituída, pelo que deve reinstalá-la com o
novo instalador. Desinstale em Definições do Windows > Aplicações: executa
`gitpoulpi uninstall` (alias, entradas do PATH, atalhos) e elimina os ficheiros,
incluindo os restos `.old` do atualizador. As suas definições e os seus
repositórios ficam.

### Linux e macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

Escolhe o arquivo para o seu SO e CPU, verifica o respetivo `.sha256`, instala
em `~/.local/opt/gitpoulpi` e liga `gitpoulpi` e `gitpoulpi-mcp` em
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

A barra de título mostra a versão em execução (`v1.0.0`) entre o nome da aplicação
e o caminho do repositório. Clique nela, ou execute **Ajuda: notas de versão** a partir da paleta
de comandos (`Ctrl+P`): abre-se um separador Notas de versão com o registo de
alterações, deslocado até à versão em execução.

Uma vez por dia, em segundo plano, o GitPoulpi pergunta ao GitHub se existe uma
versão mais recente (`updates.mode`, [Definições > Sincronização](settings.md#sincronização)).
Quando existe, o chip passa a dizer `v0.2.0 disponível` na cor de destaque e o
clique abre o separador Notas de versão com as notas dessa versão em primeiro
lugar. O separador tem três botões, cada um com uma tecla e um comando da paleta:

- **Transferir** (`d`, *Atualizações: transferir*) abre a página da versão no
  navegador.
- **Ignorar esta versão** (`s`, *Atualizações: ignorar esta versão*) oculta o
  aviso até aparecer uma versão mais recente.
- **Procurar agora** (`r`, *Atualizações: procurar atualizações agora*) pergunta
  de novo de imediato.

`updates.mode = "off"` não faz qualquer pedido. Sem ligação à rede, ou enquanto o
repositório for privado e não houver um token do github.com guardado, a
verificação falha em silêncio (uma linha no registo da aplicação, nada na
interface). `GITPOULPI_UPDATE_API` aponta a verificação para outra raiz de API
(usado pelos testes).

### Atualizar no próprio sítio

Uma compilação instalada a partir de um arquivo de versão pode substituir-se a si
própria. O arquivo traz um pequeno ficheiro `.gitpoulpi-dist` ao lado dos
executáveis; uma compilação em execução só é "atualizável" quando esse ficheiro
está ao lado do seu executável (e indica a sua plataforma). Uma compilação feita
com `cargo`, uma compilação personalizada copiada para outro sítio ou um pacote
que tenha deixado o ficheiro de fora nunca se atualiza sozinha: o seu separador
Notas de versão di-lo e oferece apenas **Transferir**.

Para uma compilação atualizável, o separador acrescenta:

- **Atualizar agora** (`u`, *Atualizações: atualizar agora*) transfere o arquivo
  e o respetivo `.sha256` em segundo plano (uma tarefa no indicador da barra de
  estado, com a sua percentagem, cancelável como qualquer tarefa), verifica o
  hash, descompacta o arquivo e coloca cada ficheiro novo ao lado do antigo como
  `<nome>.new`. Nada do que está em execução é tocado. No modo `auto`, isto
  acontece sozinho assim que uma verificação encontra uma versão mais recente.
- **Reiniciar para atualizar** (`R`, *Atualizações: reiniciar para atualizar*; o
  chip diz o mesmo quando a transferência está preparada) é sempre uma ação sua,
  nunca automática. Pára o daemon autónomo da sessão (um daemon embutido
  desaparece com a janela), troca os ficheiros e inicia a nova versão com os
  mesmos argumentos e a mesma sessão.
- **Reverter para a versão anterior** (`v`, *Atualizações: reverter para a versão anterior*)
  repõe os ficheiros anteriores e reinicia. Os ficheiros da versão anterior ficam
  como `<nome>.old` até a nova versão estar em execução há um minuto, altura em
  que são eliminados, pelo que o botão existe para essa primeira execução. Não faz
  parte do registo de desfazer do git: altera os ficheiros do próprio GitPoulpi,
  não um repositório.

Como os ficheiros são substituídos: no Linux e no macOS, cada ficheiro novo é
renomeado por cima do antigo (atómico; um programa em execução mantém a sua cópia
antiga até terminar). No Windows, um `.exe` em execução não pode ser
sobrescrito, por isso o antigo é renomeado para `<nome>.old` e o novo é movido
para o lugar; um `.old` que ainda esteja em execução é eliminado por um arranque
posterior. A troca é tudo ou nada: se outro programa tiver um dos executáveis
aberto (um cliente MCP a executar `gitpoulpi-mcp`, por exemplo), o GitPoulpi
indica o ficheiro, não altera nada e deixa a transferência preparada para uma
nova tentativa.

O que é recusado, sempre com uma mensagem: uma versão sem ficheiro `.sha256`, um
hash que não corresponde, um arquivo cujo nome não é o desta plataforma
(`gitpoulpi-<version>-<target>.<zip|tar.gz>`), um arquivo cujo marcador indica
outra versão ou plataforma, e qualquer ligação de transferência fora destes
anfitriões: `github.com`, `api.github.com`, `objects.githubusercontent.com`,
`release-assets.githubusercontent.com` e `github-releases.githubusercontent.com`
do GitHub (o primeiro pedido e cada salto de redirecionamento; um redirecionamento
para outro lado interrompe a transferência). Os arquivos de um repositório
privado exigem uma sessão de navegador, por isso a transferência falha aí até o
repositório ser público.

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
