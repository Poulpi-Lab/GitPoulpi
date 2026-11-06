[English](../enterprise-distribution.md) · [Čeština](../cs/enterprise-distribution.md) · [Deutsch](../de/enterprise-distribution.md) · [Español](../es/enterprise-distribution.md) · [Français](../fr/enterprise-distribution.md) · [Italiano](../it/enterprise-distribution.md) · [日本語](../ja/enterprise-distribution.md) · [한국어](../ko/enterprise-distribution.md) · [Nederlands](../nl/enterprise-distribution.md) · [Polski](../pl/enterprise-distribution.md) · [Português (Brasil)](../pt-BR/enterprise-distribution.md) · **Português (Portugal)** · [Русский](../ru/enterprise-distribution.md) · [Svenska](../sv/enterprise-distribution.md) · [Türkçe](../tr/enterprise-distribution.md) · [Українська](../uk/enterprise-distribution.md) · [简体中文](../zh-CN/enterprise-distribution.md)

# Distribuição empresarial

Para administradores que implementam o GitPoulpi em muitos computadores a partir dos seus próprios servidores: feeds de pacotes internos, espelhos, uma política de atualização válida para toda a máquina e uma configuração predefinida. Leia primeiro [Instalação](installation.md) e [Atualizações](updates.md).

Dois factos comandam tudo o que se segue:

- Cada instalação regista **como foi instalada** no marcador `.gitpoulpi-dist`, ao lado dos programas (`channel=...`). O GitPoulpi só se atualiza da forma que esse canal permite: nunca substitui ficheiros de que um gestor de pacotes é dono, e um MSI nunca é tocado.
- A **política** (um ficheiro e, no Windows, o registo) é sua. O `config.toml` de um utilizador nunca a pode atenuar.

## Chocolatey, com um feed privado

O pacote público transfere o instalador do GitHub. Para uma rede fechada, internalize-o:

1. Obtenha o pacote (`choco download gitpoulpi --internalize` com as ferramentas Chocolatey licenciadas, ou transfira o `.nupkg` e descompacte-o).
2. Em `tools\chocolateyinstall.ps1`, aponte `url64bit` para a sua própria cópia de `gitpoulpi-<versão>-x86_64-pc-windows-msvc-setup.exe` e mantenha `checksum64` igual ao primeiro campo do ficheiro `.sha256` da versão (ou à soma de verificação da sua cópia).
3. `choco pack` e depois envie o `.nupkg` para o seu feed (Nexus, Artifactory, ProGet, uma partilha de ficheiros).

Instale-o com parâmetros do pacote:

```
choco install gitpoulpi --source=https://nexus.example/repository/choco-internal/ ^
  --params "/Channel:choco-internal /Feed:https://nexus.example/repository/choco-internal/"
```

| Parâmetro | Efeito |
|---|---|
| `/Channel:choco-internal` | Regista `channel=choco-internal` no marcador, para que o GitPoulpi saiba que o feed é seu e não o feed da comunidade Chocolatey. |
| `/Feed:<url>` | Regista o endereço do feed (`feed=` no marcador). O GitPoulpi verifica-o à procura de novas versões e executa `choco upgrade gitpoulpi -y --source=<feed>`. |

Sem parâmetros, o canal é `choco` e a verificação vai para o feed da comunidade Chocolatey. O pacote instala-se para todos os utilizadores em Program Files, sem alias `gp`.

A atualização do GitPoulpi executa `choco upgrade` sem `--params`, para que nada do que implementou seja tocado. O Chocolatey tem, por isso, de memorizar os parâmetros do pacote da primeira instalação (`useRememberedArgumentsForUpgrades`, ativo por predefinição desde o Chocolatey 2.0; em versões mais antigas `choco feature enable -n useRememberedArgumentsForUpgrades`). Caso contrário, a atualização registaria o canal simples `choco`. Um `check_url` da política (abaixo) mantém a verificação e a atualização apontadas ao seu feed, diga o que disser o marcador, e tem precedência sobre o feed do marcador.

