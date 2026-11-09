[English](../updates.md) · [Čeština](../cs/updates.md) · [Deutsch](../de/updates.md) · [Español](../es/updates.md) · [Français](../fr/updates.md) · [Italiano](../it/updates.md) · [日本語](../ja/updates.md) · [한국어](../ko/updates.md) · [Nederlands](../nl/updates.md) · [Polski](../pl/updates.md) · [Português (Brasil)](../pt-BR/updates.md) · [Português (Portugal)](../pt-PT/updates.md) · [Русский](../ru/updates.md) · **Svenska** · [Türkçe](../tr/updates.md) · [Українська](../uk/updates.md) · [简体中文](../zh-CN/updates.md)

# Uppdateringar

GitPoulpi söker efter nyare releaser och, beroende på hur det installerades, ersätter antingen sina egna filer eller lämnar uppdateringen till programmet som installerade det. Ingenting ersätts eller startas om utan att du gör något.

## Chipet och panelen

Titelraden visar den körande versionen (`v1.0.0`) mellan appnamnet och repositoryts sökväg. När en nyare release är känd blir chipet `v0.2.0 tillgänglig` i accentfärgen.

Tryck `Ctrl+N` och sedan `u`, kör **Uppdateringar: Visa uppdatering** från kommandopaletten (`Ctrl+P`), eller klicka på chipet medan det står `tillgänglig` (eller medan en uppdatering pågår): uppdateringspanelen öppnas. Utan någon nyare release öppnar ett klick på chipet fliken Versionsanteckningar i stället. Den är ett fönster över appen: `Esc`, `q`, stängningskorset eller ett klick utanför stänger den. Den visar:

- den körande versionen och **Installerat med**: hur GitPoulpi kom till den här datorn (det avgör allt nedan);
- hur uppdateringen står sig (söker, laddar ned med en procentsats, redo, misslyckad);
- knapparna för den här installationens procedur, och det exakta kommandot när det finns ett;
- en rad från din administratör när de har satt en.

Varje knapp har en tangent (utskriven på panelens tipsrad) och ett palettkommando, så panelen fungerar utan mus:

| Knapp | Tangent | Palettkommando |
|---|---|---|
| **Uppdatera och starta om** (eller **Kör uppdatering**) | `u` | Uppdateringar: Uppdatera nu |
| **Kopiera kommando** | `c` | Uppdateringar: Kopiera uppdateringskommando |
| **Öppna i terminal** | `t` | Uppdateringar: Kör uppdateringskommando i en terminal |
| **Gå tillbaka** | `v` | Uppdateringar: Gå tillbaka till föregående version |
| **Versionsanteckningar** | `n` | Hjälp: Versionsanteckningar |
| **Hoppa över den här versionen** | `s` | Uppdateringar: Hoppa över den här versionen |
| **Öppna meddelande** (bara med ett säkerhetsmeddelande) | `a` | Uppdateringar: Öppna säkerhetsmeddelande |
| **Sök nu** | `r` | Uppdateringar: Sök efter uppdateringar nu |
| **Stäng** | `Esc` | |

Ett kommando som inte gäller för den här installationen är gråat i paletten med orsaken (till exempel "den här installationen uppdateras på annat sätt: se Uppdateringar: Visa uppdatering").

## Vad som händer, per installation

