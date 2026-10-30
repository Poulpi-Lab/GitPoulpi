[English](../submodules.md) · [Čeština](../cs/submodules.md) · [Deutsch](../de/submodules.md) · [Español](../es/submodules.md) · [Français](../fr/submodules.md) · [Italiano](../it/submodules.md) · [日本語](../ja/submodules.md) · [한국어](../ko/submodules.md) · **Nederlands** · [Polski](../pl/submodules.md) · [Português (Brasil)](../pt-BR/submodules.md) · [Português (Portugal)](../pt-PT/submodules.md) · [Русский](../ru/submodules.md) · [Svenska](../sv/submodules.md) · [Türkçe](../tr/submodules.md) · [Українська](../uk/submodules.md) · [简体中文](../zh-CN/submodules.md)

# Submodules

Een submodule is een andere repository die op een vaste commit in de jouwe is genest. GitPoulpi toont ze in de zijbalk Refs, werkt ze bij vanuit de UI, de CLI of het opdrachtenpalet, en kan ze gelijk houden met de commit die je uitcheckt.

## De sectie SUBMODULES

Als een repository een `.gitmodules` heeft, krijgt de zijbalk Refs een sectie **SUBMODULES** (onder de andere; ze ontbreekt als er geen zijn). Eén rij per submodule: een statusteken, het pad en een korte commit.

| Teken | Status | Betekenis |
|---|---|---|
| `✓` | up-to-date | uitgecheckt op de commit die de repository vastlegt |
| `↑` | gewijzigd | uitgecheckt op een andere commit (hij is verder gegaan, of je wisselde van branch zonder hem bij te werken) |
| `○` | niet gekloond | gedeclareerd in `.gitmodules` en vastgelegd, maar de map is leeg: werk hem bij om hem te klonen |
| `✗` | in conflict | de repository heeft merge-conflicten op de submodule |

| Doel | Muis | Toetsenbord |
|---|---|---|
| De submodule als eigen tabblad openen | dubbelklik op de rij | selecteer hem, `Enter` (palet: `Refs: Submodule openen in tabblad`) |
| Menu: **Bijwerken (kloon indien nodig)**, **Initialiseren**, **Openen in nieuw tabblad** | rechtsklik op de rij | selecteer hem, de contextmenutoets (standaard `.`) |
| De geselecteerde submodule bijwerken (kloont hem eerst indien nodig) | menu | `Ctrl+P`: `Refs: Submodule bijwerken (geselecteerd)` |
| Hem registreren in `.git/config` zonder te klonen | menu | `Ctrl+P`: `Refs: Submodule initialiseren (geselecteerd)` |
| Elke submodule bijwerken | | `Ctrl+P`: `Git: Alle submodules bijwerken` |
| Naar de sectie springen | klik op de kop | `Ctrl+P`: `Navigatie: Focus op Refs > Submodules` |

Een update kan ophalen van de remote van de submodule, dus hij volgt **Instellingen > SSH & inloggegevens > HTTPS-inloggegevens** zoals elke andere overdracht. Een submodule die niet gekloond is, kan niet als tabblad worden geopend; de statusbalk zegt dat en verwijst naar Bijwerken.

## Submodules up-to-date houden

**Instellingen > Synchronisatie > Submodules up-to-date houden** (`general.keep_submodules_up_to_date`, standaard uit). Als dit aan staat, wordt na een geslaagde **checkout** of **pull** `git submodule update --init --recursive` uitgevoerd. Dat gebeurt alleen als een submodule niet meer overeenkomt met wat de repository nu vastlegt, dus een repository zonder submodules wordt niet vertraagd. Als die update mislukt, zijn de checkout of pull toch uitgevoerd en dat staat in de foutmelding.

## Vanaf de opdrachtregel

```
gitpoulpi submodule list                       # pad, status, url (--json: de volledige lijst)
gitpoulpi submodule init [<path>...]           # registreren, niet klonen
gitpoulpi submodule update [<path>...] [--init] [--recursive]
```

Zonder paden worden alle submodules bedoeld. Dezelfde drie zijn MCP-tools (`submodule_list` leest alleen; `init` en `update` zijn mutaties en volgen `allow_mutations` van `[mcp]`).

## Opmerkingen

- Alleen het hoogste niveau wordt getoond; de eigen submodules van een submodule verschijnen als je hem als tabblad opent (of gebruik `--recursive` om ze bij te werken).
- Nieuwere git weigert een submodule met een *lokaal pad* te klonen via het transport `file`, tenzij `protocol.file.allow` het toestaat. Dat geldt alleen voor submodules waarvan de URL een lokale map is; stel het in je globale git-configuratie in als je ze gebruikt.
