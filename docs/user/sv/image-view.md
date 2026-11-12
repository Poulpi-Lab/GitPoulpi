[English](../image-view.md) · [Čeština](../cs/image-view.md) · [Deutsch](../de/image-view.md) · [Español](../es/image-view.md) · [Français](../fr/image-view.md) · [Italiano](../it/image-view.md) · [日本語](../ja/image-view.md) · [한국어](../ko/image-view.md) · [Nederlands](../nl/image-view.md) · [Polski](../pl/image-view.md) · [Português (Brasil)](../pt-BR/image-view.md) · [Português (Portugal)](../pt-PT/image-view.md) · [Русский](../ru/image-view.md) · **Svenska** · [Türkçe](../tr/image-view.md) · [Українська](../uk/image-view.md) · [简体中文](../zh-CN/image-view.md)

# Bilder i Diff-vyn och filvyn

En binärfil som är en bild visas som en bild, inte som meddelandet
”(inga textändringar - binärt innehåll, eller bara en läges-/namnändring)”.
Diff-vyn visar **före** och **efter**; filvyn visar filen som den är;
merge-redigeraren visar sidorna av en bild i konflikt. Allt här är skrivskyddat:
att titta på en bild ändrar aldrig repositoryt, så det finns inget att ångra.

Stäng av det med **Inställningar > Bilder > Förhandsvisa bilder**
(`ui.image_view`): då får du det vanliga binärmeddelandet tillbaka. Ett anpassat
bygge utan funktionen `image-view` har ingen visare alls och sidan Bilder
behåller bara listan över redigerare (se [Anpassade byggen](custom-builds.md)).

## Vilka filer

PNG och JPEG stöds alltid; GIF, WebP (deras första bildruta), BMP och ICO/CUR när
bygget har `image-view`. TIFF, AVIF, HEIC, PSD och RAW känns igen men avkodas
inte: ett infokort anger formatet, storleken och måtten. SVG är text och behåller
textdiffen. En bild över **Max förhandsvisningsstorlek (MiB)** (32 MiB som
standard) eller över 64 megapixlar visar ett infokort i stället. En fil lagrad
med Git LFS visas när dess objekt finns i det lokala lagret; annars ber kortet
dig göra `LFS > Pull` först. Transparens ritas över ett schackbräde.

## Hur den ritas

I en terminal med ett bildprotokoll (Kitty och Ghostty, WezTerm och iTerm2, Rio,
mintty och sixel-terminalerna Windows Terminal, foot och Tabby) är bilden en riktig bild. Överallt annars (den vanliga Windows-konsolen, zellij,
en terminal GitPoulpi inte känner till) ritas den med färgade halvblockstecken,
halva den vertikala upplösningen men läsbar överallt; **Utan bildprotokoll** kan
sättas till *Bara infokort*. **Bildprotokoll** på sidan **Log & graf** tvingar fram
ett.

## Diff-vyn

Öppna diffen för en bildfil (en commits, eller de lokala ändringarnas Unstaged-
eller Staged-del). Rubriken namnger båda filerna
(`PNG 512x512 . 34 KiB -> PNG 640x480 . 41 KiB`), sedan zoomknapparna,
**Anpassa**, **1:1** och **Redigera i**.

- **Sida vid sida | Infogad** till höger i rubriken, eller `m`: sida vid sida,
  eller en bild i taget. `Tab`, eller ett klick på *Före | Efter*, visar den
  andra i den infogade vyn. En delning som är för smal för två paneler ritas
  infogad utan att ändra din inställning (`ui.diff_layout`).
- Delningens **avdelare** flyttas genom att dra den, `Alt+Right` /
  `Alt+Left` (före-panelen tar / ger plats) och återgår till mitten med ett
  dubbelklick eller `Alt+0`. Andelen kommer ihåg under sessionen.
- En ny fil visar en panel med bildtexten *Ny fil*, en borttagen *Borttagen
  fil*; båda döljer växlaren. En ospårad bild markerad bland de lokala ändringarna
  förhandsvisas som en ny fil.
