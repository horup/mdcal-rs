# mdcal

`mdcal` is a Rust CLI that downloads an iCal feed from a URL and renders calendar data as markdown or html.

Built using VS Code, OpenCode using Big Pickle.

## Setup

Provide an iCal URL:

```bash
mdcal https://example.com/calendar.ics year
mdcal https://example.com/calendar.ics month3
```

## Usage

```bash
mdcal https://example.com/calendar.ics year
mdcal https://example.com/calendar.ics month3
```

## Commands

- `mdcal`: shows help
- `mdcal year`: renders the current year calendar
- `mdcal month3`: renders the current month and the next two months

## Options

- `-l, --lang`: Language code (e.g., en, da)
- `--format`: Output format (markdown or html, default: markdown)
- `--file`: Output file (default is stdout)