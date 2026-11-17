[English](../updates.md) · [Čeština](../cs/updates.md) · [Deutsch](../de/updates.md) · [Español](../es/updates.md) · [Français](../fr/updates.md) · [Italiano](../it/updates.md) · [日本語](../ja/updates.md) · [한국어](../ko/updates.md) · **Nederlands** · [Polski](../pl/updates.md) · [Português (Brasil)](../pt-BR/updates.md) · [Português (Portugal)](../pt-PT/updates.md) · [Русский](../ru/updates.md) · [Svenska](../sv/updates.md) · [Türkçe](../tr/updates.md) · [Українська](../uk/updates.md) · [简体中文](../zh-CN/updates.md)

# Updates

GitPoulpi controleert of er nieuwere releases zijn en vervangt, afhankelijk van hoe het is geïnstalleerd, ofwel zijn eigen bestanden, ofwel geeft het de update door aan het programma dat het heeft geïnstalleerd. Er wordt nooit iets vervangen of herstart zonder jouw actie.

## De chip en het paneel

De titelbalk toont de draaiende versie (`v1.0.0`) tussen de appnaam en het pad van de repository. Zodra een nieuwere release bekend is, verandert de chip in `v0.2.0 beschikbaar` in de accentkleur.

Druk op `Ctrl+N` en dan `u`, voer **Updates: Updatepaneel tonen** uit in het opdrachtpalet (`Ctrl+P`), of klik op de chip terwijl die `beschikbaar` meldt (of terwijl een update bezig is): het updatepaneel opent. Zonder nieuwere release opent een klik op de chip in plaats daarvan het tabblad Releasenotes. Het is een venster boven de app: `Esc`, `q`, het sluitkruisje of een klik buiten het venster sluit het. Het toont:

- de draaiende versie en **Geïnstalleerd via**: hoe GitPoulpi op deze computer is gekomen (dat bepaalt alles hieronder);
- hoe de update ervoor staat (controleren, downloaden met een percentage, klaar, mislukt);
- de knoppen van de procedure van deze installatie, en de exacte opdracht als die er is;
- een regel van je beheerder, als die er een heeft ingesteld.

Elke knop heeft een toets (afgedrukt in de hintregel van het paneel) en een palette-opdracht, dus het paneel werkt zonder muis:

| Knop | Toets | Palette-opdracht |
|---|---|---|
| **Bijwerken en herstarten** (of **Update uitvoeren**) | `u` | Updates: Nu bijwerken |
| **Opdracht kopiëren** | `c` | Updates: Updateopdracht kopiëren |
| **In terminal openen** | `t` | Updates: Updateopdracht in een terminal uitvoeren |
| **Terug naar ...** | `v` | Updates: Terug naar de vorige versie |
| **Releasenotes** | `n` | Help: Releasenotes |
| **Deze versie overslaan** | `s` | Updates: Deze versie overslaan |
| **Melding openen** (alleen bij een beveiligingsmelding) | `a` | Updates: Beveiligingsmelding openen |
| **Nu controleren** | `r` | Updates: Nu op updates controleren |
| **Sluiten** | `Esc` | |

Een opdracht die niet van toepassing is op deze installatie is in het palet grijs weergegeven, met de reden (bijvoorbeeld "deze installatie wordt op een andere manier bijgewerkt: zie Updates: Updatepaneel tonen").

## Wat er gebeurt, per installatie

