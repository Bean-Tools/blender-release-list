# blender-release-list

This is a daughter project of [blender-release](https://github.com/Bean-Tools/blender-beans/). It is a list of all the Blender releases, with their download links, and their release dates. You can view the current list on [blender-releases.bean.tools](https://blender-releases.bean.tools/releases.json).

The code is a bit messy since it is parsing HTML, but it's the best we can do for now. See [DotBow/Blender-Launcher Issue #213](https://github.com/DotBow/Blender-Launcher/issues/213) for further discussion on why we generate a cached response instead of scraping the Blender servers directly.

## How to use

You can either generate a binary file or run the code directly with `cargo run`. The binary file is generated with `cargo build --release`.

The list that is generated will be written to STDOUT. You can redirect it to a file, or pipe it to another program.

## Library

This project also shares a library which is used by Blender Beans. It exports the current struct of the release list for use with `serde` in Blender Beans.