| Installerat med | Söker mot | Vad panelen gör | Gå tillbaka |
|---|---|---|---|
| Ett releasearkiv, eller `install.sh` | GitHub-releaser | **Uppdatera och starta om**: GitPoulpi laddar ned och verifierar arkivet, stängs, ersätter sina filer och öppnas igen i samma terminal. | **Gå tillbaka** byter tillbaka de tidigare filerna. |
| Windows-installationsprogrammet, bara för mig | GitHub-releaser | Samma sak, utan fråga. I Windows öppnas det igen i ett nytt fönster. | **Gå tillbaka** byter tillbaka. |
| Windows-installationsprogrammet, alla användare | GitHub-releaser | Samma sak, men Windows ber om administratörsgodkännande. Andra användares GitPoulpi-sessioner på datorn stängs, vilket förlorar deras osparade inmatning. Om du nekar ändras ingenting. | **Gå tillbaka** byter tillbaka, med samma fråga. |
| Chocolatey | Chocolateys communityfeed | **Kör uppdatering** kör `choco upgrade gitpoulpi -y` (Windows ber om administratörsgodkännande). | Panelen visar `choco install gitpoulpi --version <previous> --allow-downgrade -y`. |
| Chocolatey, intern feed | Organisationens feed | **Kör uppdatering** kör `choco upgrade gitpoulpi -y --source=<feed>`. Paketparametrar skickas inte igen, så en driftsatt konfiguration lämnas orörd. | Samma `choco install`-rad med feeden. |
| winget | GitHub-releaser | **Kör uppdatering** kör `winget upgrade --id GitPoulpi.GitPoulpi --exact --silent` med avtalen godkända. winget ber själv om administratörsgodkännande när installationen gäller hela datorn. | Panelen visar `winget install --id GitPoulpi.GitPoulpi --version <previous>`. |
| Scoop | GitHub-releaser | **Kör uppdatering** kör `scoop update gitpoulpi` när GitPoulpi har stängts helt. | **Gå tillbaka** kör `scoop reset gitpoulpi@<previous>`. |
| Homebrew | GitHub-releaser | **Kör uppdatering** kör `brew upgrade gitpoulpi` i den här terminalen. | Panelen visar `brew install gitpoulpi@<previous>`. |
| apt | GitHub-releaser | Panelen visar `sudo apt update && sudo apt install --only-upgrade gitpoulpi` med **Kopiera kommando** och **Öppna i terminal**. GitPoulpi kör aldrig `sudo` åt dig. Utan GitPoulpis apt-repository på den här datorn står det att du ska ladda ned den nya `.deb` i stället. | Panelen visar `sudo apt install gitpoulpi=<previous>`. |
| Ett MSI-paket | GitHub-releaser, eller din administratörs källa | Inget att köra: installationen hanteras av din administratör, vars meddelande panelen visar. | Erbjuds inte. |
| AUR | GitHub-releaser | Panelen visar `yay -S gitpoulpi-bin` (eller `paru`, vilket den hittar först; utan en hjälpare stegen `git clone` och `makepkg -si`). | Erbjuds inte. |
| cargo | GitHub-releaser | Panelen visar `cargo binstall gitpoulpi-cli` när `cargo-binstall` är installerat, annars `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli`. | Samma kommando med den tidigare taggen. |
| En Docker-avbild | GitHub-releaser | Inget att köra: en avbild ersätts. Panelen visar `docker pull poulpilab/gitpoulpi:<version>` och `:latest`. Se [Köra i Docker](docker.md). | Panelen visar `docker pull poulpilab/gitpoulpi:<previous>`. |
| Ett releasearkiv i en mapp du inte kan skriva i | GitHub-releaser | Panelen säger att GitPoulpi inte kan ersätta sig självt där och pekar till **Ladda ned**. | Erbjuds inte. |
| Ett utvecklingsbygge (`cargo build`) | ingen | Uppdateras aldrig. Chipet visar versionen och inget mer; panelen säger att det inte finns någon installationspost. | Erbjuds inte. |

Raderna som bara visar ett kommando kör aldrig något själva. **Öppna i terminal** öppnar din terminal med kommandot inskrivet, och du trycker Enter.

Releaser kommer alltid från projektets GitHub-repository om inte en administratör pekar sökningen någon annanstans (en intern feed, en spegel eller en `latest.json`), se [Företagsdistribution](enterprise-distribution.md). Versionsanteckningarna kommer från releasen eller från den ändringslogg som finns inbyggd i GitPoulpi, vilken som nås.

## Meddela och Auto

