# Change Log
All notable changes to this project will be documented in this file.
 
The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [0.3.1] - 2026-11-03
### Added
- Portable PNG and SVG generation (missing in 0.3.0)

## [0.3.0] - 2026-11-03
### Added
- Animated playback
- Chess.com style icons
- Allow flipping the board to blacks perspective

### Changed
- Use resvg to composite the SVG in code instead of the templated files

## [0.2.1] - 2023-07-18
### Fixed
- Fixed handling of non-text non-chess markdown elements (tables)

## [0.2.0] - 2022-09-04
### Added
- Square highlighting
- Implemented move arrows and expose via lines field in yaml

### Changed
- Only log chapters containing chess blocks
- Fixed squares on board so A1 is dark not light
 
## [0.1.0] - 2022-08-06
### Added
- Initial release. Specify boards and render to mdbook. 
