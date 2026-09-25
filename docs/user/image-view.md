**English** · [Čeština](cs/image-view.md) · [Deutsch](de/image-view.md) · [Español](es/image-view.md) · [Français](fr/image-view.md) · [Italiano](it/image-view.md) · [日本語](ja/image-view.md) · [한국어](ko/image-view.md) · [Nederlands](nl/image-view.md) · [Polski](pl/image-view.md) · [Português (Brasil)](pt-BR/image-view.md) · [Português (Portugal)](pt-PT/image-view.md) · [Русский](ru/image-view.md) · [Svenska](sv/image-view.md) · [Türkçe](tr/image-view.md) · [Українська](uk/image-view.md) · [简体中文](zh-CN/image-view.md)

# Pictures in the Diff view and the File View

A binary file that is an image is shown as a picture, not as the
"(no textual changes - binary content...)" notice. The Diff view shows the
**before** and the **after**; the File View shows the file as it is; the merge
editor shows the sides of a conflicted picture. Everything here is read-only:
looking at a picture never changes the repository, so there is nothing to undo.

Turn it off with **Settings > Images > Preview images** (`ui.image_view`): you
get the plain binary notice back. A custom build without the `image-view`
feature has no viewer at all and the Images page keeps only the editors list
(see [Custom builds](custom-builds.md)).

## Which files

PNG and JPEG always; GIF, WebP (their first frame), BMP and ICO/CUR when the
build has `image-view`. TIFF, AVIF, HEIC, PSD and RAW are recognized but not
decoded: an info card gives the format, the size and the dimensions. SVG is text
and keeps the text diff. An image over **Max preview size** (32 MiB by default)
or over 64 megapixels shows an info card instead. A file stored with Git LFS is
shown when its object is in the local store; otherwise the card says to
`LFS > Pull` first. Transparency is drawn over a checkerboard.

## How it is drawn

On a terminal with an image protocol (Kitty and Ghostty, WezTerm and iTerm2,
Rio, mintty) the picture is a real image. Anywhere else (plain Windows Terminal,
tmux, a terminal GitPoulpi does not know) it is drawn with colored half-block
characters, half the vertical resolution but legible everywhere; **Without an
image protocol** can be set to *Info card only*. **Image protocol** on the
**Log & graph** page forces one.

## The Diff view

Open the diff of an image file (a commit's, the WIP's unstaged or staged
half). The header names both files (`PNG 512x512 . 34 KiB -> PNG 640x480 . 41 KiB`),
then the zoom buttons, **Fit**, **1:1** and **Edit in**.

- **Split | Inline** at the right of the header, or `m`: side by side, or one
  picture at a time. **Tab**, or a click on *Before | After*, shows the other
  one in the inline view. A split too narrow for two panes is drawn inline
  without changing your setting (`ui.diff_layout`).
- The **divider** of a split moves by dragging it, `Alt+Right` /
  `Alt+Left` (the before pane takes / gives room) and returns to the middle with
  a double-click or `Alt+0`. The share is remembered for the session.
- A new file shows one pane captioned *New file*, a deleted one *Deleted
  file*; both hide the toggle. An untracked picture selected in the working
  changes is previewed as a new file.
- The before side is the parent commit (commit diff), `HEAD` (staged) or the
  index (unstaged); the after side is the commit, the index or the file on
  disk. If the file on disk changes while the diff is open (you saved it from
  an editor, a tool rewrote it) the picture follows within about half a second.

## The File View

Open a picture with **File View** from a file's menu or the diff bar: one pane,
the file at the revision (or on disk for the working changes).

## Zoom and scroll

| | |
|---|---|
| `+` / `-` (or `=`) | Zoom in / out, around the center of the pane |
| `0` | **Fit** (the default): the whole picture, and it stays fitted when the window or the divider is resized |
| `1` | Actual size (1:1) |
| Arrows, `h j k l` | Scroll a tenth of the pane; with `Shift` a full pane; `PageUp`/`PageDown`, `Home`/`End` |
| Wheel | Scroll vertically; `Shift`+wheel horizontally; `Ctrl`+wheel zooms around the pointer |
| Drag | Grab the picture and move it; the scrollbars on the sides work as everywhere |

A zoomed view keeps its scale and the point at its center when the pane is
resized; Fit stays Fit. In the Diff view the two panes share one zoom and one
scroll position (**Sync before/after zoom and scroll**, on by default), so the
same part of both pictures is in view; when they differ in size the zoom is kept
relative to each. From **Zoom in pixelated from** (2x by default) the picture is
enlarged as square pixels instead of smoothed, which suits icons and pixel art.

## Edit in

The **Edit in** button of the header lists the image editors found on this
machine (GIMP, Krita, Inkscape, Photoshop, Paint.NET, Preview, ... and the
system's default app); `o` runs the default one and `Shift+O` opens the list.
It is also in the right-click menu of an image in the file lists, and in the
command palette (*Image: Edit in...*). The list and the default are set under
**Settings > Images**, the way [Open in](open-in.md) works.

Editing the file on disk (the working-changes side) is direct: the editor saves
in place and the picture refreshes. Any other version (a commit's, the index, a
deleted file) is written to a **read-only temp copy**, with a notice in the
status line, so an editor cannot change the repository.

The applications the system registers for the picture's type (its *Open with* list) follow your editors in the list, without repeating a program that is already there. The other binary files have the same list under [Open in](open-in.md).

## In the merge editor

A conflicted picture shows its card with the **A**, **B** and (when there is
one) **Base** pictures in the boxes, side by side, each fitted to its box and
read-only (no zoom there; the pictures are only to help you choose). The buttons
and keys are unchanged: **Keep A** (`a`), **Take B** (`b`), **Take base** (`o`),
**Open in merge tool** (`m`). The previews need a wide enough window (about 70
columns) and some free rows; narrower, the card is the plain text one. See
[Resolving merge conflicts](merge-conflicts.md).

## Keys and settings

The keys are in [Keybindings](keybindings.md#image-view); every one is
rebindable and reachable from the palette (`Ctrl+P`, type *Image*). The options
are on the **Images** page of [Settings](settings.md#images) and in the
[config reference](config-reference.md) (`ui.image_view*`, `ui.image_editors`).
