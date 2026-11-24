[English](../lfs.md) · **Čeština** · [Deutsch](../de/lfs.md) · [Español](../es/lfs.md) · [Français](../fr/lfs.md) · [Italiano](../it/lfs.md) · [日本語](../ja/lfs.md) · [한국어](../ko/lfs.md) · [Nederlands](../nl/lfs.md) · [Polski](../pl/lfs.md) · [Português (Brasil)](../pt-BR/lfs.md) · [Português (Portugal)](../pt-PT/lfs.md) · [Русский](../ru/lfs.md) · [Svenska](../sv/lfs.md) · [Türkçe](../tr/lfs.md) · [Українська](../uk/lfs.md) · [简体中文](../zh-CN/lfs.md)

# Git LFS

Git LFS („Large File Storage“) drží velké soubory (návrhy, videa, datové sady) mimo historii repozitáře: git místo nich ukládá třířádkový textový *ukazatel* a skutečný obsah žije na serveru LFS a v lokální cache pod `.git/lfs`.

GitPoulpi k tomu nepotřebuje žádný účet. LFS používá přihlašovací údaje, které váš remote už používá (váš credential helper nebo ssh-agent).

## Co potřebuje program `git-lfs` a co ne

GitPoulpi spouští standardní program `git-lfs` pro vše, co soubory převádí nebo přesouvá (install, track, fetch, pull, push, zámky).

Čtení *stavu* LFS žádný program nepotřebuje: zda repozitář LFS používá, které soubory jsou ukazatele, jak jsou velké a zda je skutečný objekt už uložen lokálně.

| | potřebuje `git-lfs` |
|---|---|
| `gitpoulpi lfs status` (chybějící program oznámí, místo aby selhal) | ne |
| `gitpoulpi lfs files` | ne |
| install / track / untrack / prune | ano |
| fetch / pull / push | ano |
| zámky | ano |

## Instalace `git-lfs`

- **Windows**: je součástí Git for Windows; nebo `winget install GitHub.GitLFS`.
- **Debian / Ubuntu**: `apt install git-lfs`.
- **macOS**: `brew install git-lfs`.
- Cokoli jiného: https://git-lfs.com

Je vyžadována verze 3.0 nebo novější. Po instalaci to GitPoulpi zjistí do minuty (nebo okamžitě při `lfs status`).

## Když `git-lfs` chybí

GitPoulpi kvůli tomu nikdy nespadne ani neskončí:

- `gitpoulpi lfs status` uspěje a řekne `git-lfs: not installed` s odkazem na instalaci.
- Operace, která program potřebuje, selže s jasnou zprávou končící odkazem na instalaci (`https://git-lfs.com`) a nenulovým stavem ukončení; přes MCP nástroj tuto zprávu vrátí.
- Repozitář, který LFS používá, se přesto otevře; jeho velké soubory se jen zobrazují jako ukazatele, dokud program nenainstalujete.

## Prohlížení souborů LFS

```
gitpoulpi lfs status            # program, patterns, filters, hook
gitpoulpi lfs files             # pointer files in the index, with size and local/not downloaded
gitpoulpi lfs files --at v1.0   # ... as of a revision
gitpoulpi show big.bin --at HEAD  # reports "Git LFS object, 12.3 MB, sha256:..." instead of pointer text
```

## Nastavení repozitáře: install, track, untrack

```
gitpoulpi lfs install                 # filters + hooks for THIS repository (git lfs install --local)
gitpoulpi lfs install --user          # into your global git config instead (never done silently)
gitpoulpi lfs track '*.psd'           # route *.psd through LFS
gitpoulpi lfs track '*.psd' --lockable   # ... and make them read-only until locked
gitpoulpi lfs track '*.bin' --renormalize  # ... and re-stage already-committed *.bin as pointers
gitpoulpi lfs untrack '*.psd'
gitpoulpi lfs prune                   # delete old, already-pushed objects from the local store
```

