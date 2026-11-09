[English](../troubleshooting.md) · [Čeština](../cs/troubleshooting.md) · [Deutsch](../de/troubleshooting.md) · [Español](../es/troubleshooting.md) · [Français](../fr/troubleshooting.md) · [Italiano](../it/troubleshooting.md) · [日本語](../ja/troubleshooting.md) · [한국어](../ko/troubleshooting.md) · **Nederlands** · [Polski](../pl/troubleshooting.md) · [Português (Brasil)](../pt-BR/troubleshooting.md) · [Português (Portugal)](../pt-PT/troubleshooting.md) · [Русский](../ru/troubleshooting.md) · [Svenska](../sv/troubleshooting.md) · [Türkçe](../tr/troubleshooting.md) · [Українська](../uk/troubleshooting.md) · [简体中文](../zh-CN/troubleshooting.md)

# Probleemoplossing en veelgestelde vragen

Begin met de snelle antwoorden hieronder; de langere secties gaan over Git LFS en
het tekenen van de grafiek. Helpt niets daarvan, open dan een issue in de
GitHub-repository met je besturingssysteem, je terminal, `gitpoulpi --version` en
wat je deed.

## Wanneer er iets mislukt

Een mislukte bewerking toont geen pop-up en geen omlijste kaart: de **statusbalk** zegt het, in het rood, met de titel en de reden (*fout: Fetch mislukt: De remote is niet bereikbaar.*), en het blijft daar staan tot een ander bericht het vervangt. De knop Activiteit aan zijn rechteruiteinde telt de fouten die je nog niet hebt bekeken. Klik op het rode bericht, of voer in het palet `Activiteitenlogboek: Laatste fout openen` uit, om het activiteitenlogboek op dat item te openen. Elke mislukking die je zelf veroorzaakt werkt zo, niet alleen git-bewerkingen:

- **git-bewerkingen**: een fetch, een push, een pull, een checkout, een merge, een rebase, een commit, een stash...
- **Een kloon** (en het maken van een nieuwe repository): "Klonen mislukt".
- **Een aanroep naar GitHub of GitLab**: de repositorylijsten van de keuzelijsten en het dialoogvenster Klonen, het maken van een pull request of een issue, het controleren van een token of SSH-sleutel op de pagina Integraties: "Verzoek aan de hostingdienst mislukt".
- **Een extern programma**: de editor, de diff- of merge-tool, de bestandsverkenner, de browser, de terminal of de launcher, een afbeeldingseditor, de *Testen*-knoppen van de Instellingen: "Extern programma mislukt".
- **Een AI-verzoek** (een commit-bericht, een uitleg, een suggestie voor een conflict...): "AI-verzoek mislukt".
- **Een bestand dat niet geschreven kon worden**: Opslaan van de ingebouwde editor, de merge-editor, een instelling die niet opgeslagen kon worden: "Opslaan mislukt".

Het rode bericht staat op de statusbalk welk beeld er ook voorop staat: een repositorytabblad, Start, de Radar, de Werkruimten, de Instellingen (op de onderste rij van het scherm), onder het activiteitenlogboek of een dialoogvenster. Waar de statusbalk bedekt is (Start, de Radar, de Werkruimten, de Instellingen) wordt het bericht over zijn onderste rij geschreven tot je naar de fout hebt gekeken. Het volgt je van het ene tabblad naar het andere. Een bewerking die nog loopt als je van tabblad wisselt, wordt niet gevolgd: open het activiteitenlogboek om te zien hoe ze afliep.

De lijsten van de secties PULL REQUESTS en ISSUES die niet vernieuwd kunnen worden, houden in plaats daarvan hun eigen regel in de sectie (ze vernieuwen zichzelf, een bericht om de paar seconden zou alleen ruis zijn). Een dialoogvenster dat nog openstaat (Klonen, Pull request maken...) houdt ook zijn eigen bericht van één regel, zodat je het veld kunt verbeteren en het opnieuw kunt proberen.

Het item in het activiteitenlogboek bevat alles wat de pop-up vroeger toonde:

