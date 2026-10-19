SpringSharp Windows x64 portable
===============================

Extract the complete ZIP to a writable folder; keep assets beside the programs.
No installer, administrator rights, font installation or JavaScript is required.
Target: Windows 10/11 x64 with a working OpenGL driver for the native windows.

- SpringSharp.exe: native egui workbench. Open/Save use native file dialogs.
- SpringSharp-Web.exe: starts a loopback service on an available port and opens
  your browser. Keep its small window open while using the page. Close it or
  select Stop to end the service. No background service is installed.
- springsharp-cli.exe: console interface; run `springsharp-cli.exe help`.

The two graphical programs use the Windows GUI subsystem (no console window).
Only the command-line program retains a console. The MSVC CRT is statically linked.

Fonts
-----
Sans: Noto Sans + Noto Sans CJK SC (shared Chinese/Japanese Han shapes).
Serif: Noto Serif + Noto Serif CJK JP (Mincho shapes for both Chinese and Japanese).
Monospace: Noto Sans Mono + CJK SC fallback.

Select Serif to use the serif family for ship names/headings.
Font groups are Linguae Germanicae (English/German), 汉字 (Chinese/Japanese), and Linguae Italicae (Latin/French). Font selection uses collection/script metadata. Report language is English, Chinese, or Latin with French fallback as detailed below.
English, German, French, Latin diacritics, Chinese and Japanese kana/Han are supported by
these bundled fonts. English UI labels remain the fallback. Unrepresented rare
characters can still require an additional font; no font claims all Unicode.
Font assets, source URLs, SHA-256 hashes and OFL licenses are in assets/fonts/.
Font lookup is relative to the executable, independent of the current directory.

Documents
---------
Save designs anywhere you can write. A cancelled file dialog changes nothing.
Saving calculates and exports the same committed snapshot displayed by the UI.
The browser uses native form submissions and full-page refreshes. Uploading a file
reads it; no file is overwritten by the Web service. Downloads use browser dialogs.
Standalone reports remain readable offline using local/system font fallbacks;
exact bundled typography is guaranteed in the running workbenches, not when a
report is moved alone to another computer without the fonts.

Build from source
-----------------
Install Rust MSVC, Visual Studio C++ Build Tools + Windows SDK, and Python 3.
Run `powershell -ExecutionPolicy Bypass -File scripts/build-windows.ps1` from the
source tree. The script builds only the release binaries, checks CLI startup,
verifies x64 PE/subsystems and font hashes, packages dependencies' license files,
and writes dist/SpringSharp-<version>-windows-x64.zip plus a SHA-256 sidecar.
Intermediate target-windows files are cleaned by default; -KeepBuild retains them.
The manual Windows x64 portable workflow performs the same steps without full gates.

The package is unsigned. Windows runtime acceptance includes native dialog
open/save/cancel, Chinese/Japanese/French paths, DPI scaling and server shutdown.
A cross-compiled package still needs that Windows runtime acceptance.

Build isolation
---------------
Temporary SDK downloads, extracted libraries and cross-linker shims belong only
in the ignored project directory target-tools/. Cross-build output belongs in
target-windows/. Supply linker and library paths per command; do not install the
SDK into /usr or /opt, or change shell profiles or global Cargo configuration.
After packaging, remove both temporary directories and retain only dist/ outputs.
The supplied channel manifest in ../raw/ is an input and must remain untouched.

Superstructure planning
-----------------------
Web: expand Superstructure. egui: Engine > Superstructure.
Select Auto, Low distributed, Compact central, Fore / aft split, Concentrated
tower, Aft-shifted citadel, Flotilla streamlined, or Side island. Mast style is independent: Auto, Pole, Tripod, Lattice,
Pagoda, or Integrated tower. Auto uses the resolved combat modality, tactical echelon, machinery layout,
battery disposition, optical/radar equipment and era; exhaust sizes also respond to power demand.

All positions and longitudinal dimensions are fractions of hull length from
the bow; heights are fractions of hull length above deck. Relative shift and
length/height multipliers modify the inferred layout. Lock geometry uses the
explicit position, length and height instead. Other parts continue to seek deck
space around locks. Funnel correction fixes the group envelope; funnel spacing
is centre-to-centre before group scaling. Aft correction applies only to types
with an aft deckhouse. Reset layout clears type, mast and manual corrections.
Calculate publishes the shared plan; .ssr saves all settings.

Warnings cover deck bounds, part overlaps, battery clearance, close exhaust and
unusual height. Battery clearance is a conservative geometric check, not a
complete three-dimensional firing/smoke simulation. The planner is a concept
layout heuristic, not a naval architecture arrangement solver. It does not add
mass or change the stability calculation; existing miscellaneous/topweight
inputs remain the source of mass. Frontends project the same part/mast geometry.

Single-file startup diagnostics
-------------------------------
The standalone SpringSharp.exe embeds its fonts and can be copied to a local
folder without assets. Windows may show an unknown-publisher/reputation prompt
before application code starts because this build is unsigned. No application
log can be written while Windows is still blocking launch. For personal testing,
a paid signing certificate is not required; only allow a build whose origin you
have verified. Signing identifies the publisher but does not guarantee immediate
SmartScreen reputation for a new release. See:
https://learn.microsoft.com/windows/apps/package-and-deploy/smartscreen-reputation

After launch, springsharp_boot.log appends timestamped, process-specific startup
stages beside the exe, falling back to the working directory or TEMP if needed.
Startup errors return a nonzero exit code and write springsharp_startup_error.log;
Rust panics write springsharp_crash.log with a forced backtrace. Native driver or
loader failures may not invoke the Rust panic handler.

To isolate a graphics-driver problem, launch from PowerShell:
    .\SpringSharp.exe --renderer glow
or explicitly select the default backend:
    .\SpringSharp.exe --renderer wgpu
The explicit selection bypasses automatic fallback, making each backend testable.

Report language and original-formula comparison
----------------------------------------------
The report follows the selected text language: 汉字 renders Chinese, Germanic
renders English, and Italic renders Latin with French nautical/technical terms
where a suitable Latin expression is unavailable.
Switching language preserves user-entered names, design inputs and calculated
values. The desktop report, Web report and HTML export use the same generator.

Each calculated report ends with current / original-formula / percentage-difference
columns. The original-formula column uses the same design with the reconstructed
SpringSharp empirical resistance model, unit power factors, no blending and no
CFD correction, then recalculates dependent values. Remaining weight, stability,
combat-system and iteration calculations use this project's engine. This is a
Rust reconstruction comparison, not a result obtained by running SpringSharp 3.0.
The appendix reports convergence and uses long tons for mass.

Interface language scope
------------------------
The Italic selection also uses Latin with French technical terms in parameter
labels, discrete option catalogs, existing localized headings/navigation, metric
cards and weight legends. Selected functional actions, propulsion choices and
status messages additionally use the same policy. Chinese retains its existing
localization scope; this does not attempt to translate every decorative heading,
proper name or diagnostic. Language switching never changes saved field keys,
option indices, design inputs or the committed calculation snapshot.
