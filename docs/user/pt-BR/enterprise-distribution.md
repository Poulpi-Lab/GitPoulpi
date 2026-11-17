[English](../enterprise-distribution.md) · [Čeština](../cs/enterprise-distribution.md) · [Deutsch](../de/enterprise-distribution.md) · [Español](../es/enterprise-distribution.md) · [Français](../fr/enterprise-distribution.md) · [Italiano](../it/enterprise-distribution.md) · [日本語](../ja/enterprise-distribution.md) · [한국어](../ko/enterprise-distribution.md) · [Nederlands](../nl/enterprise-distribution.md) · [Polski](../pl/enterprise-distribution.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/enterprise-distribution.md) · [Русский](../ru/enterprise-distribution.md) · [Svenska](../sv/enterprise-distribution.md) · [Türkçe](../tr/enterprise-distribution.md) · [Українська](../uk/enterprise-distribution.md) · [简体中文](../zh-CN/enterprise-distribution.md)

# Distribuição corporativa

Para administradores que implantam o GitPoulpi em muitos computadores a partir de servidores próprios: feeds de pacotes internos, espelhos, uma política de atualização válida para a máquina inteira e uma configuração padrão. Leia antes [Instalação](installation.md) e [Atualizações](updates.md).

Dois fatos movem tudo aqui:

- Toda instalação registra **como foi instalada** no marcador `.gitpoulpi-dist`, ao lado dos programas (`channel=...`). O GitPoulpi se atualiza apenas da forma que esse canal permite: nunca substitui arquivos que pertencem a um gerenciador de pacotes, e um MSI nunca é tocado.
- A **política** (um arquivo e, no Windows, o registro) é sua. O `config.toml` do próprio usuário nunca consegue afrouxá-la.

## Chocolatey, com um feed privado

O pacote público baixa o instalador do GitHub. Para uma rede fechada, internalize-o:

1. Obtenha o pacote (`choco download gitpoulpi --internalize` com as ferramentas licenciadas do Chocolatey, ou baixe o `.nupkg` e descompacte-o).
2. Em `tools\chocolateyinstall.ps1`, aponte `url64bit` para a sua própria cópia de `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe` e mantenha `checksum64` igual ao primeiro campo do arquivo `.sha256` da release (ou ao checksum da sua cópia).
3. `choco pack` e depois envie o `.nupkg` ao seu feed (Nexus, Artifactory, ProGet, um compartilhamento de arquivos).

Instale-o com parâmetros de pacote:

```
choco install gitpoulpi --source=https://nexus.example/repository/choco-internal/ ^
  --params "/Channel:choco-internal /Feed:https://nexus.example/repository/choco-internal/"
```

| Parâmetro | Efeito |
|---|---|
| `/Channel:choco-internal` | Grava `channel=choco-internal` no marcador, para que o GitPoulpi saiba que o feed é seu e não o feed da comunidade do Chocolatey. |
| `/Feed:<url>` | Grava o endereço do feed (`feed=` no marcador). O GitPoulpi o consulta em busca de novas versões e executa `choco upgrade gitpoulpi -y --source=<feed>`. |

Sem parâmetros, o canal é `choco` e a verificação vai ao feed da comunidade do Chocolatey. O pacote instala para todos os usuários em Program Files, sem o alias `gp`.

A atualização do GitPoulpi executa `choco upgrade` sem `--params`, para que nada do que você implantou seja tocado. O Chocolatey precisa então lembrar os parâmetros de pacote da primeira instalação (`useRememberedArgumentsForUpgrades`, ativado por padrão desde o Chocolatey 2.0; em versões mais antigas, `choco feature enable -n useRememberedArgumentsForUpgrades`). Caso contrário, a atualização gravaria o canal simples `choco`. Um `check_url` de política (abaixo) mantém a verificação e a atualização apontadas para o seu feed seja qual for o marcador, e ele tem precedência sobre o feed do marcador.

