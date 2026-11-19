[English](../docker.md) · [Čeština](../cs/docker.md) · [Deutsch](../de/docker.md) · [Español](../es/docker.md) · **Français** · [Italiano](../it/docker.md) · [日本語](../ja/docker.md) · [한국어](../ko/docker.md) · [Nederlands](../nl/docker.md) · [Polski](../pl/docker.md) · [Português (Brasil)](../pt-BR/docker.md) · [Português (Portugal)](../pt-PT/docker.md) · [Русский](../ru/docker.md) · [Svenska](../sv/docker.md) · [Türkçe](../tr/docker.md) · [Українська](../uk/docker.md) · [简体中文](../zh-CN/docker.md)

# Exécuter dans Docker

L'image `poulpilab/gitpoulpi` contient les binaires officiels des versions (`gitpoulpi`, plus les relais `gitpoulpi-tui` et `gitpoulpi-mcp`) sur `debian:bookworm-slim`, avec `git`, `git-lfs`, `openssh-client`, `less` et `tini`. Elle fonctionne sur `linux/amd64` et `linux/arm64`. Elle est sur Docker Hub.

## Lancer l'interface

```
docker run -it --rm \
  --user "$(id -u):$(id -g)" \
  -v "$PWD":/work \
  -v gitpoulpi-config:/config \
  poulpilab/gitpoulpi
```

- `-it` donne un terminal à l'interface. Sans lui, le programme n'a rien sur quoi dessiner.
- `-v "$PWD":/work` monte le dossier courant comme dossier de travail (`/work`). Ouvrez un dépôt à partir de là, ou montez plusieurs dépôts sous `/work`.
- `-v gitpoulpi-config:/config` est un volume nommé qui conserve vos paramètres et vos sessions d'une exécution à l'autre.
- `--user "$(id -u):$(id -g)"` lance le conteneur sous votre identité : les fichiers que GitPoulpi crée dans vos dépôts gardent votre propriétaire sur l'hôte au lieu de `root`. Chaque dossier où l'image écrit est accessible en écriture à tous, n'importe quel identifiant d'utilisateur convient donc.

## Volumes et chemins

| Chemin | Ce qu'il contient |
|---|---|
| `/work` | Le dossier de travail. Montez-y vos dépôts. |
| `/config` | Les paramètres (`config.toml`), les sessions et, en sous-dossiers, les données et le cache (`GITPOULPI_CONFIG_DIR=/config`). Utilisez un volume nommé ou un montage lié pour les conserver. |
| `/data`, `/cache` | Déclarés aussi comme volumes, pour les outils qui suivent les variables XDG (`XDG_DATA_HOME=/data`, `XDG_CACHE_HOME=/cache`). |
| `/home/gitpoulpi` | `HOME`. Ce n'est pas un volume : un neuf par conteneur. |

Sans volume pour `/config`, chaque exécution avec `--rm` repart avec les paramètres par défaut. Pour partir de votre propre `config.toml`, montez-le : `-v "$PWD/config.toml":/config/config.toml:ro` (lecture seule signifie que les Paramètres ne peuvent pas enregistrer).

## Dépôts appartenant à un autre utilisateur

Git refuse un dépôt dont le propriétaire diffère de l'utilisateur qui l'exécute (« dubious ownership »). Avec `--user` comme ci-dessus, le propriétaire correspond. Quand ce n'est pas le cas (un montage lié depuis un autre compte, un partage Docker Desktop), approuvez tous les dossiers avec :

```
docker run -it --rm -e GITPOULPI_SAFE_DIRECTORY='*' -v "$PWD":/work poulpilab/gitpoulpi
```

Le point d'entrée lance alors `git config --global --add safe.directory '*'` avant de démarrer l'interface. N'utilisez cela que pour des dossiers de confiance, car cela lève la protection de git pour tout le conteneur.

## SSH

Transmettez votre agent SSH au lieu de copier des clés dans le conteneur.

Linux :

```
docker run -it --rm \
  -v "$SSH_AUTH_SOCK":/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

Docker Desktop (macOS, Windows) expose l'agent de l'hôte à un chemin fixe dans sa VM :

```
docker run -it --rm \
  -v /run/host-services/ssh-auth.sock:/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

Avec `--user`, le socket doit être lisible par votre identifiant d'utilisateur, ce qui est le cas quand il vous appartient. Le conteneur n'a pas de `known_hosts` : la première connexion à un hôte vous demande d'accepter sa clé, ou montez votre propre fichier avec `-v ~/.ssh/known_hosts:/home/gitpoulpi/.ssh/known_hosts:ro`.

