# haygeom

Geographic mapping plugin for Hayashi.

## Installation

```bash
hay install sheep-farm/haygeom
```

## Usage

```hayashi
import("sheep-farm/haygeom", as=geo)

// Create a map
let map = geo::map(800, 600)

// Add a polygon layer (e.g., states)
map = geo::add_layer(map, states, {"type": "polygon", "fill": "#2D3E50"})

// Add a point layer (e.g., capitals)
map = geo::add_layer(map, capitals, {"type": "point", "color": "red", "size": 5})

// Render to SVG
let svg = geo::render(map)
write(svg, "map.svg")
```

## Features

- Multiple layers (polygons, points, lines)
- Independent styling per layer
- WKT geometry parsing
- SVG output with viewBox for proper scaling

## Development

```bash
cargo build --release
cp target/release/libhaygeom.so ~/.hay/packages/sheep-farm/haygeom.so
```