**Nada no pacote grava `policy.toml` nem `config.toml`.** O que você implantou permanece, na instalação e em toda atualização. Veja [Implantando um config.toml padrão](#implantando-um-configtoml-padrão).

## Scoop, com um bucket privado

Coloque o `gitpoulpi.json` (o manifesto do bucket público) em `bucket/` de um repositório Git que você hospeda, mude `url` para a sua cópia do zip da release e defina `hash` como o sha256 dele (o `autoupdate` pode ficar se a sua cópia seguir os nomes da release). Depois:

```
scoop bucket add corp https://git.example/scoop-corp.git
scoop install corp/gitpoulpi
```

O `post_install` do manifesto reescreve o marcador com `channel=scoop`; o GitPoulpi então executa `scoop update gitpoulpi` por conta própria e nunca troca arquivos por trás do Scoop. O Scoop instala por usuário e não precisa de direitos de administrador.

## apt, com um espelho

O repositório é uma árvore de diretórios simples (`dists/`, `pool/`, o `gitpoulpi-archive-keyring.gpg` e o `.asc` da chave de assinatura, e o exemplo `gitpoulpi.sources`) servida por HTTPS. Espelhe-o com `wget --mirror`, `rsync` ou o seu gerenciador de artefatos, sirva-o internamente e instale a chave e a fonte em cada computador:

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

Mantenha o nome de arquivo `gitpoulpi.sources` (ou `gitpoulpi.list`): o GitPoulpi só mostra comandos `apt` quando encontra `/etc/apt/sources.list.d/gitpoulpi.*`. Use a chave do projeto: o espelho continua assinado por ela, e `Signed-By` fixa essa única chave só para este repositório. O repositório guarda as últimas cinco versões de cada pacote.

**`.deb` offline:** toda release anexa `gitpoulpi_<version>_<arch>.deb`. Copie-o e execute `sudo apt install ./gitpoulpi_<version>_<arch>.deb`. O marcador diz `channel=apt`; sem o arquivo de fonte, o GitPoulpi diz ao usuário para baixar o próximo `.deb` em vez de mostrar um comando `apt`.

## MSI, com Intune ou Política de Grupo

O `gitpoulpi-<version>-x86_64-pc-windows-msvc.msi` instala para todos os usuários e foi feito para ser gerenciado: o painel do GitPoulpi só diz "gerenciada pelo seu administrador" (com a sua `message`) e nunca atualiza uma instalação MSI. Atualize implantando o novo MSI; é uma atualização principal (major upgrade), então a versão antiga é substituída.

```
msiexec /i gitpoulpi-0.2.0-x86_64-pc-windows-msvc.msi /qn ^
  ADDTOPATH=1 UPDATES_MODE=notify UPDATES_PIN=0.2 ^
  UPDATES_MESSAGE="Updates are deployed by IT."
```

| Propriedade | Efeito |
|---|---|
| `INSTALLFOLDER` | Pasta de instalação (padrão `Program Files\GitPoulpi`). |
| `ADDTOPATH` | `1` (padrão) adiciona a pasta ao PATH da máquina, `0` não adiciona. |
| `UPDATES_MODE` | O `mode` da política: `off`, `notify` ou `auto`. |
| `UPDATES_PIN` | O `pin` da política, por exemplo `0.2` ou `0.2.4`. |
| `UPDATES_CHECK` | O `check` da política: `github`, `nuget`, `manifest` ou `none`. |
| `UPDATES_CHECK_URL` | O `check_url` da política. |
| `UPDATES_MESSAGE` | O `message` da política, mostrado no painel de atualização. |

Uma propriedade grava o valor em `HKLM\SOFTWARE\Policies\GitPoulpi\Updates` só quando você a define, então uma propriedade não definida deixa o registro (e um arquivo de política) em paz. **Intune:** implante o MSI como um aplicativo de linha de negócios ou Win32 com o comando acima. **Política de Grupo:** a instalação de software não consegue passar propriedades, então use uma transformação (`.mst`) ou defina os valores do registro com as Preferências da Política de Grupo, veja abaixo.

## Homebrew, com um tap interno

Copie o `gitpoulpi.rb` do tap público para `Formula/` de um repositório Git que você hospeda, mude as linhas `url` para as suas cópias dos arquivos de release e atualize cada `sha256` (o primeiro campo do arquivo `.sha256` ao lado de cada arquivo). Depois:

```
brew tap corp/gitpoulpi https://git.example/homebrew-gitpoulpi.git
brew install corp/gitpoulpi/gitpoulpi
```

A fórmula instala o marcador com `channel=homebrew`: o GitPoulpi executa `brew upgrade gitpoulpi` por conta própria, que lê o seu tap.

## Espelho de registro Docker

O nome da imagem é `poulpilab/gitpoulpi`. O painel sempre mostra esse nome, então um espelho precisa ser transparente.

- **Cache pull-through:** execute um registro como proxy do Docker Hub (a imagem `registry` com `proxy.remoteurl: https://registry-1.docker.io`) e defina `"registry-mirrors": ["https://registry.example"]` no `daemon.json` do daemon do Docker. `docker pull poulpilab/gitpoulpi` passa então pelo seu cache.
- **Cópia:** `docker pull poulpilab/gitpoulpi:<v>`, `docker tag poulpilab/gitpoulpi:<v> registry.example/tools/gitpoulpi:<v>`, `docker push registry.example/tools/gitpoulpi:<v>` e depois execute esse nome. O painel continua mostrando o público, o que não tem problema como indicação da versão.

Veja [Executando no Docker](docker.md) para saber como executá-la.

## O arquivo de política e o registro

A política define como as atualizações se comportam em um computador e não pode ser alterada pelos usuários. Ela é lida de:

| SO | Arquivo de política |
|---|---|
| Windows | `%ProgramData%\GitPoulpi\policy.toml` |
| Linux | `/etc/gitpoulpi/policy.toml` |
| macOS | `/Library/Application Support/GitPoulpi/policy.toml` |
| Windows, também | a chave do registro `HKLM\SOFTWARE\Policies\GitPoulpi\Updates`, um valor `REG_SZ` por chave abaixo, com os mesmos nomes (para Política de Grupo, Intune e propriedades do MSI) |

O arquivo tem uma tabela:

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
message = "Updates are deployed by IT."
```

| Chave | Valores | Efeito |
|---|---|---|
| `mode` | `off`, `notify`, `auto` | Um teto: `off` < `notify` < `auto`. Os usuários podem escolher um valor menor, nunca um maior. `off` esconde a verificação e não faz nenhuma requisição. |
| `release_channel` | `stable`, `prerelease` | Força o canal de atualização. |
| `pin` | `1.4` ou `1.4.2` | A versão mais alta oferecida: `1.4` permite qualquer `1.4.x`, `1.4.2` permite no máximo `1.4.2`. |
| `check` | `github`, `nuget`, `manifest`, `none` | Onde se pergunta "qual é a release mais recente". Padrão por instalação: o GitHub, o feed do Chocolatey para os canais do Chocolatey. `none` nunca verifica. |
| `check_url` | URL | Com `nuget`: o feed. Com `manifest`: o endereço do `latest.json`. Com `github`: a base da API do GitHub, para o GitHub Enterprise Server (acrescente `repo`). |
| `repo` | `owner/name` | O repositório verificado com `check = "github"`. |
| `download_base` | URL | Um espelho dos arquivos da release (mesmos nomes de arquivo) para os canais de arquivo e de instalador. Downloads desse host são permitidos além dos do GitHub. |
| `allow_self_update` | `true`, `false` | `false` transforma toda ação de "substituir os arquivos do GitPoulpi" e de "executar o gerenciador de pacotes" em uma nota ou em um comando para copiar. Padrão `true`. |
| `message` | texto | Mostrada no painel de atualização ("Do seu administrador: ..."). |

**Precedência**, da mais forte para a mais fraca: o registro, o arquivo de política, o `[updates]` do `config.toml` do usuário, o `feed=` do marcador, os padrões embutidos. Uma chave que a política define fica **Definido pelo seu administrador** nas Configurações e não pode ser editada ali.

**Falha fechada:** um arquivo de política ou valor do registro que não pode ser lido (um erro de sintaxe TOML, `mode = "sometimes"`, um `pin` que não é uma versão) é tratado como `mode = notify` e `allow_self_update = false`, e uma linha vai para o registro do aplicativo. Um erro de digitação nunca deixa uma frota se atualizando sozinha. Uma chave desconhecida é ignorada com um aviso. No Windows, a etapa que tem direitos de administrador lê só a política da máquina, nunca a configuração do usuário.

### Exemplos

Desativar as atualizações (uma imagem travada, uma rede isolada):

```toml
[updates]
mode = "off"
```

Fixar a frota na linha 1.4, só notificar e dizer a quem perguntar:

```toml
[updates]
mode = "notify"
pin = "1.4"
allow_self_update = false
message = "Updates go through the software center. Ask the service desk."
```

Verificar um feed interno do Chocolatey (NuGet):

```toml
[updates]
check = "nuget"
check_url = "https://nexus.example/repository/choco-internal/"
```

O mesmo tipo de política no Windows pelo registro (PowerShell):

```
New-Item -Path HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Force
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name mode -Value notify
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name pin -Value 1.4
```

### latest.json e download_base: um espelho da release

Copie os arquivos de uma release (os arquivos compactados, o instalador, os arquivos `.sha256`, o `latest.json`, **todos os arquivos de assinatura `.minisig`** e o `security.json`) para uma pasta de um servidor web interno, mantendo os nomes. O `latest.json` é anexado a toda release:

```json
{
  "version": "1.4.2",
  "notes": "Markdown release notes",
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

Os canais de arquivo e de instalador verificam o manifesto e baixam de `download_base`, com a mesma verificação que vale para o GitHub. Um espelho só precisa copiar arquivos; não precisa de API.

As assinaturas viajam com os arquivos, então um espelho não consegue enfraquecê-las:

- O `latest.json` só é confiável quando o `latest.json.minisig` está ao lado dele e confere com a chave do projeto embutida no GitPoulpi. Um manifesto sem a assinatura, ou editado depois de assinado, é recusado (os hashes dentro dele são aquilo contra o que os downloads são conferidos).
- Cada arquivo é baixado junto com o `<archive>.minisig` do mesmo host e só é instalado quando essa assinatura confere e nomeia este arquivo e esta versão. Um espelho que copiou o arquivo mas não o `.minisig` dele faz a atualização falhar com "sem assinatura", e nada é alterado.
- O `security.json` (os avisos de segurança, veja [Atualizações](updates.md#avisos-de-segurança)) e o `security.json.minisig` são procurados ao lado do manifesto. Um espelho que não os copia simplesmente não mostra avisos.

Copie os arquivos sem alterá-los: uma assinatura cobre os bytes exatos. Os avisos de segurança são republicados para a release mais recente sem uma nova build, então atualize `latest.json`, `security.json` e seus arquivos `.minisig` a partir da página de release sempre que atualizar o espelho.

## Implantando um config.toml padrão

O GitPoulpi não tem um `config.toml` para a máquina inteira: as configurações ficam na pasta de cada usuário (`%APPDATA%\GitPoulpi\config.toml`, `~/.config/gitpoulpi/config.toml`, `~/Library/Application Support/GitPoulpi/config.toml`). Para dar seus padrões a novos usuários, copie um `config.toml` esparso (só as chaves que você quer, veja a [Referência de configuração](config-reference.md)) para essa pasta quando o usuário não tiver nenhum: um script de logon ou do Intune no Windows, `/etc/skel/.config/gitpoulpi/` no Linux, um script de primeiro login no macOS. Nada nos pacotes do GitPoulpi sobrescreve uma configuração, então ela é sua para gerenciar. Não coloque ali regras de atualização que precisam valer: use a política, que um usuário não consegue substituir. Uma configuração portátil ou por tarefa também pode ser escolhida com `GITPOULPI_CONFIG_DIR`.
