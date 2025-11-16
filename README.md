# Factorio achievements editor

This program can parse and make limited edits to [Factorio][factorio] `achievements.dat` files.

Note that Factorio does not appear to use an `achievements.dat` file to track achievements while running under Steam.

This implementation works only for Factorio 2.x, and is based primarily on the [documentation in the Factorio Wiki][wiki].


## Installation

Build and install using [Cargo][cargo]:

```sh
$ cargo install --path .
```


## Usage

Provide a file to read on standard input:

```sh
$ factorio-achievements-editor < ~/.factorio/achievements.dat
$ cat ~/.factorio/achievements-modded.dat | factorio-achievements-editor
```

By default, the program will simply dump the parsed file contents on standard error.
This behaviour can also be chosen explicitly using the `dump` command:

```sh
$ factorio-achievements-editor dump < ~/.factorio/achievements.dat
```

The `list` command prints the IDs of all achievements currently tracked in the file to standard error:

```sh
$ factorio-achievements-editor list < ~/.factorio/achievements.dat
```

The `delete` command takes an achievement ID as an argument, deletes that achievement from the file and prints the resulting file to standard output:

```sh
$ cp ~/.factorio/achievements.dat ~/.factorio/achievements.dat.backup
$ factorio-achievements-editor delete lazy-bastard < ~/.factorio/achievements.dat.backup > ~/.factorio/achievements.dat
```


## Non-features

- There is no option to unlock achievements.
  You are welcome to modify the program to add such a feature, but I will not help you nor accept patches that add it.


## License

GNU Affero General Public License, version 3 or later.


[cargo]: https://doc.rust-lang.org/cargo/
[factorio]: https://www.factorio.com/
[wiki]: https://wiki.factorio.com/Achievement_file_format
