@echo off
setlocal
cd /d "%~dp0"

gh label create design --color e879f9 --description "Interface, design system, components" --force
gh label create accessibility --color 22d3ee --description "WCAG, keyboard, screen readers" --force
gh label create distribution --color f97316 --description "Reach: embeds, links, offline" --force
gh label create platform --color 818cf8 --description "Library, CLI, language bindings" --force
gh label create teaching --color 34d399 --description "Schools, worksheets, examples" --force

gh issue create --title "Design system: an instrument, not a dashboard" --body-file 13.md --label design
gh issue create --title "Command palette" --body-file 14.md --label design
gh issue create --title "Every number explains itself" --body-file 15.md --label design --label rigour
gh issue create --title "Accessibility to WCAG 2.2 AA" --body-file 16.md --label accessibility
gh issue create --title "Responsive and touch" --body-file 17.md --label design
gh issue create --title "Make kosmos citable: CITATION.cff and a DOI" --body-file 18.md --label rigour
gh issue create --title "A validation report, published and versioned" --body-file 19.md --label rigour
gh issue create --title "Permalinks that carry the whole state" --body-file 20.md --label distribution
gh issue create --title "Embed kosmos anywhere" --body-file 21.md --label distribution
gh issue create --title "Works offline" --body-file 22.md --label distribution
gh issue create --title "@kosmos/engine on npm" --body-file 23.md --label platform
gh issue create --title "kosmos for Python" --body-file 24.md --label platform
gh issue create --title "A command-line kosmos" --body-file 25.md --label platform
gh issue create --title "Speak the reader's language" --body-file 26.md --label distribution
gh issue create --title "What an IT department needs to say yes" --body-file 27.md --label platform
gh issue create --title "Worksheets and a gallery of worked examples" --body-file 28.md --label teaching
