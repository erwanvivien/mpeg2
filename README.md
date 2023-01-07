# MPEG-2 Decoder

Ce projet a été réalisé dans le cadre du cours de TVID à l'EPITA.

## Partie A - Décoder une vidéo MPEG2

### 0. Installer Rust

Si Rust n'est pas installé, il faut l'installer avec [rustup](https://rustup.rs/).

Sous bash :

```bash
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Sous Windows : \
[https://win.rustup.rs/x86_64](https://win.rustup.rs/x86_64)

### 1. Visualiser une vidéo MPEG2

```bash
cargo run --release -- --pathdir="./videos/elementary"
```

### 2. Comment utiliser l'application

Il faut d'abord générer les PGMs et le fichier de log (tvid.log) à partir de la vidéo MPEG2.

Les divers fichiers seront générés dans le dossier courant.

```bash
$ ./tools/mpeg2dec/src/mpeg2dec videos/elementary/pendulum.m2v -o pgm -l -v
```

Les différents paramètres sont :

```bash
$ cargo run --release -- --help

MPEG2 Decoder

Usage: mpeg2.exe [OPTIONS]

Options:
  -p, --pathdir <PATHDIR>  Folder containing *.pgm and tvid.log files (default: videos/pendulum)
  -f, --fps <FPS>
  -m, --mode <MODE>        "[PROG] [RFF] [TFF]"
  -t, --threshold <t>      Weaving threshold
  -h, --help               Print help information
  -V, --version            Print version information
```

Pour lancer l'application avec les paramètres :

```bash
cargo run --release -- --pathdir="."
```
