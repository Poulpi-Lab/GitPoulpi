[English](../docker.md) · [Čeština](../cs/docker.md) · [Deutsch](../de/docker.md) · [Español](../es/docker.md) · [Français](../fr/docker.md) · [Italiano](../it/docker.md) · [日本語](../ja/docker.md) · [한국어](../ko/docker.md) · [Nederlands](../nl/docker.md) · [Polski](../pl/docker.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/docker.md) · [Русский](../ru/docker.md) · [Svenska](../sv/docker.md) · [Türkçe](../tr/docker.md) · [Українська](../uk/docker.md) · [简体中文](../zh-CN/docker.md)

# Executando no Docker

A imagem `poulpilab/gitpoulpi` contém os binários oficiais da release (`gitpoulpi`, mais os relays `gitpoulpi-tui` e `gitpoulpi-mcp`) sobre `debian:bookworm-slim`, com `git`, `git-lfs`, `openssh-client`, `less` e `tini`. Ela roda em `linux/amd64` e `linux/arm64`. Está no Docker Hub.

## Iniciar a interface

```
docker run -it --rm \
  --user "$(id -u):$(id -g)" \
  -v "$PWD":/work \
  -v gitpoulpi-config:/config \
  poulpilab/gitpoulpi
```

- `-it` dá um terminal à interface. Sem ele, o programa não tem onde desenhar.
- `-v "$PWD":/work` monta a pasta atual como diretório de trabalho (`/work`). Abra um repositório a partir dali, ou monte vários repositórios sob `/work`.
- `-v gitpoulpi-config:/config` é um volume nomeado que mantém suas configurações e sessões entre as execuções.
- `--user "$(id -u):$(id -g)"` executa como você, para que os arquivos que o GitPoulpi cria nos seus repositórios mantenham o dono do host em vez de `root`. Toda pasta em que a imagem grava é gravável por todos, então qualquer id de usuário funciona.

## Volumes e caminhos

| Caminho | O que contém |
|---|---|
| `/work` | O diretório de trabalho. Monte seus repositórios aqui (bind mount). |
| `/config` | Configurações (`config.toml`), sessões e, em subpastas, dados e cache (`GITPOULPI_CONFIG_DIR=/config`). Use um volume nomeado ou um bind mount para mantê-los. |
| `/data`, `/cache` | Declarados também como volumes, para ferramentas que seguem as variáveis XDG (`XDG_DATA_HOME=/data`, `XDG_CACHE_HOME=/cache`). |
| `/home/gitpoulpi` | `HOME`. Não é um volume: um novo a cada contêiner. |

Sem um volume para `/config`, toda execução com `--rm` começa com as configurações padrão. Para começar do seu próprio `config.toml`, monte-o: `-v "$PWD/config.toml":/config/config.toml:ro` (somente leitura significa que as Configurações não conseguem salvar).

## Repositórios de outro usuário

O Git recusa um repositório cujo dono é diferente do usuário que o executa ("dubious ownership"). Com `--user` como acima, o dono confere. Quando não confere (um bind mount de outra conta, um compartilhamento do Docker Desktop), confie em todos os diretórios com:

```
docker run -it --rm -e GITPOULPI_SAFE_DIRECTORY='*' -v "$PWD":/work poulpilab/gitpoulpi
```

O ponto de entrada então executa `git config --global --add safe.directory '*'` antes de iniciar a interface. Use isto só para pastas em que você confia, já que suspende a proteção do git para o contêiner inteiro.

## SSH

Encaminhe o seu agente SSH em vez de copiar chaves para dentro do contêiner.

Linux:

```
docker run -it --rm \
  -v "$SSH_AUTH_SOCK":/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

O Docker Desktop (macOS, Windows) expõe o agente do host em um caminho fixo dentro da sua VM:

```
docker run -it --rm \
  -v /run/host-services/ssh-auth.sock:/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

Com `--user`, o socket precisa ser legível pelo seu id de usuário, o que acontece quando ele é seu. O contêiner não tem `known_hosts`: a primeira conexão com um host pede que você aceite a chave dele, ou monte o seu próprio arquivo com `-v ~/.ssh/known_hosts:/home/gitpoulpi/.ssh/known_hosts:ro`.