## Identité Git et identifiants

- **Identité et paramètres :** montez votre configuration Git en lecture seule : `-v ~/.gitconfig:/home/gitpoulpi/.gitconfig:ro`. Ou définissez l'identité de commit dans un profil GitPoulpi (Paramètres > Profils), qui vit dans `/config`.
- **Identifiants HTTPS :** l'image n'a pas d'assistant d'identifiants comme un trousseau. Montez un magasin d'identifiants (`-v ~/.git-credentials:/home/gitpoulpi/.git-credentials:ro`, avec `credential.helper = store` dans le `.gitconfig` monté), ou utilisez un distant SSH avec l'agent transmis.
- **Jetons d'hébergement** (pull requests, tickets) : sans assistant d'identifiants, GitPoulpi garde un jeton en mémoire pour la session seulement. Pour une configuration durable, passez-le par l'environnement avec `-e GITPOULPI_TOKEN_<ID>=...`, où `<ID>` est l'identifiant de l'intégration en majuscules avec les caractères non alphanumériques remplacés par `_` (voir [Hébergeurs et suivi de tickets](hosting-integration.md)). Les intégrations elles-mêmes sont dans `config.toml` sous `/config`.

## Terminal, couleurs et souris

Les couleurs et la souris dépendent de votre émulateur de terminal. Docker transmet `TERM` à partir de `-t` ; ajoutez `-e TERM -e COLORTERM` quand les couleurs ou le thème en couleurs vraies semblent incorrects, par exemple `-e COLORTERM=truecolor`. Les clics, les glisser-déposer et la molette arrivent par le terminal comme d'habitude. Le graphe des commits est dessiné avec des caractères ; les images (kitty, iTerm2, sixel) ne sont pas transmises par tous les terminaux et multiplexeurs, voir [Choisir un terminal](terminals.md). Redimensionner la fenêtre du terminal redimensionne l'interface : le conteneur reçoit le changement de taille.

## CLI et MCP

Le point d'entrée démarre l'interface. Les autres programmes sont accessibles en le remplaçant :

```
# la CLI scriptable (ajoutez --json pour les scripts)
docker run --rm --entrypoint gitpoulpi -v "$PWD":/work poulpilab/gitpoulpi status

# le serveur MCP, sur l'entrée et la sortie standard, pour un agent qui le lance
docker run -i --rm --entrypoint gitpoulpi -v "$PWD":/work poulpilab/gitpoulpi mcp serve
```

Chaque `docker run` est son propre conteneur avec son propre démon GitPoulpi d'arrière-plan : les changements faits par la CLI n'apparaissent donc pas en direct dans une interface lancée dans un autre conteneur. Pour piloter une interface depuis la CLI, lancez les deux dans le même conteneur : démarrez l'interface avec `--name gp` et utilisez `docker exec gp gitpoulpi ...`. Pour un agent, enregistrez `docker run -i --rm --entrypoint gitpoulpi ... mcp serve` comme commande du serveur MCP, voir [MCP et agents](mcp-and-agents.md) ; les options de permissions MCP du `config.toml` monté s'appliquent comme d'habitude.

## Étiquettes et mise à jour

| Étiquette | Signification |
|---|---|
| `X.Y.Z` | Une version exacte. |
| `X.Y` | Le dernier correctif de cette version mineure (versions stables seulement). |
| `latest` | La dernière version stable. |

Une préversion n'a que son étiquette exacte `X.Y.Z-...`.

Une image se remplace, elle ne se met jamais à jour sur place. GitPoulpi dans un conteneur le sait (sa trace d'installation dit « une image Docker ») : le [panneau de mise à jour](updates.md) affiche la version et les commandes et ne remplace rien. Pour mettre à jour :

```
docker pull poulpilab/gitpoulpi
```

Démarrez ensuite un nouveau conteneur. Vos paramètres restent dans le volume `/config`. Pour revenir en arrière, lancez une étiquette plus ancienne : `docker pull poulpilab/gitpoulpi:<previous>`. Pour figer une version dans des scripts, utilisez l'étiquette exacte au lieu de `latest`. Une organisation peut miroiter l'image sur son propre registre, voir [Distribution en entreprise](enterprise-distribution.md#miroir-de-registre-docker).
