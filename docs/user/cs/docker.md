[English](../docker.md) · **Čeština** · [Deutsch](../de/docker.md) · [Español](../es/docker.md) · [Français](../fr/docker.md) · [Italiano](../it/docker.md) · [日本語](../ja/docker.md) · [한국어](../ko/docker.md) · [Nederlands](../nl/docker.md) · [Polski](../pl/docker.md) · [Português (Brasil)](../pt-BR/docker.md) · [Português (Portugal)](../pt-PT/docker.md) · [Русский](../ru/docker.md) · [Svenska](../sv/docker.md) · [Türkçe](../tr/docker.md) · [Українська](../uk/docker.md) · [简体中文](../zh-CN/docker.md)

# Běh v Dockeru

Obraz `poulpilab/gitpoulpi` obsahuje oficiální binárky vydání (`gitpoulpi-tui`, `gitpoulpi`, `gitpoulpi-mcp`) na `debian:bookworm-slim`, spolu s `git`, `git-lfs`, `openssh-client`, `less` a `tini`. Běží na `linux/amd64` a `linux/arm64`. Je na Docker Hubu.

## Spuštění rozhraní

```
docker run -it --rm \
  --user "$(id -u):$(id -g)" \
  -v "$PWD":/work \
  -v gitpoulpi-config:/config \
  poulpilab/gitpoulpi
```

- `-it` dává rozhraní terminál. Bez něj program nemá kam kreslit.
- `-v "$PWD":/work` připojí aktuální složku jako pracovní adresář (`/work`). Repozitář otevřete odtud, nebo připojte několik repozitářů pod `/work`.
- `-v gitpoulpi-config:/config` je pojmenovaný svazek, který mezi spuštěními uchovává vaše nastavení a relace.
- `--user "$(id -u):$(id -g)"` spustí program jako vy, takže soubory, které GitPoulpi vytvoří ve vašich repozitářích, si ponechají vlastníka z hostitele místo `root`. Každá složka, do které obraz zapisuje, je zapisovatelná pro všechny, takže funguje jakékoli id uživatele.

## Svazky a cesty

| Cesta | Co obsahuje |
|---|---|
| `/work` | Pracovní adresář. Sem připojte (bind mount) své repozitáře. |
| `/config` | Nastavení (`config.toml`), relace a jako podsložky data a cache (`GITPOULPI_CONFIG_DIR=/config`). Pro uchování použijte pojmenovaný svazek nebo bind mount. |
| `/data`, `/cache` | Také deklarované jako svazky, pro nástroje, které se řídí proměnnými XDG (`XDG_DATA_HOME=/data`, `XDG_CACHE_HOME=/cache`). |
| `/home/gitpoulpi` | `HOME`. Není to svazek: pro každý kontejner nový. |

Bez svazku pro `/config` začíná každé spuštění s `--rm` s výchozím nastavením. Chcete-li začít z vlastního `config.toml`, připojte ho: `-v "$PWD/config.toml":/config/config.toml:ro` (jen pro čtení znamená, že Nastavení nemůže ukládat).

## Repozitáře vlastněné jiným uživatelem

Git odmítne repozitář, jehož vlastník se liší od uživatele, který ho spouští ("dubious ownership"). S `--user` jako výše vlastník sedí. Když nesedí (bind mount z jiného účtu, sdílená složka Docker Desktop), označte důvěryhodnými všechny adresáře pomocí:

```
docker run -it --rm -e GITPOULPI_SAFE_DIRECTORY='*' -v "$PWD":/work poulpilab/gitpoulpi
```

Vstupní bod pak před spuštěním rozhraní provede `git config --global --add safe.directory '*'`. Používejte to jen pro složky, kterým důvěřujete, protože to ruší ochranu gitu pro celý kontejner.

## SSH

Přeposlete svého SSH agenta místo kopírování klíčů do kontejneru.

Linux:

```
docker run -it --rm \
  -v "$SSH_AUTH_SOCK":/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

Docker Desktop (macOS, Windows) zpřístupňuje agenta hostitele na pevné cestě uvnitř své VM:

```
docker run -it --rm \
  -v /run/host-services/ssh-auth.sock:/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

