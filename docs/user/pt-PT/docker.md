[English](../docker.md) · [Čeština](../cs/docker.md) · [Deutsch](../de/docker.md) · [Español](../es/docker.md) · [Français](../fr/docker.md) · [Italiano](../it/docker.md) · [日本語](../ja/docker.md) · [한국어](../ko/docker.md) · [Nederlands](../nl/docker.md) · [Polski](../pl/docker.md) · [Português (Brasil)](../pt-BR/docker.md) · **Português (Portugal)** · [Русский](../ru/docker.md) · [Svenska](../sv/docker.md) · [Türkçe](../tr/docker.md) · [Українська](../uk/docker.md) · [简体中文](../zh-CN/docker.md)

# Executar no Docker

A imagem `poulpilab/gitpoulpi` contém os binários oficiais da versão (`gitpoulpi`, mais os relays `gitpoulpi-tui` e `gitpoulpi-mcp`) sobre `debian:bookworm-slim`, com `git`, `git-lfs`, `openssh-client`, `less` e `tini`. Funciona em `linux/amd64` e `linux/arm64`. Está no Docker Hub.

## Iniciar a interface

```
docker run -it --rm \
  --user "$(id -u):$(id -g)" \
  -v "$PWD":/work \
  -v gitpoulpi-config:/config \
  poulpilab/gitpoulpi
```

- `-it` dá um terminal à interface. Sem ele, o programa não tem onde desenhar.
- `-v "$PWD":/work` monta a pasta atual como pasta de trabalho (`/work`). Abra um repositório a partir dela, ou monte vários repositórios em `/work`.
- `-v gitpoulpi-config:/config` é um volume com nome que guarda as suas definições e sessões entre execuções.
- `--user "$(id -u):$(id -g)"` executa como o utilizador atual, para que os ficheiros que o GitPoulpi cria nos seus repositórios mantenham o dono do anfitrião em vez de `root`. Todas as pastas onde a imagem escreve têm permissão de escrita para todos, pelo que qualquer id de utilizador serve.

## Volumes e caminhos

| Caminho | O que contém |
|---|---|
| `/work` | A pasta de trabalho. Monte aqui os seus repositórios (bind mount). |
| `/config` | Definições (`config.toml`), sessões e, em subpastas, dados e cache (`GITPOULPI_CONFIG_DIR=/config`). Use um volume com nome ou um bind mount para os manter. |
| `/data`, `/cache` | Declarados também como volumes, para ferramentas que seguem as variáveis XDG (`XDG_DATA_HOME=/data`, `XDG_CACHE_HOME=/cache`). |
| `/home/gitpoulpi` | `HOME`. Não é um volume: é novo em cada contentor. |

Sem um volume para `/config`, cada execução com `--rm` começa com as definições predefinidas. Para começar a partir do seu próprio `config.toml`, monte-o: `-v "$PWD/config.toml":/config/config.toml:ro` (só de leitura significa que as Definições não conseguem guardar).

## Repositórios de outro utilizador

O git recusa um repositório cujo dono é diferente do utilizador que o executa ("dubious ownership"). Com `--user`, como acima, o dono coincide. Quando não coincide (um bind mount de outra conta, uma partilha do Docker Desktop), confie em todas as pastas com:

```
docker run -it --rm -e GITPOULPI_SAFE_DIRECTORY='*' -v "$PWD":/work poulpilab/gitpoulpi
```

O ponto de entrada executa então `git config --global --add safe.directory '*'` antes de iniciar a interface. Use-o só para pastas em que confia, porque levanta a proteção do git em todo o contentor.

## SSH

Reencaminhe o seu agente SSH em vez de copiar chaves para o contentor.

Linux:

```
docker run -it --rm \
  -v "$SSH_AUTH_SOCK":/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

O Docker Desktop (macOS, Windows) expõe o agente do anfitrião num caminho fixo dentro da sua VM:

```
docker run -it --rm \
  -v /run/host-services/ssh-auth.sock:/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

Com `--user`, o socket tem de ser legível pelo seu id de utilizador, o que acontece quando lhe pertence. O contentor não tem `known_hosts`: a primeira ligação a um anfitrião pede-lhe que aceite a sua chave, ou monte o seu próprio ficheiro com `-v ~/.ssh/known_hosts:/home/gitpoulpi/.ssh/known_hosts:ro`.

