[English](../image-view.md) · [Čeština](../cs/image-view.md) · [Deutsch](../de/image-view.md) · [Español](../es/image-view.md) · [Français](../fr/image-view.md) · [Italiano](../it/image-view.md) · [日本語](../ja/image-view.md) · [한국어](../ko/image-view.md) · **Nederlands** · [Polski](../pl/image-view.md) · [Português (Brasil)](../pt-BR/image-view.md) · [Português (Portugal)](../pt-PT/image-view.md) · [Русский](../ru/image-view.md) · [Svenska](../sv/image-view.md) · [Türkçe](../tr/image-view.md) · [Українська](../uk/image-view.md) · [简体中文](../zh-CN/image-view.md)

# Afbeeldingen in de Diff-weergave en de Bestandsweergave

Een binair bestand dat een afbeelding is, wordt als afbeelding getoond, niet
met de melding "(no textual changes - binary content...)". De Diff-weergave
toont het **voor** en het **na**; de Bestandsweergave toont het bestand zoals
het is; de merge-editor toont de kanten van een afbeelding in conflict. Alles
hier is alleen-lezen: naar een afbeelding kijken wijzigt de repository nooit,
dus er is niets om ongedaan te maken.

Zet het uit met **Instellingen > Afbeeldingen > Afbeeldingen voorvertonen**
(`ui.image_view`): je krijgt dan de gewone binaire melding terug. Een
aangepaste build zonder de feature `image-view` heeft helemaal geen viewer en
de pagina Afbeeldingen behoudt alleen de lijst met editors (zie
[Aangepaste builds](custom-builds.md)).

## Welke bestanden

PNG en JPEG altijd; GIF, WebP (hun eerste frame), BMP en ICO/CUR als de build
`image-view` heeft. TIFF, AVIF, HEIC, PSD en RAW worden herkend maar niet
gedecodeerd: een infokaart geeft het formaat, de grootte en de afmetingen. SVG
is tekst en houdt de tekstdiff. Een afbeelding boven de **Max. voorbeeldgrootte
(MiB)** (standaard 32 MiB) of boven 64 megapixels toont in plaats daarvan een
infokaart. Een bestand dat met Git LFS is opgeslagen wordt getoond als het
object in de lokale opslag zit; anders zegt de kaart dat je eerst `LFS > Pull`
moet doen. Transparantie wordt over een dambordpatroon getekend.

## Hoe het wordt getekend

Op een terminal met een afbeeldingsprotocol (Kitty en Ghostty, WezTerm en
iTerm2, Rio, mintty) is de afbeelding een echte afbeelding. Overal elders
(gewone Windows Terminal, tmux, een terminal die GitPoulpi niet kent) wordt ze
getekend met gekleurde halfblok-tekens, de helft van de verticale resolutie
maar overal leesbaar; **Zonder afbeeldingsprotocol** kan worden ingesteld op
*Alleen infokaart*. **Afbeeldingsprotocol** op de pagina **Log & grafiek** dwingt er
een af.

## De Diff-weergave

Open de diff van een afbeeldingsbestand (van een commit, of de unstaged of
staged helft van de WIP). De kop noemt beide bestanden
(`PNG 512x512 . 34 KiB -> PNG 640x480 . 41 KiB`), dan de zoomknoppen,
**Aanpassen**, **1:1** en **Bewerken in**.

- **Naast elkaar | Inline** rechts in de kop, of `m`: naast elkaar, of één
  afbeelding tegelijk. **Tab**, of een klik op *Voor | Na*, toont de andere in
  de inline-weergave. Een split die te smal is voor twee panelen wordt inline
  getekend zonder je instelling (`ui.diff_layout`) te wijzigen.
- De **scheiding** van een split verplaats je door hem te slepen,
  `Alt+Right` / `Alt+Left` (het voor-paneel krijgt / geeft ruimte) en
  keert terug naar het midden met een dubbelklik of `Alt+0`. De verdeling wordt
  voor de sessie onthouden.
- Een nieuw bestand toont één paneel met het bijschrift *Nieuw bestand*, een
  verwijderd bestand *Verwijderd bestand*; beide verbergen de schakelaar. Een
  untracked afbeelding die in de lokale wijzigingen is geselecteerd, krijgt een
  voorbeeld als nieuw bestand.