S `--user` musí být soket čitelný pro vaše id uživatele, což platí, když patří vám. Kontejner nemá `known_hosts`: první připojení k hostiteli vás požádá o přijetí jeho klíče, nebo připojte vlastní soubor pomocí `-v ~/.ssh/known_hosts:/home/gitpoulpi/.ssh/known_hosts:ro`.

## Identita Gitu a přihlašovací údaje

- **Identita a nastavení:** připojte svou konfiguraci Gitu jen pro čtení: `-v ~/.gitconfig:/home/gitpoulpi/.gitconfig:ro`. Nebo nastavte identitu commitů v profilu GitPoulpi (Nastavení > Profily), který žije v `/config`.
- **Přihlašovací údaje HTTPS:** obraz nemá credential helper typu klíčenky. Buď připojte úložiště údajů (`-v ~/.git-credentials:/home/gitpoulpi/.git-credentials:ro`, s `credential.helper = store` v připojeném `.gitconfig`), nebo použijte SSH remote s přeposlaným agentem.
- **Tokeny hostingu** (pull requesty, issues): bez credential helperu drží GitPoulpi token jen v paměti po dobu relace. Pro trvalé nastavení ho předejte přes prostředí pomocí `-e GITPOULPI_TOKEN_<ID>=...`, kde `<ID>` je id integrace velkými písmeny s nealfanumerickými znaky jako `_` (viz [Hosting a issue trackery](hosting-integration.md)). Samotné integrace jsou v `config.toml` pod `/config`.

## Terminál, barvy a myš

Barvy a podpora myši pocházejí z vašeho emulátoru terminálu. Docker předává `TERM` z `-t`; když barvy nebo téma true-color vypadají špatně, přidejte `-e TERM -e COLORTERM`, například `-e COLORTERM=truecolor`. Kliknutí myší, tažení a kolečko přicházejí přes terminál jako obvykle. Graf commitů se kreslí znaky; obrázky (kitty, iTerm2, sixel) se nepřeposílají každým terminálem a multiplexerem, viz [Výběr terminálu](terminals.md). Změna velikosti okna terminálu změní velikost rozhraní: kontejner dostane změnu velikosti.

## CLI a MCP

Vstupní bod spouští rozhraní. K ostatním programům se dostanete jeho přepsáním:

```
# the scriptable CLI (add --json for scripts)
docker run --rm --entrypoint gitpoulpi -v "$PWD":/work poulpilab/gitpoulpi status

# the MCP server, over standard input and output, for an agent that launches it
docker run -i --rm --entrypoint gitpoulpi-mcp -v "$PWD":/work poulpilab/gitpoulpi
```

Každé `docker run` je samostatný kontejner s vlastním daemonem GitPoulpi na pozadí, takže změny provedené přes CLI se živě neukážou v rozhraní běžícím v jiném kontejneru. Chcete-li ovládat jedno rozhraní z CLI, spusťte obojí ve stejném kontejneru: rozhraní spusťte s `--name gp` a použijte `docker exec gp gitpoulpi ...`. Pro agenta zaregistrujte `docker run -i --rm --entrypoint gitpoulpi-mcp ...` jako příkaz serveru MCP, viz [MCP a agenti](mcp-and-agents.md); přepínače oprávnění MCP v připojeném `config.toml` platí jako obvykle.

## Tagy a aktualizace

| Tag | Význam |
|---|---|
| `X.Y.Z` | Jedno přesné vydání. |
| `X.Y` | Poslední patch dané minor verze (jen stabilní vydání). |
| `latest` | Poslední stabilní vydání. |

Předběžné vydání dostane jen svůj přesný tag `X.Y.Z-...`.

Obraz se nahrazuje, nikdy neaktualizuje na místě. GitPoulpi uvnitř kontejneru to ví (jeho záznam o instalaci říká "obraz Dockeru"): [panel aktualizací](updates.md) ukáže verzi a příkazy a nic nenahradí. Aktualizace:

```
docker pull poulpilab/gitpoulpi
```

Pak spusťte nový kontejner. Vaše nastavení zůstává ve svazku `/config`. Pro návrat spusťte starší tag: `docker pull poulpilab/gitpoulpi:<previous>`. Chcete-li ve skriptech verzi připnout, použijte přesný tag místo `latest`. Organizace může obraz zrcadlit ve vlastním registru, viz [Podniková distribuce](enterprise-distribution.md#zrcadlo-registru-dockeru).
