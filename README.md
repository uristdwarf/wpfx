# wpfx - A dead simple wine prefix manager

WIP: expect things to break horribly.

Mostly a wrapper around wine to help deal with prefixes and wine versions (like
Proton) and metadata generation for shortcuts. Think Lutris but much more narrow
and command-line/text file oriented.

## Dependencies

Probably `wine`, though not strictly required (by default it will use the system
wine executable, but you can point it to any wine executable by setting the
runner configuration option).

`xrandr`, `grep`, `sort` and `head` are nice to have, but not strictly needed.

## Usage

Install with `cargo install --path .`

Usage:

```console
$ wpfx -h
```

Create a configuration file and prefix directory in the working directory you
want to run wine in.

```console
$ wpfx init
```

Run the specific executable:

```console
$ wpfx run <EXE>
```

## Motivation

Mostly born out of frustration with Lutris. I didn't use Lutris other than to
generate metadata (to make nice shortcuts in my application launcher), manage
prefixes/runners and gamescope. The rest of the features I never used nor will
probably use. Eventually the thing that boiled me over was the fact it was
pretty hard to backup single games in a portable way (without needing Lutris)
and restore them (I usually archive games after I'm done playing them on my
server, rather than simply delete them), which what prompted me to write this.

## Design Goals

- [X] Manage prefixes
- [ ] Manage game/application metadata
- [ ] Aim for portability

### Extras

- [ ] Allow creating executables that can be used in the CLI
- [ ] Manage wine executables

## Philosophy

- Avoid creating your own tools, use the Linux and Wine ecosystem where possible
- Focus solely on the command-line experience
- Configuration and settings should generally be done in a file, but doing it in
  the command line is also an option

### Current objectives

- [x] Allow setting a `WINEPREFIX` in a configuration file, and read that
  (default should be `pfx`). Use the system wine as default for now.
- [x] Add Gamescope support
- [x] Add support for using specific wine version.
- [ ] Add metadata generation
- [ ] Figure out a way to manage multiple wine versions for games while still
  being portable (GNU stow?)