| Geïnstalleerd via | Controle bij | Wat het paneel doet | Teruggaan |
|---|---|---|---|
| Een releasearchief, of `install.sh` | GitHub-releases | **Bijwerken en herstarten**: GitPoulpi downloadt en verifieert het archief, sluit, vervangt zijn bestanden en opent opnieuw in dezelfde terminal. | **Terug** zet de vorige bestanden terug. |
| Het Windows-installatieprogramma, alleen voor mij | GitHub-releases | Hetzelfde, zonder vraag. Op Windows opent het opnieuw in een nieuw venster. | **Terug** zet ze terug. |
| Het Windows-installatieprogramma, alle gebruikers | GitHub-releases | Hetzelfde, maar Windows vraagt om goedkeuring van een beheerder. GitPoulpi-sessies van andere gebruikers op de computer worden gesloten, waardoor hun niet-opgeslagen invoer verloren gaat. Als je weigert, verandert er niets. | **Terug** zet ze terug, met dezelfde vraag. |
| Chocolatey | De Chocolatey-communityfeed | **Update uitvoeren** voert `choco upgrade gitpoulpi -y` uit (Windows vraagt om goedkeuring van een beheerder). | Het paneel toont `choco install gitpoulpi --version <vorige> --allow-downgrade -y`. |
| Chocolatey, interne feed | De feed van je organisatie | **Update uitvoeren** voert `choco upgrade gitpoulpi -y --source=<feed>` uit. Pakketparameters worden niet opnieuw meegestuurd, dus een uitgerolde configuratie blijft onaangeroerd. | Dezelfde `choco install`-regel met de feed. |
| Scoop | GitHub-releases | **Update uitvoeren** voert `scoop update gitpoulpi` uit nadat GitPoulpi volledig is gesloten. | **Terug** voert `scoop reset gitpoulpi@<vorige>` uit. |
| Homebrew | GitHub-releases | **Update uitvoeren** voert `brew upgrade gitpoulpi` uit in deze terminal. | Het paneel toont `brew install gitpoulpi@<vorige>`. |
| apt | GitHub-releases | Het paneel toont `sudo apt update && sudo apt install --only-upgrade gitpoulpi` met **Opdracht kopiëren** en **In terminal openen**. GitPoulpi voert nooit `sudo` voor je uit. Zonder de apt-repository van GitPoulpi op deze computer staat er dat je in plaats daarvan de nieuwe `.deb` moet downloaden. | Het paneel toont `sudo apt install gitpoulpi=<vorige>`. |
| Een MSI-pakket | GitHub-releases, of de bron van je beheerder | Niets uit te voeren: de installatie wordt beheerd door je beheerder, wiens bericht het paneel toont. | Niet beschikbaar. |
| cargo | GitHub-releases | Het paneel toont `cargo binstall gitpoulpi-cli` als `cargo-binstall` is geïnstalleerd, anders `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<versie> gitpoulpi-cli`. | Dezelfde opdracht met de vorige tag. |
| Een Docker-image | GitHub-releases | Niets uit te voeren: een image wordt vervangen. Het paneel toont `docker pull poulpilab/gitpoulpi:<versie>` en `:latest`. Zie [GitPoulpi in Docker draaien](docker.md). | Het paneel toont `docker pull poulpilab/gitpoulpi:<vorige>`. |
| Een releasearchief in een map waar je niet naar kunt schrijven | GitHub-releases | Het paneel zegt dat GitPoulpi zichzelf daar niet kan vervangen en wijst naar **Downloaden**. | Niet beschikbaar. |
| Een ontwikkelbuild (`cargo build`) | geen | Werkt nooit bij. De chip toont de versie en verder niets; het paneel zegt dat er geen installatiegegevens zijn. | Niet beschikbaar. |

Rijen die alleen iets "tonen" voeren zelf nooit iets uit. **In terminal openen** opent je terminal met de opdracht al ingetypt, en jij drukt op Enter.

Releases komen altijd uit de GitHub-repository van het project, tenzij een beheerder de controle ergens anders heen wijst (een interne feed, een mirror of een `latest.json`), zie [Distributie voor organisaties](enterprise-distribution.md). Releasenotes komen uit de release of uit de changelog die in GitPoulpi is ingebouwd, wat bereikbaar is.

## Melden en auto

