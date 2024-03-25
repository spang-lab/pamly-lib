# pamly-lib

The pamly-lib originates from the initial [pamly](https://pamly.spang-lab.de) project for whole slide image conversion, upload and viewing.
The pamly-lib contains the source code for the pamly rust crate which contains the converter tool of pamly as a CLI. 

## Installing and Compiling Openslide

It builds upon the C-library [Openslide](https://github.com/openslide/openslide) in order to convert the main propietary scanner formats to an accessible sqlite database.
Thus, follow the installation and compilation instructions for [Openslide](https://github.com/openslide/openslide) as the pamly-lib depends on openslide to be properly installed.
Mac users can also install via brew.

NOTE: If openslide is not found during cargo install it might be caused by your PATH, e.g. if you brew install on Mac run
```
export LIBRARY_PATH="/opt/homebrew/lib"
```

If you've installed and compiled via the git repo, add the path to the .dylib, so something like
```
export DYLD_LIBRARY_PATH="$HOME/.../openslide/builddir/src"
```

## Installing the Converter Tool

In order to install the pamly CLI, including the converter tool, run
```
cargo install pamly --features convert
```

Without the flag --features convert you still get other features of pamly which do not depend on openslide, such as generating the types file (containing diagnoses, stains, labels, ...).

## Run the Converter Tool

In order to convert a raw WSI scan into an .sqlite databse run
```
pamly convert <Slide Path>
``` 


