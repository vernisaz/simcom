# 3d party crates SimCommander depends on

There is one 3d party dependance [exif](https://github.com/kamadak/exif-rs), and [mutate_once](https://github.com/kamadak/mutate_once-rs) it depends on.

Checkout the repositories in _projects_ or other directory where you keep 3rd party projects.

## Build
Building the crates without Cargo is trivial. Just copy provided _bee.7b_ in the repository of the project and execute *rb*.
Since *mutate_once* wasn't added in the dependency list of _exif_ build script, you need to add it in the begining of
**src/lib.rs** as `extern crate mutate_once;`. Or you can specify it as `dep_crates=[mutate_once]` in _bee.7b_.
Scripts are configured considering that all 3rd party repositories reside ib _side_ of the _projects_. Since all paths are relative, the 3rd party root repository may
have any name.

If you have a different directory structure, then the path to components in _bee.7b_ has to be modified accordingly.