- `install` ve výchozím stavu zapisuje do vlastního `.git/config` a hooků repozitáře, takže se vaše globální konfigurace gitu nikdy nedotkne, pokud nezadáte `--user`.
- `track` a `untrack` upravují `.gitattributes` a nechávají ho **unstaged**: zobrazí se jako běžná změna v pracovním stromu, kterou commitnete sami. Soubory přidané *po* zapnutí sledování se příštím commitem uloží jako ukazatele.
- Soubory, které jsou **už commitnuté** jako běžný obsah, zůstávají v historii běžné. `--renormalize` je znovu přidá do indexu, takže je váš příští commit nahradí ukazateli. Chcete-li je přesunout i ve všech minulých commitech, použijte pravidlo přepisu historie *Převést soubory do LFS* (viz [Přepis historie](history-rewrite.md#přesun-souborů-do-git-lfs-nebo-z-něj)); je to vlastní pravidlo GitPoulpi, ne `git lfs migrate`.
- Vzor uzavřete do uvozovek, aby ho shell nerozbalil.
- Přes MCP jde o běžné mutace (`lfs_install`, `lfs_track`, `lfs_untrack`, `lfs_prune`), na které se vztahuje omezení `[mcp]` v `config.toml`.

## Přenos objektů: fetch, pull, push

```
gitpoulpi lfs fetch [--remote origin] [--all]   # download objects into the local store; working tree untouched
gitpoulpi lfs pull  [--remote origin]           # fetch, then replace pointer files with the real content
gitpoulpi lfs push  [--remote origin] [--all]   # upload the current branch's objects (or every ref with --all)
```

- Remote je ve výchozím stavu výchozí remote repozitáře, stejně jako u `fetch` a `push`.
- Přihlašovací údaje jsou obvyklé údaje vašeho remote (credential helper, ssh-agent). GitPoulpi nikdy nežádá o vlastní účet.
- `pull` nainstaluje filtry LFS pro tento repozitář, pokud chybí: bez nich git-lfs objekty stáhne, ale nahrazení ukazatelů přeskočí a pozdější checkout by ukazatele vrátil.
- MCP zachází s `lfs push` jako s `push`: je to destruktivní nástroj (mění remote).
- Okno ukazuje průběh přenosu (soubory a bajty) na stavovém řádku. CLI počká na výsledek.

### Checkout obsahu: `lfs checkout`

```
gitpoulpi lfs checkout [<path>...]   # offline: write real content over the pointer files, from the local store
```

Nikdy nestahuje (to dělá `lfs pull`); objekty chybějící v lokálním úložišti zůstanou ukazateli. Stejně jako `pull` nejprve nainstaluje filtry LFS repozitáře, pokud žádné nejsou nastaveny. Jako každou operaci LFS ji nelze vrátit zpět; viz [Zpět a znovu](undo-redo.md). Přes MCP je to `lfs_checkout` (zápis, nikoli destruktivní).

### Tlačítko LFS na panelu nástrojů a pull / push

Panel nástrojů má za **Pop** tlačítko **`LFS ▾`** (skryté, je-li `git.lfs_enabled` vypnuto, a v repozitáři, kde LFS není ani inicializované, ani ho nepoužívá jeho `.gitattributes`: viz [Nastavení > LFS](settings.md#nastavení-repozitáře-lfs)). Kliknutím se otevře rozbalovací nabídka: **Fetch všech souborů LFS** (`lfs fetch --all`), **Checkout všech souborů LFS** (`lfs checkout`), **Pull všech souborů LFS** (`lfs pull`), **Push všech souborů LFS** (`lfs push --all`, objekty všech refs) a **Vyčistit lokální úložiště LFS** (`lfs prune`). Každý řádek je také příkaz palety (`LFS: Fetch všech objektů`, `LFS: Checkout všech souborů`, `LFS: Pull objektů`, `LFS: Push všech objektů`, `LFS: Pročistit lokální objekty`). `LFS: Otevřít nabídku` otevře rozbalovací nabídku. Každý má akord `Ctrl+G`: `l` pro nabídku, pak `Shift+F`, `Shift+K`, `Shift+L`, `Shift+J` a `Shift+V` pro řádky. Můžete je změnit v *Nastavení > Klávesnice a myš*. Když `git-lfs` chybí, každý řádek vede do dialogu instalace.

![Rozbalovací nabídka LFS na panelu nástrojů: fetch, checkout, pull a push všech souborů LFS a vyčištění lokálního úložiště](../images/lfs-menu.png)

**Pull a push si s LFS poradí samy.** V repozitáři, který používá LFS, s nainstalovaným `git-lfs`, běžný `pull` nejprve ověří, že jsou nainstalovány filtry LFS repozitáře, a běžný `push` ověří jeho hook `pre-push`. Pokud jeden chybí, GitPoulpi před přenosem spustí `git lfs install --local` (jen tento repozitář) a stavový řádek řekne „filtry LFS nainstalovány“. Bez hooku by push nenahrál žádné objekty LFS. Vypnete to pomocí `git.lfs_autofix_filters = false` (*Nastavení > Git a kódování > Instalovat filtry LFS před pull a push*). Běžný `push` stále nahrává jen objekty odesílané větve, nikdy objekty všech refs; prune není nikdy implicitní.

## Zámky souborů

Některé soubory nelze sloučit (soubor Photoshopu, level, binární asset). *Zámky* Git LFS umožňují jedné osobě říct „upravuji to“: nikdo jiný ho nemůže zamknout ani do něj pushnout změny, dokud se neodemkne.

```
gitpoulpi lfs locks  [--remote origin]              # who holds which file (yours are marked)
gitpoulpi lfs lock   assets/hero.psd [--remote origin]
gitpoulpi lfs unlock assets/hero.psd [--remote origin] [--force]
```

- Zamykání je funkce **serveru LFS** (má ji GitHub, GitLab a Gitea). Používá přihlašovací údaje remote; GitPoulpi nemá vlastní účet.
- Server bez zamykání způsobí, že `lock` selže se zprávou, která to říká (typovaná chyba `LfsLockingUnsupported`). `locks` na takovém serveru nevypíše nic, což hlásí i samotný git-lfs.
- Sledování vzoru s `--lockable` (`lfs track '*.psd' --lockable`) učiní tyto soubory jen pro čtení, dokud je nezamknete.
- `unlock` cizího zámku se odmítne; `--force` ho prolomí, a proto MCP zachází s `lfs_unlock` jako s destruktivním nástrojem. `lfs_locks` je jen pro čtení.

## Klonování repozitáře LFS

`gitpoulpi clone <url>` (a dialog Klonovat v TUI) provede checkout repozitáře *bez* spuštění filtrů LFS a poté, když repozitář LFS používá a `git-lfs` je nainstalován, stáhne objekty příkazem `git lfs pull` (nainstaluje filtry pro daný repozitář, pokud žádné nejsou nastaveny) a zobrazí to jako samostatnou fázi, *Stahování objektů LFS*. `--no-lfs` tento krok přeskočí a ponechá ukazatele.

Klon **nikdy neselže kvůli LFS**: když `git-lfs` chybí nebo stahování selže, klon přesto uspěje a velké soubory zůstanou ukazateli; stav LFS repozitáře řekne proč a `gitpoulpi lfs pull` práci dokončí, jakmile se problém opraví.

## V terminálovém UI

- **Tag `LFS`.** Soubor uložený přes Git LFS má za názvem malý štítek `LFS` v seznamech Unstaged a Staged v panelu Stav a v seznamu změněných souborů commitu. **Jen LFS** (`o` při fokusu na panelu Stav nebo Logu, políčko na panelu nástrojů seznamů souborů, nebo **Soubory: Přepnout jen LFS** v paletě) zúží tyto seznamy na soubory LFS. Ukazuje se jen v repozitáři, který nějaké má. Soubor se počítá jako LFS, když je jeho cesta v indexu ukazatel, stejným čtením, které krmí štítek (jedno dávkové volání `git` na obnovení, nic pro repozitář bez atributů `filter=lfs`).
- **Karta diffu.** Otevření diffu souboru LFS ukáže místo textu ukazatele kartu: `Objekt Git LFS`, `1.2 MB -> 1.4 MB`, krátké staré a nové oid a zda je nový objekt uložen lokálně, nebo `nestažen`. Klávesou `r` zobrazíte surový diff ukazatele, dalším `r` se vrátíte. Na soubor LFS platí jen stage na úrovni souboru (nejsou tu žádné hunky ani řádky, které by šlo přidat do indexu). Pod kartou **Otevřít výchozím programem** a **Otevřít v** (`o`, `Shift+O` nebo paleta) otevřou skutečný soubor, nikdy ukazatel: viz [Otevřít v](open-in.md#soubory-git-lfs).
- **Zobrazení Soubor** u souboru LFS říká `Git LFS object, 12.3 MB, sha256:ab12cd34...` a kde obsah získat.
- **Banner.** U repozitáře, který používá LFS, zatímco `git-lfs` chybí nebo je starší než 3.0, to řekne banner v panelu Stav. Kliknutím na něj (nebo položkou palety *LFS: Otevřít stránku instalace*) otevřete https://git-lfs.com. Nic dalšího se nemění: okno se otevře, diffy fungují, velké soubory se jen zobrazují jako ukazatele.
- **Nabídka souboru a paleta.** Klikněte pravým tlačítkem na změněný soubor v panelu Stav: *Sledovat přes LFS: tento soubor* / *tuto příponu* u běžného souboru; *Zamknout soubor (LFS)*, *Odemknout soubor (LFS)* nebo *Vynutit odemknutí (LFS, cizí zámek)* u souboru LFS podle toho, kdo drží jeho zámek. Řádek zamčeného souboru říká `zamkl vy` / `zamkl <jméno>`. Vše je také příkazem palety (`Ctrl+P`, napište `LFS`): inicializovat, sledovat nebo přestat sledovat vzor, fetch, pull nebo push objektů, prune, zobrazit zámky, otevřít stránku instalace.
- **Neúspěšná akce se vysvětlí sama.** Pokud akce LFS selže, protože `git-lfs` chybí, dialog nese zprávu a tlačítko *Otevřít stránku instalace*.
- **Nastavení.** *Nastavení > LFS* (pro jednotlivé repozitáře) ukazuje, zda je LFS inicializované, vypisuje sledované vzory a má **Zprovoznit LFS zde**. *Git a kódování* (pro jednotlivé repozitáře) má přepínač **Git LFS**, řádek **Program git-lfs** jen pro čtení a tlačítko **Inicializovat LFS pro tento repozitář**.
- Vypnutí **Git LFS** (`git.lfs_enabled = false`, viz [referenční příručka konfigurace](config-reference.md)) toto vše skryje. Vlastní filtry LFS gitu se dál spouštějí.
- Stav LFS se čte krátce po otevření repozitáře, po každé změně, při ručním obnovení, při změně `.gitattributes` a každých deset sekund u repozitáře, který LFS používá.

## Známá omezení

- Repozitář, jehož vzory LFS žijí jen ve *vnořeném* `.gitattributes` (ne v kořenovém, ne v `.git/info/attributes`), je rozpoznán jako používající LFS, až když má úložiště objektů LFS (`.git/lfs`), dříve ne.
- Samotný `git lfs migrate` se nepoužívá. Přesun souborů do LFS nebo z něj napříč historií obstarávají pravidla přepisu historie *Převést soubory do LFS* / *Vrátit soubory z LFS*: pracují na lokálním klonu, zapisuje se jen kořenový `.gitattributes` a objekty se pak ještě musí odeslat (*Odeslat objekty LFS* v panelu s výsledkem, nebo `lfs push --all`).