- De voor-kant is de ouder-commit (commit-diff), `HEAD` (staged) of de index
  (unstaged); de na-kant is de commit, de index of het bestand op schijf. Als
  het bestand op schijf verandert terwijl de diff open is (je hebt het in een
  editor opgeslagen, een tool heeft het herschreven), volgt de afbeelding
  binnen ongeveer een halve seconde.

## De Bestandsweergave

Open een afbeelding met **Bestandsweergave** vanuit het menu van een bestand of
de diffbalk: één paneel, het bestand bij de revisie (of op schijf voor de
lokale wijzigingen).

## Zoomen en scrollen

| | |
|---|---|
| `+` / `-` (of `=`) | In- / uitzoomen, rond het midden van het paneel |
| `0` | **Aanpassen** (de standaard): de hele afbeelding, en ze blijft aangepast als het venster of de scheiding van grootte verandert |
| `1` | Ware grootte (1:1) |
| Pijltoetsen, `h j k l` | Een tiende van het paneel scrollen; met `Shift` een heel paneel; `PageUp`/`PageDown`, `Home`/`End` |
| Wiel | Verticaal scrollen; `Shift`+wiel horizontaal; `Ctrl`+wiel zoomt rond de aanwijzer |
| Slepen | Pak de afbeelding vast en verplaats haar; de schuifbalken aan de zijkanten werken zoals overal |

Een ingezoomde weergave behoudt haar schaal en het punt in het midden als het
paneel van grootte verandert; Aanpassen blijft Aanpassen. In de Diff-weergave
delen de twee panelen één zoom en één scrollpositie (**Zoom en scrollen
voor/na synchroniseren**, standaard aan), zodat hetzelfde deel van beide
afbeeldingen in beeld is; als ze in grootte verschillen, blijft de zoom
relatief aan elk. Vanaf **Pixelig zoomen vanaf** (standaard 2x) wordt de
afbeelding vergroot met vierkante pixels in plaats van afgevlakt, wat past bij
pictogrammen en pixelart.

## Bewerken in

De knop **Bewerken in** in de kop toont de afbeeldingseditors die op deze
machine zijn gevonden (GIMP, Krita, Inkscape, Photoshop, Paint.NET, Preview,
... en de standaardapp van het systeem); `o` start de standaard en `Shift+O`
opent de lijst. Hij staat ook in het rechtsklikmenu van een afbeelding in de
bestandslijsten en in het opdrachtenpalet (*Afbeelding: Bewerken in...*). De lijst
en de standaard stel je in onder **Instellingen > Afbeeldingen**, op de manier
van [Openen in](open-in.md).

Het bestand op schijf bewerken (de kant van de lokale wijzigingen) gaat direct: de
editor slaat ter plekke op en de afbeelding wordt vernieuwd. Elke andere versie
(van een commit, de index, een verwijderd bestand) wordt naar een
**alleen-lezen tijdelijke kopie** geschreven, met een melding in de statusbalk,
zodat een editor de repository niet kan wijzigen.

De toepassingen die het systeem voor het type van de afbeelding registreert (de lijst *Openen met*) volgen in de lijst op je editors, zonder een programma te herhalen dat er al staat. De andere binaire bestanden hebben dezelfde lijst, zie [Openen in](open-in.md).

## In de merge-editor

Een afbeelding in conflict toont haar kaart met de afbeeldingen **A**, **B** en
(als die er is) **Basis** in de vakken, naast elkaar, elk aangepast aan zijn
vak en alleen-lezen (daar geen zoom; de afbeeldingen zijn alleen een hulp om te
kiezen). De knoppen en toetsen blijven ongewijzigd: **A behouden** (`a`),
**B overnemen** (`b`), **Basis overnemen** (`o`), **Openen in merge-tool** (`m`). De
voorbeelden hebben een voldoende breed venster nodig (ongeveer 70 kolommen) en
wat vrije rijen; smaller is de kaart de gewone tekstkaart. Zie
[Merge-conflicten oplossen](merge-conflicts.md).

## Toetsen en instellingen

De toetsen staan in [Sneltoetsen](keybindings.md#afbeeldingsweergave); elke
toets kan opnieuw worden toegewezen en is bereikbaar via het palet (`Ctrl+P`,
typ *Afbeelding*). De opties staan op de pagina **Afbeeldingen** van
[Instellingen](settings.md#afbeeldingen) en in de
[configuratienaslag](config-reference.md) (`ui.image_view*`,
`ui.image_editors`).
