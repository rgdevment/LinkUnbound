# Assets

The images drawn for LinkUnbound that the build does not use: the Microsoft
Store listing, promotional art, the GitHub social preview and screenshots for
the README and the wiki. The images the application ships with live next to the
code that reads them, in `app/src-tauri/icons/` and `app/src/assets/`.

| Folder | What it holds | Where it goes |
| --- | --- | --- |
| `store/screenshots/` | The Microsoft Store screenshots, one set per language | The Store listing, the README and the wiki |
| `store/logos/` | The square logos the Store asks for | The Store listing |
| `promo/` | Box art, hero and poster | The Store listing and press |
| `social/` | The repository's social preview, in two colours | GitHub settings and the top of the README |
| `screenshots/` | Real captures: the picker in motion and the settings window | The README and the wiki |

## Store screenshots

`store/screenshots/en/` and `store/screenshots/es/` each hold seven
1920×1080 images, named `<order>-<subject>.png` in their own language and in
the same order: the picker, its two looks, rules, browsers, settings, privacy
and the closing card. The wiki links to them on `main`, so renaming one means
updating the wiki in the same change.

## Mascot

LinkBear's sources are in `resources/mascot/`: `linkunbound-head.svg` in
colour and `linkunbound-head-mono.svg` in one ink. `app/src/assets/linkunbound.svg`
is the copy the window shows in About.