`updates.mode` ([Instellingen > Synchronisatie](settings.md#synchronisatie)):

- **Melden** (standaard): eenmaal per dag controleert GitPoulpi op de achtergrond en toont de chip. Er wordt niets gedownload tot je op de knop drukt.
- **Auto**: waar GitPoulpi zijn eigen bestanden vervangt (releasearchief, het Windows-installatieprogramma) downloadt en verifieert het de update ook op de achtergrond zodra het er een vindt. De chip toont dan **Herstarten om bij te werken**, en het herstarten kies je nog steeds zelf. Waar een pakketbeheerder de update beheert, gedraagt **Auto** zich als **Melden**.
- **Uit**: er wordt nooit een verzoek gedaan en de chip toont alleen de versie.

De instelling **Updatekanaal** (`updates.channel`) kiest tussen **Stabiel** en **Pre-release** (ook release candidates). Een controle die je met **Nu controleren** start, negeert de limiet van eenmaal per dag. Zonder netwerkverbinding, of zolang de repository privé is en er geen github.com-token is opgeslagen, mislukt een controle op de achtergrond stilletjes met één regel in het toepassingslogboek.

## Herstarten en wat je daarna ziet

Op **Bijwerken en herstarten** of **Update uitvoeren** drukken bewaart je sessie en sluit het venster. Een apart klein programma, `gitpoulpi-updater`, doet het werk, zodat de eigen bestanden van GitPoulpi niet in gebruik zijn terwijl ze worden vervangen en een pakketbeheerder niet wordt geblokkeerd door een wachtende bovenliggende app. Het stopt eerst de achtergronddaemons van GitPoulpi (de andere vensters en MCP-servers van je gebruiker sluiten ook), voert de update uit, controleert of de nieuwe `gitpoulpi --version` de verwachte versie afdrukt en start GitPoulpi opnieuw: in dezelfde terminal op Linux en macOS, in een nieuw venster op Windows. Mislukt de controle, dan zet het de oude bestanden terug.

De volgende start toont één regel in de statusbalk:

- `Bijgewerkt naar v1.4.2`
- `Bijwerken mislukt: <reden>. De vorige versie is nog geïnstalleerd.`
- `Bijwerken geannuleerd: de goedkeuring van de beheerder is geweigerd. Er is niets gewijzigd.`

Er draait maar één update tegelijk; een lockbestand in de gegevensmap van GitPoulpi (en een in de installatiemap als die beschrijfbaar is) houdt een tweede buiten, en een lock die ouder is dan tien minuten wordt genegeerd, zodat een vastgelopen update GitPoulpi nooit definitief blokkeert. De CLI en de daemon starten niet zolang er een verse lock bestaat.

## Terug naar de vorige versie

Waar GitPoulpi zijn eigen bestanden vervangt, zet **Terug naar ...** de vorige bestanden terug en herstart. Ze blijven nog een minuut na de start van de nieuwe versie als `<naam>.old` staan en worden dan verwijderd, dus teruggaan is bedoeld voor de eerste keer draaien na een update. Waar een pakketbeheerder de installatie beheert, is teruggaan de opdracht uit de tabel: het paneel toont of voert die uit. Teruggaan wijzigt de eigen bestanden van GitPoulpi, geen repository, en maakt dus geen deel uit van het git-ongedaanmakingsjournaal.

## Wat wordt gecontroleerd en geweigerd

Voordat iets wordt vervangen, downloadt GitPoulpi het archief, zijn `.minisig`-handtekening en zijn `.sha256`. Het controleert eerst de handtekening, tegen de releasesleutel van het project die in GitPoulpi is ingebouwd, en de vertrouwde opmerking van de handtekening moet dit archief en deze versie noemen. Daarna controleert het de hash. Het weigert: een release zonder `.minisig` (een oudere release die is gepubliceerd voordat releases werden ondertekend, kan niet via de updater worden geïnstalleerd, ook niet door te pinnen of terug te gaan), een handtekening van een andere sleutel, een handtekening voor een ander bestand of een andere versie, een release zonder `.sha256`, een hash die niet overeenkomt, een archief waarvan de naam niet die voor dit platform is (`gitpoulpi-<versie>-<target>.<zip|tar.gz>`), een archief waarvan de markering een andere versie of een ander platform noemt, en een downloadlink buiten de releasehosts van GitHub (elke redirect-stap wordt gecontroleerd) of de mirror die een beheerder heeft geconfigureerd. De vervanging is alles of niets: als een ander programma een van de uitvoerbare bestanden vasthoudt, noemt GitPoulpi het bestand, wijzigt niets en laat de download staan voor een nieuwe poging. Op Windows wordt een update voor alle gebruikers opnieuw geverifieerd door de verhoogde stap, die de `.minisig` en de `.sha256` zelf downloadt en beide opnieuw controleert, zodat een bestand dat ondertussen in de downloadmap van de gebruiker is verwisseld, wordt geweigerd. Pakketbeheerders (Chocolatey, Scoop, Homebrew, apt) verifiëren hun eigen pakketten: GitPoulpi downloadt daar niets.

## Beveiligingsmeldingen

Naast elke release publiceert het project `security.json`: de lijst met beveiligingsmeldingen (een identificatie, een ernst, de getroffen versies als bereik zoals `>=1.2.0, <1.4.3`, de opgeloste versie, een link en een samenvatting van één zin). Dezelfde dagelijkse controle die naar een nieuwe versie zoekt, haalt het ook op, van dezelfde plek als de release: de releasepagina van de repository voor GitHub, de map van `latest.json` voor een mirror, GitHub voor de openbare Chocolatey-feed. Een melding wordt alleen gebruikt als `security.json.minisig` klopt met de sleutel van het project; een niet-ondertekend of slecht ondertekend bestand wordt nooit getoond (één regel in het activiteitenlogboek zegt dat het is genegeerd).

Als een melding de draaiende versie noemt, wordt de chip rood met de tekst **Beveiligingsmelding: deze versie is getroffen**, op elke installatie, ook die door een pakketbeheerder worden beheerd. Het updatepaneel toont dan elke melding met ernst, samenvatting, opgeloste versie en link; **Melding openen** (`a`, of **Updates: Beveiligingsmelding openen** in het palet) opent de link van de ernstigste in de browser. Waar een pakketbeheerder de installatie beheert, voegt het paneel "Werk bij met je pakketbeheerder." toe.

Met updatecontroles uit (`updates.mode = "off"`), met een beheerdersbeleid waarvan `check` `none` is, en voor een build gemaakt met `cargo build`, wordt niets opgehaald en geen melding getoond. De eigen NuGet-feed van een organisatie laat GitPoulpi geen contact met GitHub opnemen: de meldingen komen dan alleen van de `download_base`-mirror van het beleid.

## Instellingen die een beheerder heeft vergrendeld

Een beheerder kan in een beleid voor de hele machine de updatemodus, het kanaal, een vastgepinde versie, waar gecontroleerd wordt en of GitPoulpi zichzelf überhaupt mag bijwerken instellen. In de Instellingen tonen die rijen **Ingesteld door je beheerder**, het paneel toont hun bericht, en als zelf bijwerken is uitgezet, biedt het paneel de opdrachten of een opmerking aan in plaats van knoppen die bestanden vervangen. Onleesbare beleidsbestanden falen veilig: alleen melden, geen zelf bijwerken. De kant van de beheerder staat in [Distributie voor organisaties](enterprise-distribution.md#het-beleidsbestand-en-het-register).

## Omgeving

`GITPOULPI_UPDATE_API` wijst de controle naar een andere API-root (gebruikt door de tests).
