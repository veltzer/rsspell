# Usage

`rsspell` is easy to use from the command line. Below are the most common commands and flags.

## Scanning for Typos

To scan all Markdown and SVG files in the current directory (and its subdirectories):

```bash
rsspell scan
```

To scan a specific directory or file:

```bash
rsspell scan path/to/your/files
```

To specify a language (e.g., for installed dictionaries):

```bash
rsspell scan --lang de-DE
```

## Managing Dictionaries

`rsspell` includes a built-in `en-US` dictionary. You can manage additional dictionaries with the `dicts` subcommand.

### List Installed Dictionaries

```bash
rsspell dicts list
```

### List Dictionaries Available for Download

```bash
rsspell dicts list-remote
```

### Install a New Dictionary

To install a dictionary from the [wooorm/dictionaries](https://github.com/wooorm/dictionaries) repository:

```bash
rsspell dicts install de-DE
```

### Show Dictionary Path

To see where dictionaries are stored on your system:

```bash
rsspell dicts path
```

## Other Commands

### Show Version Information

```bash
rsspell version
```

### Generate Shell Completions

```bash
rsspell complete <SHELL>
```
Where `<SHELL>` is one of `bash`, `elvish`, `fish`, `powershell`, or `zsh`.