`updates.mode` ([Inställningar > Synkronisering](settings.md#synkronisering)):

- **Meddela** (standard): en gång per dag söker GitPoulpi i bakgrunden och visar chipet. Ingenting laddas ned förrän du trycker på knappen.
- **Auto**: där GitPoulpi ersätter sina egna filer (releasearkiv, Windows-installationsprogrammet) laddar det också ned och verifierar uppdateringen i bakgrunden så fort det hittar en. Chipet står då **Starta om för att uppdatera**, och omstarten väljer du fortfarande själv. Där en pakethanterare äger uppdateringen beter sig **Auto** som **Meddela**.
- **Av**: inget anrop görs någonsin och chipet visar bara versionen.

Inställningen **Uppdateringskanal** (`updates.channel`) väljer mellan **Stabil** och **Förhandsversion** (även release candidates). En sökning du startar med **Sök nu** ignorerar gränsen på en gång per dag. Utan nätverksanslutning, eller medan repositoryt är privat och ingen github.com-token är lagrad, misslyckas en bakgrundssökning tyst med en rad i programloggen.

## Omstart och vad du ser efteråt

Att trycka på **Uppdatera och starta om** eller **Kör uppdatering** sparar din session och stänger fönstret. Ett separat litet program, `gitpoulpi-updater`, gör jobbet, så att GitPoulpis egna filer inte används medan de ersätts och en pakethanterare inte blockeras av en väntande förälder. Det stoppar först GitPoulpis bakgrundsdaemoner (din användares övriga fönster och MCP-servrar stängs också), gör uppdateringen, kontrollerar att den nya `gitpoulpi --version` skriver ut den förväntade versionen och startar GitPoulpi igen: i samma terminal i Linux och macOS, i ett nytt fönster i Windows. Om kontrollen misslyckas sätter det tillbaka de gamla filerna.

Nästa start visar en rad i statusraden:

- `Uppdaterad till v1.4.2`
- `Uppdateringen misslyckades: <orsak>. Föregående version är fortfarande installerad.`
- `Uppdateringen avbröts: administratörsgodkännandet nekades. Inget ändrades.`

Bara en uppdatering körs åt gången; en låsfil i GitPoulpis datamapp (och en i installationsmappen när den är skrivbar) håller en andra ute, och ett lås som är äldre än tio minuter ignoreras, så en kraschad uppdatering låser aldrig GitPoulpi för gott. CLI:t och daemonen startar inte så länge ett färskt lås finns.

## Gå tillbaka

Där GitPoulpi ersätter sina egna filer byter **Gå tillbaka** tillbaka de tidigare filerna och startar om. De finns kvar som `<name>.old` en minut efter att den nya versionen har startat och tas sedan bort, så att gå tillbaka är till för den första körningen efter en uppdatering. Där en pakethanterare äger installationen är att gå tillbaka kommandot i tabellen: panelen visar eller kör det. Att gå tillbaka ändrar GitPoulpis egna filer, inte ett repository, så det ingår inte i gits ångra-journal.

## Vad som kontrolleras och nekas

Innan något ersätts laddar GitPoulpi ned arkivet, dess `.minisig`-signatur och dess `.sha256`. Det kontrollerar signaturen först, mot projektets releasenyckel som är inbyggd i GitPoulpi, och signaturens betrodda kommentar måste namnge det här arkivet och den här versionen. Sedan kontrolleras hashen. Det nekar: en release utan `.minisig` (en äldre release som publicerades innan releaser signerades kan inte installeras via uppdateraren, vilket inkluderar att fästa eller gå tillbaka till en), en signatur av en annan nyckel, en signatur för en annan fil eller en annan version, en release utan `.sha256`, en hash som inte stämmer, ett arkiv vars namn inte är det för den här plattformen (`gitpoulpi-<version>-<target>.<zip|tar.gz>`), ett arkiv vars markör nämner en annan version eller plattform, och en nedladdningslänk utanför GitHubs releasevärdar (varje omdirigeringssteg kontrolleras) eller den spegel som en administratör har konfigurerat. Bytet är allt eller inget: om ett annat program håller en av de körbara filerna namnger GitPoulpi filen, ändrar ingenting och lämnar nedladdningen kvar till ett nytt försök. I Windows verifieras en uppdatering för alla användare på nytt av det upphöjda steget, som själv laddar ned `.minisig` och `.sha256` och kontrollerar båda igen, så en fil som bytts ut i användarens nedladdningsmapp emellan nekas. Pakethanterare (Chocolatey, winget, Scoop, Homebrew, apt, AUR) verifierar sina egna paket: GitPoulpi laddar inte ned något där.

## Säkerhetsmeddelanden

Vid sidan av varje release publicerar projektet `security.json`: listan över säkerhetsmeddelanden (en identifierare, en allvarlighetsgrad, de berörda versionerna som ett intervall som `>=1.2.0, <1.4.3`, den åtgärdade versionen, en länk och en sammanfattning på en mening). Samma dagliga sökning som letar efter en ny version hämtar även den, från samma plats som releasen: repositoryts releasesida för GitHub, mappen med `latest.json` för en spegel, GitHub för den offentliga Chocolatey-feeden. Ett meddelande används bara när `security.json.minisig` verifieras mot projektets nyckel; en osignerad eller felaktigt signerad fil visas aldrig (en rad i Aktivitetsloggen säger att den ignorerades).

När ett meddelande nämner den körande versionen blir chipet rött och lyder **Säkerhetsmeddelande: den här versionen påverkas**, vid varje installation, även de som hanteras av en pakethanterare. Uppdateringspanelen listar då varje meddelande med dess allvarlighetsgrad, sammanfattning, åtgärdade version och länk; **Öppna meddelande** (`a`, eller **Uppdateringar: Öppna säkerhetsmeddelande** i paletten) öppnar det allvarligaste meddelandets länk i webbläsaren. Där en pakethanterare äger installationen lägger panelen till "Uppdatera med din pakethanterare."

Med uppdateringssökning av (`updates.mode = "off"`), med en administratörspolicy vars `check` är `none`, och för ett bygge gjort med `cargo build` hämtas ingenting och inget meddelande visas. En organisations egen NuGet-feed får inte GitPoulpi att kontakta GitHub: dess meddelanden kommer bara från policyns `download_base`-spegel.

## Inställningar som en administratör har låst

En administratör kan i en policy för hela datorn ange uppdateringsläget, kanalen, en fäst version, var det ska sökas och om GitPoulpi alls får uppdatera sig självt. I Inställningar visar de raderna **Inställt av din administratör**, panelen visar deras meddelande, och när självuppdatering är avstängd erbjuder panelen kommandona eller en notering i stället för knappar som ersätter filer. Policyfiler som inte går att läsa misslyckas stängt: bara meddela, ingen självuppdatering. Administratörens sida finns i [Företagsdistribution](enterprise-distribution.md#policyfilen-och-registret).

## Miljö

`GITPOULPI_UPDATE_API` pekar sökningen mot en annan API-rot (används av testerna).