## Identidade do git e credenciais

- **Identidade e definições:** monte a sua configuração do Git só de leitura: `-v ~/.gitconfig:/home/gitpoulpi/.gitconfig:ro`. Ou defina a identidade dos commits num perfil do GitPoulpi (Definições > Perfis), que fica em `/config`.
- **Credenciais HTTPS:** a imagem não tem um auxiliar de credenciais como um chaveiro. Monte um armazém de credenciais (`-v ~/.git-credentials:/home/gitpoulpi/.git-credentials:ro`, com `credential.helper = store` no `.gitconfig` montado), ou use um remoto SSH com o agente reencaminhado.
- **Tokens de alojamento** (pull requests, issues): sem um auxiliar de credenciais, o GitPoulpi mantém um token em memória apenas durante a sessão. Para uma configuração duradoura, passe-o pelo ambiente com `-e GITPOULPI_TOKEN_<ID>=...`, onde `<ID>` é o id da integração em maiúsculas, com os carateres não alfanuméricos como `_` (veja [Integrações de alojamento e de gestores de issues](hosting-integration.md)). As próprias integrações estão em `config.toml`, em `/config`.

## Terminal, cores e rato

As cores e o suporte de rato vêm do seu emulador de terminal. O Docker transmite `TERM` a partir de `-t`; acrescente `-e TERM -e COLORTERM` quando as cores ou o tema de cor verdadeira parecerem errados, por exemplo `-e COLORTERM=truecolor`. Os cliques, arrastos e a roda do rato chegam através do terminal como de costume. O grafo de commits é desenhado com carateres; as imagens (kitty, iTerm2, sixel) não são reencaminhadas por todos os terminais e multiplexadores, veja [Escolher um terminal](terminals.md). Redimensionar a janela do terminal redimensiona a interface: o contentor recebe a alteração de tamanho.

## CLI e MCP

O ponto de entrada inicia a interface. Os outros programas alcançam-se substituindo-o:

```
# a CLI para scripts (acrescente --json para scripts)
docker run --rm --entrypoint gitpoulpi -v "$PWD":/work poulpilab/gitpoulpi status

# o servidor MCP, por entrada e saída padrão, para um agente que o inicia
docker run -i --rm --entrypoint gitpoulpi -v "$PWD":/work poulpilab/gitpoulpi mcp serve
```

Cada `docker run` é o seu próprio contentor, com o seu próprio daemon do GitPoulpi em segundo plano, pelo que as alterações feitas pela CLI não aparecem em direto numa interface a correr noutro contentor. Para comandar uma interface a partir da CLI, execute ambas no mesmo contentor: inicie a interface com `--name gp` e use `docker exec gp gitpoulpi ...`. Para um agente, registe `docker run -i --rm --entrypoint gitpoulpi ... mcp serve` como o comando do servidor MCP, veja [MCP e agentes](mcp-and-agents.md); os interruptores de permissões do MCP no `config.toml` montado aplicam-se como de costume.

## Etiquetas e atualização

| Etiqueta | Significado |
|---|---|
| `X.Y.Z` | Uma versão exata. |
| `X.Y` | O último patch dessa versão menor (só versões estáveis). |
| `latest` | A última versão estável. |

Uma pré-versão só recebe a sua etiqueta exata `X.Y.Z-...`.

Uma imagem é substituída, nunca atualizada no próprio sítio. O GitPoulpi dentro de um contentor sabe-o (o seu registo de instalação diz "uma imagem Docker"): o [painel de atualizações](updates.md) mostra a versão e os comandos e não substitui nada. Para atualizar:

```
docker pull poulpilab/gitpoulpi
```

Depois inicie um novo contentor. As suas definições ficam no volume `/config`. Para voltar atrás, execute uma etiqueta mais antiga: `docker pull poulpilab/gitpoulpi:<anterior>`. Para fixar uma versão em scripts, use a etiqueta exata em vez de `latest`. Uma organização pode espelhar a imagem no seu próprio registo, veja [Distribuição empresarial](enterprise-distribution.md#espelho-de-registo-docker).