- **Titel en reden.** "Fetch mislukt", dan één zin die in gewone woorden zegt waarom, dan wat je nu kunt doen. Herkend worden: de remote heeft commits die je niet hebt (*doe eerst een pull*), inloggegevens geweigerd (*controleer je inloggegevens onder Instellingen > Integraties, of je SSH-sleutel*), een stop bij conflicten, lokale wijzigingen die zouden worden overschreven (*maak een commit of gooi ze weg, of doe een stash*), de remote niet bereikbaar, en een ander git-proces dat een vergrendeling op de repository houdt. Al het andere toont de eerste regel van git zelf. Een kloon of een aanroep naar de host die op dezelfde manier mislukt (inloggegevens geweigerd, host niet bereikbaar) wordt hetzelfde verwoord; andere fouten krijgen één zin over wat er mislukte ("De repository kon niet worden gekloond.", "Het programma kon niet worden gestart.").
- **De details**, getoond als het item openstaat (`Enter`, of de klik die het opende): de bewerking, de uitgevoerde git-opdracht, de exitcode en de laatste regels van wat het afdrukte (voor de ingebouwde git-bibliotheek de foutklasse en code). Inloggegevens, tokens en `Authorization`-headers worden gemaskeerd voordat er iets wordt getoond, gelogd of gekopieerd.
- **Kopiëren** (de knop, of `c`) zet een tekstblok op het klembord dat bedoeld is om in een gesprek te plakken met iemand, of een AI, die het kan uitleggen: de versie van GitPoulpi, het systeem, de terminal, de backend, de mislukte bewerking met haar details en de git-versie.
- **Een knop voor de volgende stap**, als die er is, naast Kopiëren (of `r`): *Eerst pull* (een push die de remote zou weigeren), *Stash en opnieuw* (lokale wijzigingen staan een checkout, merge, rebase of pull in de weg), *Opnieuw* (de remote was niet bereikbaar), *Integraties openen* (inloggegevens). Hij voert de stap uit en sluit het venster. Bij een mislukking van een andere repository dan het voorste tabblad wordt alleen *Integraties openen* aangeboden: een nieuwe poging zou in de verkeerde repository lopen.
- Zonder muis heeft het opdrachtenpalet `Activiteitenlogboek: Laatste fout openen`, `Activiteitenlogboek: Laatste fout kopiëren` en `Activiteitenlogboek: Voorgestelde actie uitvoeren` (de herstelknop van het item, voor de fout die de statusbalk toont).

Een push die de remote heeft geweigerd omdat die verder is gegaan, houdt zijn eigen dialoogvenster (eerst pull, force push met lease, annuleren); de feiten staan toch in het activiteitenlogboek.

### Het activiteitenlogboek

De knop **Activiteit** aan het rechteruiteinde van de statusbalk (een getal erop telt de fouten die je nog niet hebt bekeken), `Ctrl+N` en dan `Shift+A`, en `Weergave: Activiteitenlogboek` in het palet openen het venster. `Ctrl+N` toont een badge op elk tabblad, vak, veld, selectievakje en elke knop van het venster (en op het eerste item van de lijst); druk op het teken en het doet wat een klik erop doet. Twee tabbladen, **Git-bewerkingen** en **Applicatie**, te wisselen met `Tab`, een klik, `g` (Git-bewerkingen) of `p` (Applicatie), en daaronder een vak **Bereik** dat voor beide geldt (zie hieronder):

- **Applicatie**: de eigen logregels van het programma, die van het venster en van de achtergronddaemon, nieuwste eerst. Het vak **Niveau** (een keuzelijst: Fout, Waarschuwing, Info, Debug, Trace; `l` opent hem) bepaalt hoeveel er getoond wordt; om meer detail vragen dan er wordt vastgelegd verhoogt ook het opnameniveau (en slaat het op als `log.level`). Daaronder heeft **Tonen** één vakje per niveau (Fout, Waarschuwing, Info, Debug, Trace, elk met hoeveel regels het bevat) om de regels van dat niveau te verbergen of te tonen, bijvoorbeeld om alleen de fouten te zien, en **Alles** / **Niets** vinkt elk vakje aan of uit. De toetsen `1` tot `5` schakelen de niveaus om in die volgorde, `a` is Alles, `n` is Niets; het palet heeft `Activiteitenlogboek: niveau tonen of verbergen ...`, `Activiteitenlogboek: alle niveaus tonen` en `Activiteitenlogboek: geen niveau tonen`. Niveaus die uitgebreider zijn dan de keuze van het vak Niveau zijn grijs (er wordt niets voor vastgelegd; houd de aanwijzer erboven om te zien waarom). De vinkjes worden onthouden tot je afsluit, en alleen het tabblad Applicatie heeft ze. **Filter** (`/`) houdt de regels die je tekst bevatten. **Alles kopiëren (gefilterd)** kopieert wat getoond wordt (de vinkjes en het filter gelden beide); **Logmap openen** opent de map in je bestandsbeheerder. Een lange regel wordt nooit afgekapt: zie "Lange regels" hieronder.

  ![Het activiteitenlogboek, tabblad Applicatie: het vak Niveau, de vakjes Tonen per niveau met Alles en Niets, het filter en de logregels](../images/activity-log.png)

