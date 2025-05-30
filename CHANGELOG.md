<!-- updated by cargo-release -->

# Unreleased

# 0.6.0
- Fix HTML nodes emitting text instead of HTML

# 0.5.0
- Added the `watch` default feature
- It now requires to specify `build` or `watch` mode

# 0.4.3
- Updated dependencies and fixed workflow

# 0.4.2
- Enabled HTML embedding

# 0.4.1
## Lua API
- Added `path_relative_to`, `path_filename`, `path_extension`, `path_join`, `path_with_filename` and `path_with_extension` as path helpers
- `path_parent` and `path_relative` now returns `nil` when erroring instead of producing weird error
- `search_in` now accept `nil` as `depth`
- The base of the `pages` and `output` directories are now in the `pages_base` and `output_base` global variables

# 0.4.0
## Lua API
- `search_in` added the `depth` parameter
- `config.toml` exported variables are now under the global `config` table

# 0.3.0
## Lua API
- Added `path_parent`, `path_relative` and `search_in` global functions
- Changed `generate_final_html` to pass in the output path as well

# 0.2.0
## Markdown compilation
- Emojis now use [gemoji](https://github.com/github/gemoji) shortcodes

# 0.1.1
## Markdown compilation
- Text is now scanned for emojis (`:(emoji name):`)

# 0.1.0
Initial release