**Nada no pacote escreve `policy.toml` nem `config.toml`.** O que implementou mantém-se, na instalação e em cada atualização. Veja [Implementar um config.toml predefinido](#implementar-um-configtoml-predefinido).

## winget, com uma origem interna

O winget atualiza o GitPoulpi com `winget upgrade --id GitPoulpi.GitPoulpi --exact`, usando a origem de onde o pacote veio. Para uma origem interna, aloje uma origem REST do Windows Package Manager (qualquer implementação da API REST do winget), publique os três manifestos da versão (`GitPoulpi.GitPoulpi.yaml`, `...installer.yaml`, `...locale.en-US.yaml`) com `InstallerUrl` a apontar para a sua cópia do programa de instalação e o `InstallerSha256` correspondente, e depois:

```
winget source add --name corp --arg https://winget.example/api --type Microsoft.Rest
winget install --id GitPoulpi.GitPoulpi --source corp
```

Mantenha as opções do instalador do manifesto: `/CHANNEL=winget` faz o marcador dizer `winget`, e `/ALIAS=` deixa de fora o alias `gp`. O winget não diz ao GitPoulpi o que há de novo, pelo que a verificação continua a perguntar ao GitHub pela versão. Numa rede sem GitHub, aponte-a para o seu próprio `latest.json` com uma política: `check = "manifest"` (veja [latest.json](#latestjson-e-download_base-um-espelho-da-versão)).

## Scoop, com um bucket privado

Ponha `gitpoulpi.json` (o manifesto do bucket público) em `bucket/` de um repositório Git que aloje, altere `url` para a sua cópia do zip da versão e defina `hash` com o seu sha256 (`autoupdate` pode ficar se a sua cópia seguir os nomes da versão). Depois:

```
scoop bucket add corp https://git.example/scoop-corp.git
scoop install corp/gitpoulpi
```

O `post_install` do manifesto reescreve o marcador com `channel=scoop`; o GitPoulpi executa então ele próprio `scoop update gitpoulpi` e nunca troca ficheiros nas costas do Scoop. O Scoop instala por utilizador e não precisa de direitos de administrador.

## apt, com um espelho

O repositório é uma árvore de pastas simples (`dists/`, `pool/`, o `gitpoulpi-archive-keyring.gpg` e o `.asc` da chave de assinatura, e o exemplo `gitpoulpi.sources`) servida por HTTPS. Espelhe-o com `wget --mirror`, `rsync` ou o seu gestor de artefactos, sirva-o internamente e instale a chave e a origem em cada computador:

```
sudo install -d /etc/apt/keyrings
sudo install -m 644 gitpoulpi-archive-keyring.gpg /etc/apt/keyrings/gitpoulpi.gpg
```

`/etc/apt/sources.list.d/gitpoulpi.sources`:

```
Types: deb
URIs: https://mirror.example/gitpoulpi/apt
Suites: stable
Components: main
Architectures: amd64 arm64
Signed-By: /etc/apt/keyrings/gitpoulpi.gpg
```

Mantenha o nome de ficheiro `gitpoulpi.sources` (ou `gitpoulpi.list`): o GitPoulpi só mostra comandos `apt` quando encontra `/etc/apt/sources.list.d/gitpoulpi.*`. Use a chave do projeto: o espelho continua assinado por ela, e `Signed-By` fixa essa única chave só para este repositório. O repositório guarda as últimas cinco versões de cada pacote.

**`.deb` offline:** cada versão anexa `gitpoulpi_<versão>_<arq>.deb`. Copie-o e execute `sudo apt install ./gitpoulpi_<versão>_<arq>.deb`. O marcador diz `channel=apt`; sem o ficheiro de origem, o GitPoulpi diz ao utilizador que transfira o próximo `.deb` em vez de mostrar um comando `apt`.

## MSI, com Intune ou Política de Grupo

`gitpoulpi-<versão>-x86_64-pc-windows-msvc.msi` instala-se para todos os utilizadores e destina-se a ser gerido: o painel do GitPoulpi apenas diz "gerido pelo seu administrador" (com a sua `message`), nunca atualiza uma instalação MSI. Atualize implementando o novo MSI; é uma atualização principal, pelo que a versão antiga é substituída.

```
msiexec /i gitpoulpi-0.2.0-x86_64-pc-windows-msvc.msi /qn ^
  ADDTOPATH=1 UPDATES_MODE=notify UPDATES_PIN=0.2 ^
  UPDATES_MESSAGE="As atualizações são implementadas pelos serviços de TI."
```

| Propriedade | Efeito |
|---|---|
| `INSTALLFOLDER` | Pasta de instalação (predefinição `Program Files\GitPoulpi`). |
| `ADDTOPATH` | `1` (predefinição) acrescenta a pasta ao PATH da máquina, `0` não. |
| `UPDATES_MODE` | `mode` da política: `off`, `notify` ou `auto`. |
| `UPDATES_PIN` | `pin` da política, por exemplo `0.2` ou `0.2.4`. |
| `UPDATES_CHECK` | `check` da política: `github`, `nuget`, `manifest` ou `none`. |
| `UPDATES_CHECK_URL` | `check_url` da política. |
| `UPDATES_MESSAGE` | `message` da política, mostrada no painel de atualizações. |

Uma propriedade só escreve o seu valor em `HKLM\SOFTWARE\Policies\GitPoulpi\Updates` quando a define, pelo que uma não definida deixa o registo (e um ficheiro de política) em paz. **Intune:** implemente o MSI como aplicação de linha de negócio ou Win32 com o comando acima. **Política de Grupo:** a instalação de software não consegue passar propriedades, por isso use uma transformação (`.mst`) ou defina os valores do registo com as Preferências da Política de Grupo, veja abaixo.

## Homebrew, com um tap interno

Copie `gitpoulpi.rb` do tap público para `Formula/` de um repositório Git que aloje, altere as linhas `url` para as suas cópias dos arquivos da versão e atualize cada `sha256` (o primeiro campo do ficheiro `.sha256` junto a cada arquivo). Depois:

```
brew tap corp/gitpoulpi https://git.example/homebrew-gitpoulpi.git
brew install corp/gitpoulpi/gitpoulpi
```

A fórmula instala o marcador com `channel=homebrew`: o GitPoulpi executa ele próprio `brew upgrade gitpoulpi`, que lê o seu tap.

## Espelho de registo Docker

O nome da imagem é `poulpilab/gitpoulpi` (e `ghcr.io/poulpi-lab/gitpoulpi`). O painel mostra sempre esse nome, pelo que um espelho tem de ser transparente.

- **Cache pull-through:** execute um registo como proxy do Docker Hub (a imagem `registry` com `proxy.remoteurl: https://registry-1.docker.io`) e defina `"registry-mirrors": ["https://registry.example"]` no `daemon.json` do daemon do Docker. `docker pull poulpilab/gitpoulpi` passa então pela sua cache.
- **Cópia:** `docker pull poulpilab/gitpoulpi:<v>`, `docker tag poulpilab/gitpoulpi:<v> registry.example/tools/gitpoulpi:<v>`, `docker push registry.example/tools/gitpoulpi:<v>` e depois execute esse nome. O painel continua a mostrar o público, o que serve bem como indicação da versão.

Veja [Executar no Docker](docker.md) para saber como o executar.

## O ficheiro de política e o registo

A política define como as atualizações se comportam num computador e não pode ser alterada pelos utilizadores. É lida de:

| SO | Ficheiro de política |
|---|---|
| Windows | `%ProgramData%\GitPoulpi\policy.toml` |
| Linux | `/etc/gitpoulpi/policy.toml` |
| macOS | `/Library/Application Support/GitPoulpi/policy.toml` |
| Windows, também | a chave de registo `HKLM\SOFTWARE\Policies\GitPoulpi\Updates`, um valor `REG_SZ` por cada chave abaixo, com os mesmos nomes (para Política de Grupo, Intune e propriedades MSI) |

O ficheiro tem uma tabela:

```toml
[updates]
mode = "notify"
release_channel = "stable"
pin = "1.4"
check = "manifest"
check_url = "https://mirror.example/gitpoulpi/latest.json"
repo = "Poulpi-Lab/GitPoulpi"
download_base = "https://mirror.example/gitpoulpi/"
allow_self_update = false
message = "As atualizações são implementadas pelos serviços de TI."
```

| Chave | Valores | Efeito |
|---|---|---|
| `mode` | `off`, `notify`, `auto` | Um teto: `off` < `notify` < `auto`. Os utilizadores podem escolher um valor inferior, nunca superior. `off` esconde a verificação e não faz nenhum pedido. |
| `release_channel` | `stable`, `prerelease` | Impõe o canal de atualização. |
| `pin` | `1.4` ou `1.4.2` | A versão mais alta oferecida: `1.4` permite qualquer `1.4.x`, `1.4.2` permite no máximo `1.4.2`. |
| `check` | `github`, `nuget`, `manifest`, `none` | Onde se pergunta "qual é a última versão". Predefinição por instalação: o GitHub, o feed do Chocolatey para os canais Chocolatey. `none` nunca verifica. |
| `check_url` | URL | Com `nuget`: o feed. Com `manifest`: o endereço de `latest.json`. Com `github`: a base da API do GitHub, para o GitHub Enterprise Server (acrescente `repo`). |
| `repo` | `owner/name` | O repositório verificado com `check = "github"`. |
| `download_base` | URL | Um espelho dos ficheiros da versão (mesmos nomes de ficheiro) para os canais de arquivo e de instalador. As transferências deste anfitrião são permitidas além das do GitHub. |
| `allow_self_update` | `true`, `false` | `false` transforma cada ação "substituir os ficheiros do GitPoulpi" e "executar o gestor de pacotes" numa nota ou num comando para copiar. Predefinição `true`. |
| `message` | texto | Mostrada no painel de atualizações ("Do seu administrador: ..."). |

**Precedência**, da mais forte para a mais fraca: o registo, o ficheiro de política, o `[updates]` do `config.toml` do utilizador, o `feed=` do marcador, as predefinições incorporadas. Uma chave que a política define fica **Definido pelo seu administrador** nas Definições e não pode ser editada aí.

**Falha segura:** um ficheiro de política ou valor de registo que não possa ser lido (um erro de sintaxe TOML, `mode = "sometimes"`, um `pin` que não é uma versão) é tratado como `mode = notify` e `allow_self_update = false`, e uma linha vai para o registo da aplicação. Um erro de escrita nunca deixa uma frota a atualizar-se sozinha. Uma chave desconhecida é ignorada com um aviso. No Windows, o passo que detém direitos de administrador lê apenas a política da máquina, nunca a configuração do utilizador.

### Exemplos

Desligar as atualizações (uma imagem bloqueada, uma rede isolada):

```toml
[updates]
mode = "off"
```

Fixar a frota na linha 1.4, só notificar, e dizer a quem perguntar:

```toml
[updates]
mode = "notify"
pin = "1.4"
allow_self_update = false
message = "As atualizações passam pelo centro de software. Pergunte ao service desk."
```

Verificar um feed Chocolatey (NuGet) interno:

```toml
[updates]
check = "nuget"
check_url = "https://nexus.example/repository/choco-internal/"
```

O mesmo tipo de política no Windows através do registo (PowerShell):

```
New-Item -Path HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Force
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name mode -Value notify
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name pin -Value 1.4
```

### latest.json e download_base: um espelho da versão

Copie os ficheiros de uma versão (os arquivos, o instalador, os ficheiros `.sha256`, `latest.json`, **todos os ficheiros de assinatura `.minisig`** e `security.json`) para uma pasta de um servidor web interno, mantendo os nomes. `latest.json` vem anexado a cada versão:

```json
{
  "version": "1.4.2",
  "notes": "Notas de versão em Markdown",
  "assets": { "gitpoulpi-1.4.2-x86_64-pc-windows-msvc.zip": "<sha256>" }
}
```

Então a política é:

```toml
[updates]
check = "manifest"
check_url = "https://mirror.example/gitpoulpi/latest.json"
download_base = "https://mirror.example/gitpoulpi/"
```

Os canais de arquivo e de instalador verificam o manifesto e transferem de `download_base`, com a mesma verificação que a do GitHub. Um espelho só tem de copiar ficheiros; não precisa de nenhuma API.

As assinaturas viajam com os ficheiros, pelo que um espelho não as pode enfraquecer:

- `latest.json` só é aceite quando `latest.json.minisig` está ao seu lado e é validado com a chave do projeto incorporada no GitPoulpi. Um manifesto sem a sua assinatura, ou editado depois de assinado, é recusado (os hashes que contém são aquilo com que as transferências são verificadas).
- Cada arquivo é transferido juntamente com `<arquivo>.minisig` do mesmo anfitrião e só é instalado quando essa assinatura é validada e indica este ficheiro e esta versão. Um espelho que copiou o arquivo mas não o seu `.minisig` faz a atualização falhar com "sem assinatura", e nada é alterado.
- `security.json` (os avisos de segurança, veja [Atualizações](updates.md#avisos-de-segurança)) e `security.json.minisig` são procurados ao lado do manifesto. Um espelho que não os copie simplesmente não mostra avisos.

Copie os ficheiros sem alterações: uma assinatura cobre os bytes exatos. Os avisos de segurança são republicados para a última versão sem uma nova compilação, por isso atualize `latest.json`, `security.json` e os respetivos ficheiros `.minisig` a partir da página da versão sempre que atualizar o espelho.

## Implementar um config.toml predefinido

O GitPoulpi não tem um `config.toml` para toda a máquina: as definições ficam na pasta de cada utilizador (`%APPDATA%\GitPoulpi\config.toml`, `~/.config/gitpoulpi/config.toml`, `~/Library/Application Support/GitPoulpi/config.toml`). Para dar as suas predefinições a novos utilizadores, copie um `config.toml` esparso (só as chaves que quer, veja a [Referência de configuração](config-reference.md)) para essa pasta quando o utilizador não tiver nenhum: um script de início de sessão ou do Intune no Windows, `/etc/skel/.config/gitpoulpi/` no Linux, um script de primeiro início de sessão no macOS. Nada nos pacotes do GitPoulpi substitui alguma vez uma configuração, por isso é sua para gerir. Não ponha aí regras de atualização quando elas tiverem de se manter: use a política, que um utilizador não pode contornar. Uma configuração portátil ou por tarefa também pode ser escolhida com `GITPOULPI_CONFIG_DIR`.