- Före-sidan är föräldracommiten (commit-diff), `HEAD` (staged) eller indexet
  (unstaged); efter-sidan är committen, indexet eller filen på disk. Om filen på
  disk ändras medan diffen är öppen (du sparade den från en redigerare, ett
  verktyg skrev om den) följer bilden efter inom ungefär en halv sekund.

## Filvyn

Öppna en bild med **Filvy** från en fils meny eller diffraden: en panel, filen
vid revisionen (eller på disk för de lokala ändringarna).

## Zoom och rullning

| | |
|---|---|
| `+` / `-` (eller `=`) | Zooma in / ut, runt panelens mitt |
| `0` | **Anpassa** (standard): hela bilden, och den förblir anpassad när fönstret eller avdelaren ändrar storlek |
| `1` | Verklig storlek (1:1) |
| Piltangenter, `h j k l` | Rulla en tiondel av panelen; med `Shift` en hel panel; `PageUp`/`PageDown`, `Home`/`End` |
| Hjul | Rulla vertikalt; `Shift`+hjul horisontellt; `Ctrl`+hjul zoomar runt pekaren |
| Dra | Ta tag i bilden och flytta den; rullningslisterna på sidorna fungerar som överallt |

En inzoomad vy behåller sin skala och punkten i mitten när panelen ändrar
storlek; Anpassa förblir Anpassa. I Diff-vyn delar de två panelerna en zoom och
en rullningsposition (**Synka zoom och rullning före/efter**, på som standard),
så samma del av båda bilderna syns; när de har olika storlek hålls zoomen
relativ till var och en. Från **Pixlad zoom från** (2x som standard) förstoras
bilden som fyrkantiga pixlar i stället för att mjukas, vilket passar ikoner och
pixelkonst.

## Redigera i

Knappen **Redigera i** i rubriken listar de bildredigerare som hittats på den
här datorn (GIMP, Krita, Inkscape, Photoshop, Paint.NET, Preview, ... och
systemets standardprogram); `o` kör standardredigeraren och `Shift+O` öppnar
listan. Den finns också i högerklicksmenyn för en bild i fillistorna, och i
kommandopaletten (*Bild: Redigera i...*). Listan och standardvalet ställs in under
**Inställningar > Bilder**, på samma sätt som [Öppna i](open-in.md) fungerar.

Att redigera filen på disk (de lokala ändringarnas sida) går direkt: redigeraren
sparar på plats och bilden uppdateras. Varje annan version (en commits, indexet,
en borttagen fil) skrivs till en **skrivskyddad temporär kopia**, med ett
meddelande i statusraden, så att en redigerare inte kan ändra repositoryt.

Apparna som systemet har registrerat för bildens typ (dess lista *Öppna med*) kommer efter dina redigerare i listan, utan att upprepa ett program som redan finns där. De andra binärfilerna har samma lista, se [Öppna i](open-in.md).

## I merge-redigeraren

En bild i konflikt visar sitt kort med bilderna **A**, **B** och (när det finns
en) **Bas** i rutorna, sida vid sida, var och en anpassad till sin ruta och
skrivskyddad (ingen zoom där; bilderna är bara till hjälp för ditt val).
Knapparna och tangenterna är oförändrade: **Behåll A** (`a`), **Ta B** (`b`),
**Ta bas** (`o`), **Öppna i merge-verktyg** (`m`). Förhandsvisningarna kräver
ett tillräckligt brett fönster (ungefär 70 kolumner) och några lediga rader;
i ett smalare fönster är kortet det rena textkortet. Se
[Lösa merge-konflikter](merge-conflicts.md).

## Tangenter och inställningar

Tangenterna finns i [Kortkommandon](keybindings.md#bildvy); varje tangent kan
ändras och nås från paletten (`Ctrl+P`, skriv *Bild*). Alternativen finns på
sidan **Bilder** i [Inställningar](settings.md#bilder) och i
[konfigurationsreferensen](config-reference.md) (`ui.image_view*`,
`ui.image_editors`).
