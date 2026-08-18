@echo off
setlocal
cd /d "%~dp0"

gh label create molecules --color 5eead4 --description "The molecules module" --force
gh label create biology --color 4ade80 --description "Nucleic acids and sequences" --force
gh label create proofs --color fb7185 --description "Lean 4" --force
gh label create algebra --color ee6a1a --description "The OCaml module" --force
gh label create fields --color 60a5fa --description "Grid solvers" --force
gh label create export --color a78bfa --description "Getting results out" --force
gh label create rigour --color fbbf24 --description "Accuracy and provenance" --force

gh issue create --title "Molecular descriptors" --body-file 01.md --label molecules
gh issue create --title "2D structure diagrams and SVG" --body-file 02.md --label molecules
gh issue create --title "3D geometry from connectivity" --body-file 03.md --label molecules
gh issue create --title "Interactive 3D view" --body-file 04.md --label molecules
gh issue create --title "Export: SVG, PNG, MOL, SDF, XYZ, PDB, CSV, JSON" --body-file 05.md --label molecules --label export
gh issue create --title "A provenance stamp on every export" --body-file 06.md --label export --label rigour
gh issue create --title "The Molecules module in the interface" --body-file 07.md --label molecules
gh issue create --title "Nucleic acids: sequences and the double helix" --body-file 08.md --label biology
gh issue create --title "Proofs, part two: ln, sqrt, tan and division" --body-file 09.md --label proofs
gh issue create --title "Algebra, part two: partial fractions and factoring over Q" --body-file 10.md --label algebra
gh issue create --title "Life: reaction-diffusion, epidemics, cellular automata" --body-file 11.md --label fields
gh issue create --title "logP and molar refractivity, done properly" --body-file 12.md --label molecules --label rigour