**Log wissen** (`x`, of het palet-commando `Activiteitenlogboek: Applicatielogboek wissen`) staat op het tabblad **Applicatie**: het vraagt eerst om bevestiging en leegt dan de logbestanden van alle bewaarde dagen en de lijst op het scherm. Het is niet ongedaan te maken en staat niet in de geschiedenis van ongedaan maken.

- **Git-bewerkingen** (het tabblad waarop het venster opent): de bewerkingen, nieuwste eerst, elk met het tijdstip waarop het liep en hoe lang het duurde (`Push - 123 ms`): de repository openen, de commit-grafiek laden, fetch, pull, push, commit, checkout, merge, rebase, stash, clone. Mislukkingen, statusmeldingen, een mislukte kloon en het Fetch all van een werkruimte staan hier ook; `Enter` of een klik opent er een (de reden, de volgende stap, de opdracht, de exitcode en de uitvoer), `c` of de knop Kopiëren kopieert hem, `r` of zijn knop voert de voorgestelde stap uit. `Tab` (of hier `Left`/`Right`) wisselt naar het tabblad Applicatie.

  ![Het activiteitenlogboek, tabblad Git-bewerkingen: elke bewerking met haar tijdstip en duur, een mislukte fetch in rood](../images/activity-log-repository.png)

- **Lange regels** (beide tabbladen): een kop of een detailregel die breder is dan het venster wordt nooit afgekapt. Onder de lijst verschijnt een horizontale schuifbalk; scroll zijwaarts met `Left` / `Right` (opnieuw toe te wijzen als `activity_log.scroll_left` / `activity_log.scroll_right`), `Shift` + het wiel of een horizontaal wiel, `Activiteitenlogboek: Naar links scrollen` / `Activiteitenlogboek: Naar rechts scrollen` in het palet, of de balk (sleep de greep, of druk op de baan om de greep onder de aanwijzer te halen). `Home` gaat terug naar het begin, en van tabblad wisselen laat het andere tabblad aan de linkerrand beginnen. Omgebroken tekst (een samenvatting, een voorgestelde actie) blijft omgebroken.

- **Bereik** (een keuzelijst onder de tabbladen, `s` opent hem, of `Activiteitenlogboek: Het bereik kiezen` in het palet): waar beide tabbladen over gaan. De rijen zijn **Deze repository (naam)** (de repository van het tabblad waaruit je het logboek opende; niet aangeboden vanaf Start, dat er geen heeft), **Alle repositories**, elke **werkruimte** en elke repository die het venster kent. Vanaf een repositorytabblad opent het logboek op *Deze repository*, vanaf Start (de knop Activiteit, `Ctrl+N` en dan `Shift+A`) op *Alle repositories*; `Ctrl+N` en dan `Shift+G` en `Weergave: Globaal activiteitenlogboek` in het palet openen altijd op *Alle repositories*, en een klik op de rode fout van een mislukking waarvan de repository niet het voorste tabblad is ook (een mislukte kloon bijvoorbeeld). Je laatste keuze wordt voor de sessie onthouden, apart voor de twee manieren om het te openen. Op het tabblad Applicatie hoort een regel bij een repository als het programma hem over die repository schreef (een mislukte push, de tijdmeting van een bewerking); onder *Alle repositories* zegt elke zo'n regel van wie hij is (`[naam]`). Een regel over geen enkele repository (opstarten, instellingen, de daemon zelf) en een item dat bij geen enkele hoort (een mislukte opslag) zijn **app-breed**: ze staan onder elk bereik, gemarkeerd met `[app-breed]` als het bereik smaller is dan *Alle repositories*, zodat een filter nooit een mislukking verbergt. Het palet heeft ook `Activiteitenlogboek: Git-bewerkingen` en `Activiteitenlogboek: Applicatielogboek` voor de tabbladen.