## Identidade do Git e credenciais

- **Identidade e configurações:** monte a sua configuração do Git como somente leitura: `-v ~/.gitconfig:/home/gitpoulpi/.gitconfig:ro`. Ou defina a identidade de commit em um perfil do GitPoulpi (Configurações > Perfis), que fica em `/config`.
- **Credenciais HTTPS:** a imagem não tem um auxiliar de credenciais como um chaveiro. Monte um armazenamento de credenciais (`-v ~/.git-credentials:/home/gitpoulpi/.git-credentials:ro`, com `credential.helper = store` no `.gitconfig` montado) ou use um remoto SSH com o agente encaminhado.
- **Tokens de hospedagem** (pull requests, issues): sem um auxiliar de credenciais, o GitPoulpi guarda um token só na memória, durante a sessão. Para uma configuração duradoura, passe-o pelo ambiente com `-e GITPOULPI_TOKEN_<ID>=...`, onde `<ID>` é o id da integração em maiúsculas, com os caracteres não alfanuméricos como `_` (veja [Integrações de hospedagem e de rastreadores de issues](hosting-integration.md)). As próprias integrações ficam no `config.toml` em `/config`.

## Terminal, cores e mouse

As cores e o suporte a mouse vêm do seu emulador de terminal. O Docker repassa `TERM` a partir de `-t`; acrescente `-e TERM -e COLORTERM` quando as cores ou o tema de cores reais parecerem errados, por exemplo `-e COLORTERM=truecolor`. Cliques, arrastos e a roda do mouse chegam pelo terminal como de costume. O grafo de commits é desenhado com caracteres; imagens (kitty, iTerm2, sixel) não são repassadas por todo terminal e multiplexador, veja [Escolhendo um terminal](terminals.md). Redimensionar a janela do terminal redimensiona a interface: o contêiner recebe a mudança de tamanho.

## CLI e MCP

O ponto de entrada inicia a interface. Os outros programas são alcançáveis substituindo-o:

```
# a CLI para scripts (acrescente --json para scripts)
docker run --rm --entrypoint gitpoulpi -v "$PWD":/work poulpilab/gitpoulpi status

# o servidor MCP, pela entrada e saída padrão, para um agente que o inicia
docker run -i --rm --entrypoint gitpoulpi -v "$PWD":/work poulpilab/gitpoulpi mcp serve
```

Cada `docker run` é um contêiner próprio, com o seu próprio daemon do GitPoulpi em segundo plano, então as mudanças feitas pela CLI não aparecem ao vivo em uma interface rodando em outro contêiner. Para conduzir uma interface pela CLI, execute as duas no mesmo contêiner: inicie a interface com `--name gp` e use `docker exec gp gitpoulpi ...`. Para um agente, registre `docker run -i --rm --entrypoint gitpoulpi ... mcp serve` como o comando do servidor MCP, veja [MCP e agentes](mcp-and-agents.md); as chaves de permissão do MCP no `config.toml` montado valem como de costume.

## Tags e atualização

| Tag | Significado |
|---|---|
| `X.Y.Z` | Uma release exata. |
| `X.Y` | O último patch daquela versão menor (só releases estáveis). |
| `latest` | A última release estável. |

Uma pré-release recebe só a sua tag exata `X.Y.Z-...`.

Uma imagem é substituída, nunca atualizada no lugar. O GitPoulpi dentro de um contêiner sabe disso (o registro de instalação dele diz "uma imagem Docker"): o [painel de atualização](updates.md) mostra a versão e os comandos e não substitui nada. Para atualizar:

```
docker pull poulpilab/gitpoulpi
```

Depois inicie um novo contêiner. Suas configurações ficam no volume `/config`. Para voltar atrás, execute uma tag mais antiga: `docker pull poulpilab/gitpoulpi:<anterior>`. Para fixar uma versão em scripts, use a tag exata em vez de `latest`. Uma organização pode espelhar a imagem no próprio registro, veja [Distribuição corporativa](enterprise-distribution.md#espelho-de-registro-docker).
