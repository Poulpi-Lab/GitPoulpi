[English](../submodules.md) · [Čeština](../cs/submodules.md) · [Deutsch](../de/submodules.md) · [Español](../es/submodules.md) · [Français](../fr/submodules.md) · [Italiano](../it/submodules.md) · [日本語](../ja/submodules.md) · [한국어](../ko/submodules.md) · [Nederlands](../nl/submodules.md) · [Polski](../pl/submodules.md) · [Português (Brasil)](../pt-BR/submodules.md) · [Português (Portugal)](../pt-PT/submodules.md) · [Русский](../ru/submodules.md) · **Svenska** · [Türkçe](../tr/submodules.md) · [Українська](../uk/submodules.md) · [简体中文](../zh-CN/submodules.md)

# Submoduler

En submodul är ett annat repository nästlat i ditt, vid en fast commit. GitPoulpi listar dem i sidopanelen Refs, uppdaterar dem från gränssnittet, CLI:n eller kommandopaletten och kan hålla dem i takt med den commit du checkar ut.

## Avsnittet SUBMODULER

När ett repository har en `.gitmodules` får sidopanelen Refs ett avsnitt **SUBMODULER** (under de andra; det saknas när det inte finns några). En rad per submodul: ett tillståndsmärke, dess sökväg och en kort commit.

| Märke | Tillstånd | Betydelse |
|---|---|---|
| `✓` | uppdaterad | utcheckad vid den commit repositoryt registrerar |
| `↑` | ändrad | utcheckad vid en annan commit (den har gått vidare, eller du bytte gren utan att uppdatera den) |
| `○` | inte klonad | deklarerad i `.gitmodules` och registrerad, men dess mapp är tom: uppdatera den för att klona den |
| `✗` | i konflikt | repositoryt har merge-konflikter på submodulen |

![Avsnittet SUBMODULER med tre rader: vendor/almanac inte klonad (○), vendor/ephemeris uppdaterad (✓) och vendor/starfield ändrad (↑, vid en annan commit än den registrerade), vilket också syns som en ändrad fil i Status-rutan](../images/submodules.png)

| Gör | Mus | Tangentbord |
|---|---|---|
| Öppna submodulen som en egen flik | dubbelklicka på raden | markera den, `Enter` (palett: `Refs: Öppna submodul i flik`) |
| Meny: **Uppdatera (klona vid behov)**, **Initiera**, **Öppna i ny flik** | högerklicka på raden | markera den, kontextmenytangenten (standard `.`) |
| Uppdatera den markerade submodulen (klonar den först vid behov) | meny | `Ctrl+P`: `Refs: Uppdatera submodul (vald)` |
| Registrera den i `.git/config` utan att klona | meny | `Ctrl+P`: `Refs: Initiera submodul (vald)` |
| Uppdatera varje submodul | | `Ctrl+P`: `Git: Uppdatera alla submoduler` |
| Hoppa till avsnittet | klicka på dess rubrik | `Ctrl+P`: `Navigering: Fokusera Refs > Submoduler` |

![Högerklicksmenyn för den ändrade raden vendor/starfield: Uppdatera (klona vid behov), Initiera och Öppna i ny flik](../images/submodule-menu.png)

En uppdatering kan göra fetch från submodulens remote, så den följer **Inställningar > SSH & inloggning > HTTPS-uppgifter** som varje annan överföring. En submodul som inte är klonad har inget att öppna som flik; statusraden säger det och pekar på Uppdatera.

## Håll submoduler uppdaterade

**Inställningar > Synkronisering > Håll submoduler uppdaterade** (`general.keep_submodules_up_to_date`, av som standard). När den är på följs en lyckad **checkout** eller **pull** av `git submodule update --init --recursive`. Det körs bara om en submodul är ur takt med vad repositoryt nu registrerar, så ett repository utan submoduler bromsas inte. Om uppdateringen misslyckas har checkouten eller pullen ändå skett och felet säger det.

## Från kommandoraden

```
gitpoulpi submodule list                       # sökväg, tillstånd, url (--json: hela listan)
gitpoulpi submodule init [<path>...]           # registrera, klona inte
gitpoulpi submodule update [<path>...] [--init] [--recursive]
```

Inga sökvägar betyder varje submodul. De tre finns också som MCP-verktyg (`submodule_list` bara läser; `init` och `update` ändrar något och följer `allow_mutations` i `[mcp]`).

## Anteckningar

- Bara översta nivån listas; en submoduls egna submoduler visas när du öppnar den som en flik (eller använd `--recursive` för att uppdatera dem).
- Nyare versioner av git vägrar klona en submodul med *lokal sökväg* över transporten `file` om inte `protocol.file.allow` tillåter det. Det gäller bara submoduler vars URL är en lokal mapp; ställ in det i din globala git-konfiguration om du använder sådana.