- **Diagnose kopiëren** (een knop op beide tabbladen, `d`, altijd het hele logboek, wat het bereik ook is; en `Weergave: Diagnose kopiëren` in het palet) kopieert de versie, het systeem, de terminal en de backend, de laatste mislukking met haar details en de laatste 200 logregels op het gekozen niveau: het blok om een AI te geven als er iets mis is.

De duur wordt door het venster gemeten (van het verzoek tot het antwoord van de daemon, heen-en-terugreis inbegrepen), dus wat de CLI of MCP doet wordt niet getimed. `log.timings` (standaard aan) zet de duur uit, `log.extended` (standaard uit) toont ook het achtergrondwerk, het periodieke vernieuwen en auto-fetch; beide staan in Instellingen > Algemeen naast de andere loginstellingen, en elke bewerking wordt hoe dan ook naar het applicatielogboek geschreven (`info` voor wat je vroeg, `debug` voor de achtergrond).

Logbestanden heten `gitpoulpi-YYYY-MM-DD.log`, worden standaard een week bewaard en hooguit tien jaar (`log.retention_days`, 1 tot 3650), opgeruimd wanneer GitPoulpi start, en geroteerd bij `log.max_file_mb` (5 MiB). Waar ze staan: [configuratienaslag](config-reference.md#log-het-applicatielogboek). Vanuit een script draagt de `--json`-fout van een mislukte opdracht dezelfde feiten onder `detail`; zie de [CLI-naslag](cli-reference.md).

Een klik op het rode bericht opent het logboek op *Alle repositories* met dat item open. De lijst met Git-bewerkingen is die van de sessie; het tabblad Applicatie bewaart het logboek van het programma over herstarts heen.

**Achtergrondtaken.** Een fetch, push, pull, kloon of LFS-overdracht, het Fetch all / Pull all / Clone missing van een werkruimte, een AI-verzoek en een pull request of issue dat wordt verstuurd zijn taken: het rechteruiteinde van de statusbalk toont de lopende met een `[x]` om te annuleren, en een klik op het label (of `Ctrl+N` en dan `Shift+J`, de paletopdracht `Taken: achtergrondtaken tonen`) opent het paneel met een knop Annuleren per taak (ook `c` of `Del`). Een kloon draait op de achtergrond; een AI-verzoek en een werkruimterondgang overleven een tabwissel. Een taak voorbij het punt waarop stoppen niet veilig meer is, is grijs met de reden; een geannuleerde taak laat de repository zoals ze was.

## Snelle antwoorden

**De werkbalk en de zijbalk tonen vakjes of vraagtekens in plaats van pictogrammen.**
Gebruik in je terminal een Nerd Font of een ander lettertype met emoji-dekking.
De grafiek en de tekst werken ook zonder.

**De commit-grafiek bestaat uit tekens, niet uit afbeeldingen.** Dat is de
automatische terugval: afbeeldingen vragen een terminal die het kitty-grafiekprotocol
spreekt (kitty, Ghostty, herdr met kitty graphics aan) of het iTerm2-protocol
(WezTerm, iTerm2, mintty) of sixel (Windows Terminal, foot, Konsole). De Windows-console, tmux en zellij krijgen tekens.
Zie de sectie over de grafiek hieronder om het ene of het andere af te dwingen, en [Een terminal kiezen](terminals.md).

**De muis doet niets, of tekst selecteren met de muis werkt niet.**
GitPoulpi vangt de muis op, zodat klikken bij het programma aankomen. Om tekst met
de terminal zelf te selecteren houd je `Shift` ingedrukt terwijl je sleept (in de
meeste terminals). Je kunt ook in de diff selecteren (sleep en druk dan op `y`).
Om het opvangen van de muis uit te zetten gebruik je *Instellingen > Toetsenbord &
muis > Muis ingeschakeld* (herstart nodig). Het toetsenbord bereikt elke opdracht.

**Een sneltoets werkt niet.** Je terminal, tmux of een AutoHotkey-script kan de
toets afvangen voordat GitPoulpi hem ziet (`Ctrl+,` komt in terminals op Linux en
macOS vaak aan als een gewone `,`; `Ctrl+Shift+N` lijkt op `Ctrl+N`). Gebruik de
alternatieven (`Ctrl+N` en dan `,` opent de Instellingen; `Ctrl+P` bereikt elke
opdracht) of wijs de toetscombinatie opnieuw toe: zie
[Instellingen](settings.md#toetsenbord--muis).

**Ik weet een sneltoets niet meer.** Druk op `Ctrl+P` en typ wat je wilt; elk
resultaat toont zijn sneltoets. `?` toont alles.

**Een optie is grijs in de Instellingen.** Ze hangt af van een andere optie. De rij
zegt van welke (bijvoorbeeld: *Sterkte van rijtint* heeft *Achtergrond van
grafiekbanen* op *Rijtint* nodig).

**"Auto-stash openstaand" boven de Log.** GitPoulpi heeft je wijzigingen gestasht om
een bewerking uit te voeren en kon ze niet terugzetten. Gebruik **Nu poppen**,
**Stash tonen**, **Opnieuw beginnen** (de bewerking ongedaan maken en je
wijzigingen terugzetten) of **Drop** van de banner; zie
[Auto-stash](log-and-history.md#auto-stash).

**Een agent meldt `no_viewer` of `deferred`.** `no_viewer`: er is geen
GitPoulpi-venster open voor die repository (of *AI-agents de weergave laten
besturen* staat uit). `deferred`: het venster toont een chip; `Ctrl+N Enter`
accepteert hem. Zie [MCP en agents](mcp-and-agents.md#dingen-aan-de-gebruiker-tonen).

**Ik heb GitPoulpi geopend en zie een leeg venster of `log error: backend error`.**
Controleer of je het in een Git-repository hebt gestart (anders kom je op het
tabblad Start: gebruik **Bladeren**, **Klonen** of **Init**). Voer dan
`gitpoulpi daemon status` uit; `gitpoulpi daemon restart` herstart het
achtergrondproces van deze repository. Druk op `r` om te vernieuwen.

**Twee vensters op dezelfde repository gedragen zich vreemd.** Ze delen met opzet
één daemon per repository, zodat een wijziging in het ene in het andere verschijnt.
Lijkt een daemon vast te zitten, dan stopt `gitpoulpi daemon stop --all` de daemon
van elke sessie; het volgende venster start een verse.

**Een commit, pull of push vraagt om een wachtwoord of authenticeert niet.**
GitPoulpi gebruikt voor het netwerk de eigen credential helper en ssh-agent van je
`git`. Zorg dat `git fetch` eerst in een gewone terminal werkt. De tokens van
GitHub/GitLab voor pull requests staan los daarvan en zijn optioneel: zie
[Hosting en issuetrackers](hosting-integration.md).

**"stagen..." blijft in de statusbalk staan.** Een vernieuwing op de achtergrond loopt
nog; hij verdwijnt als de bestandslijst wordt bijgewerkt. `r` forceert een
vernieuwing.

**Een zeer grote repository is traag.** Verlaag *Eerste commits in de grafiek* op de
pagina Instellingen > Log & grafiek (de Log laadt meer terwijl je scrolt als *Commits
geleidelijk laden in de grafiek* aan staat); verberg de kolom *Wijzigingen* van de Log en zet
avatars uit; zet *Ongedaan maken ingeschakeld* uit als je geen Ongedaan maken nodig hebt (dan
is er helemaal geen administratie bij staging en commits). `GITPOULPI_TIMING=1`
schrijft timings per verzoek naar `gitpoulpi-timing.log` in de tijdelijke map; handig
om bij een prestatiemelding te voegen.

**De AI-knoppen ontbreken of zijn grijs.** AI staat standaard uit; zie
[AI-ondersteuning](ai.md#probleemoplossing).

**GitPoulpi sloot uit zichzelf.** Elke afsluiting wordt vastgelegd: zoek
`gitpoulpi-tui-exit.log` (en `gitpoulpi-tui-panic.txt` na een crash) in de
configuratiemap (`gitpoulpi config path` toont het instellingenbestand, dat daar
staat) en voeg het bij een issue.

**Waar staan mijn instellingen?** `gitpoulpi config path`. De status per repository
staat in `.gitpoulpi/repo-state.toml` in de repository.

**Stuurt GitPoulpi iets ergens naartoe?** Alleen wat je zelf vraagt: git-bewerkingen
naar je remotes, hostingverzoeken nadat je een account verbindt, AI-verzoeken nadat
je op een AI-knop drukt en, als je avatars aanzet, een hash van e-mailadressen van
auteurs naar Gravatar. Er is geen telemetrie. Zie [AI-ondersteuning](ai.md#privacy).

## Elke schrijfactie mislukt: "Unable to create index.lock"

Een git-opdracht die is gestopt (een crash, een afgebroken terminal) kan
`.git/index.lock` achterlaten, waarna elke wijziging aan de repository mislukt. Zodra
het bestand ouder is dan 10 seconden toont GitPoulpi een strook boven de Log:
**index.lock verwijderen** (palet: *Repository: Verouderde index.lock verwijderen*; CLI:
`gitpoulpi index-lock remove`). Hij vraagt eerst bevestiging en verwijdert alleen dat
bestand. GitPoulpi kan niet betrouwbaar vaststellen of een ander git-programma nog
draait; zorg dus dat dat niet zo is (een IDE, een andere terminal) voordat je
bevestigt. In een gekoppelde worktree is de lock die van de worktree zelf, in
`.git/worktrees/<naam>`.

## "git am" is gestopt en Rebase afbreken doet niets

Een `am` (patches toepassen uit een mailbox) is geen rebase. Een gestopte `git am`
heeft een eigen strook, **Patches toepassen (git am)**, met Conflicten oplossen,
Doorgaan, Overslaan en Afbreken (palet: *git am: Doorgaan / Deze patch overslaan / Afbreken*; CLI:
`conflicts am-continue|am-skip|am-abort`). Afbreken vraagt eerst bevestiging, laat de
tot dan toe toegepaste patches vallen en zet de branch terug waar hij was. Deze
acties zijn niet ongedaan te maken, net als de rebase-knoppen.

## Ongedaan maken is grijs en zegt "Kan ... niet ongedaan maken" (geblokkeerd item)

Het nieuwste journaalitem zit vast: de repository is buiten GitPoulpi om verder
gegaan, of het item is onderbroken. Houd de muis boven Ongedaan voor de reden;
zit het definitief vast, dan wijst de tooltip naar **Bewerken: Geblokkeerd ongedaan-maken-item vergeten**
(palet) of `gitpoulpi undo-forget`. Hij vraagt eerst bevestiging en haalt alleen het
item uit het journaal; de repository blijft onaangeroerd. Zie
[Ongedaan maken en opnieuw](undo-redo.md).

## Grote bestanden verschijnen als drie regels tekst ("LFS files show as pointers")

Een bestand dat een afbeelding, een model of een grote binary zou moeten zijn bevat
`version https://git-lfs.github.com/spec/v1` en een regel `oid sha256:...`: het is
een Git LFS-**pointer**, en de echte inhoud is niet gedownload (of de LFS-filters zijn
niet geïnstalleerd).

- Kijk in het paneel Status: een repository die LFS gebruikt terwijl `git-lfs` ontbreekt, toont een oranje banner. Installeer het programma (https://git-lfs.com; Windows: meegeleverd met Git for Windows of `winget install GitHub.GitLFS`; Debian/Ubuntu: `apt install git-lfs`; macOS: `brew install git-lfs`) en heropen of druk op `r`.
- `gitpoulpi lfs status` zegt of het programma is geïnstalleerd, of de filters en de hook zijn ingesteld en welke patronen worden gevolgd; `gitpoulpi lfs files` somt de pointerbestanden op en welke `local` zijn.
- Met `git-lfs` geïnstalleerd voer je **`LFS: Objecten pullen`** uit in het palet (of `gitpoulpi lfs pull`): het downloadt de objecten en vervangt de pointers. Het installeert eerst de LFS-filters voor deze repository als die er niet zijn.
- Mislukt de download, dan noemt de melding de reden (inloggegevens voor de remote, een server die het object niet meer heeft). GitPoulpi heeft geen eigen account nodig: LFS gebruikt de gebruikelijke inloggegevens van je remote (credential helper of ssh-agent).
- Een clone van een LFS-repository mislukt nooit door LFS; kon de downloadstap niet draaien, dan staat de clone er met pointers en maakt `LFS: Objecten pullen` hem later af. `gitpoulpi clone --no-lfs` slaat de download bewust over.
- Je kunt alles rond LFS uitzetten in Instellingen > Git & codering > Git LFS (of `[git] lfs_enabled = false`).

## "git-lfs not found" / "Git LFS is not installed"

De melding `Git LFS is not installed. This repository stores large files with Git LFS; ...` (of een dialoogvenster met een knop *Installatiepagina openen*, of de eigen `git-lfs: command not found` / `'lfs' is not a git command` van git) betekent dat de repository voor LFS is ingesteld - zijn `.gitattributes`, de filters in `.git/config` of de `pre-push`-hook roepen `git-lfs` aan - en dat het programma niet op het `PATH` staat dat GitPoulpi ziet. Installeer het (link hierboven). Op Windows wordt het programma meegeleverd met Git for Windows; werkt `git lfs version` in een terminal maar zegt GitPoulpi nog steeds dat het ontbreekt, start GitPoulpi dan vanuit een terminal die hetzelfde `PATH` ziet. Om LFS voor GitPoulpi te verbergen zonder het eigen gedrag van git aan te raken, zet je `git.lfs_enabled` uit.

## De grafiek wordt met tekens getekend, niet als afbeelding (herdr, tmux, andere terminals)

GitPoulpi tekent de kolom Grafiek en de avatars van auteurs alleen als pixels waar het zeker weet dat de terminal ze toont: iTerm2 inline-afbeeldingen in kaal WezTerm, iTerm2 en mintty; het Kitty-grafiekprotocol in kitty, Ghostty en **herdr wanneer de eigen configuratie van herdr `kitty_graphics = true` zet** (`[terminal]`, of `[experimental]` in oudere herdr) en herdr draait binnen WezTerm, kitty of Ghostty. Sixel-afbeeldingen worden gebruikt in Windows Terminal, foot, Konsole en mlterm. Overal elders tekent het tekens, wat altijd klopt.

- In herdr met Kitty graphics aan maar een configuratie die GitPoulpi niet vindt (of als je op de standaard van herdr vertrouwt), stel je `ui.image_protocol = "kitty"` in (Instellingen > Log & grafiek > Afbeeldingsprotocol) en herstart je.
- In een terminal die vakjes of rommel toont waar de grafiek hoort, stel je `ui.image_protocol = "none"` of `ui.graph_renderer = "text"` in.
- `GITPOULPI_TERMINAL_IMAGE=off` zet afbeeldingen uit, ongeacht de configuratie.

**herdr remote (`herdr --remote <host>`, of `ssh` en dan `herdr`): de tekstgrafiek is de standaard.** Via SSH draait GitPoulpi op de remote host en kan het alleen de omgeving en de herdr-`config.toml` van de remote lezen. Of er een Kitty-afbeelding verschijnt, wordt bepaald door de *lokale* herdr-client (zijn eigen instelling `kitty_graphics` en zijn eigen terminal), die GitPoulpi niet kan zien: tekent de client de afbeeldingen niet, dan blijven de cellen leeg. GitPoulpi tekent daarom de tekstgrafiek zodra het herdr over SSH detecteert (`SSH_CONNECTION` / `SSH_CLIENT` / `SSH_TTY`). Heeft je lokale herdr `kitty_graphics = true` en een terminal die Kitty-afbeeldingen tekent (kitty, Ghostty), kies er dan voor met `ui.graph_renderer = "image"` (of `ui.image_protocol = "kitty"`) op de remote. Afbeeldingen worden inline verstuurd (`t=d`), wat over elke bytestroom werkt. Is de kolom leeg, stel dan `ui.graph_renderer = "text"` in.

**Een lichtere rechthoek achter de banen.** De afbeelding is transparant (straight-alpha RGBA-PNG) overal waar niets getekend wordt, zodat de achtergrond van de terminal en de selectiebalk doorschemeren. Terminals met het Kitty-protocol en **WezTerm** combineren die alfa; GitPoulpi reserveert dan gewone lege cellen onder de afbeelding en verstuurt hem erna. Van elke andere terminal met het iTerm2-protocol (iTerm2 zelf, Rio, mintty) is niet bekend dat hij PNG-alfa blendt, dus daar draagt de afbeelding een ondoorzichtige achtergrond in de themakleur, die als rechthoek kan opvallen op een terminal waarvan de achtergrond anders is; `ui.graph_renderer = "text"` haalt hem weg. Met `ui.graph_lane_background = "off"` wordt op de transparante paden niets achter de banen geschilderd.

Zie `ui.graph_renderer` en `ui.image_protocol` in de [configuratiereferentie](config-reference.md).
