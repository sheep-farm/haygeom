//! WKT (Well-Known Text) → SVG path converter for map rendering.
//!
//! Parses WKT geometry strings (POINT, MULTIPOLYGON, POLYGON, etc.) and
//! converts them to SVG `<path>` elements with appropriate coordinate
//! projection (linear scaling from geographic bounds to pixel space).

use std::fmt::Write;

/// A parsed WKT geometry.
#[derive(Debug, Clone)]
pub enum Geometry {
    Point(f64, f64),
    MultiPoint(Vec<(f64, f64)>),
    LineString(Vec<(f64, f64)>),
    MultiLineString(Vec<Vec<(f64, f64)>>),
    Polygon(Vec<Vec<(f64, f64)>>),
    MultiPolygon(Vec<Vec<Vec<(f64, f64)>>>),
    GeometryCollection(Vec<Geometry>),
}

impl Geometry {
    /// Compute the bounding box of this geometry.
    pub fn bounds(&self) -> Option<(f64, f64, f64, f64)> {
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        let mut visit = |x: f64, y: f64| {
            if x < min_x { min_x = x; }
            if x > max_x { max_x = x; }
            if y < min_y { min_y = y; }
            if y > max_y { max_y = y; }
        };

        match self {
            Geometry::Point(x, y) => visit(*x, *y),
            Geometry::MultiPoint(pts) => for &(x, y) in pts { visit(x, y); },
            Geometry::LineString(pts) => for &(x, y) in pts { visit(x, y); },
            Geometry::MultiLineString(lines) => {
                for line in lines { for &(x, y) in line { visit(x, y); } }
            }
            Geometry::Polygon(rings) => {
                for ring in rings { for &(x, y) in ring { visit(x, y); } }
            }
            Geometry::MultiPolygon(polys) => {
                for rings in polys { for ring in rings { for &(x, y) in ring { visit(x, y); } } }
            }
            Geometry::GeometryCollection(geoms) => {
                for g in geoms {
                    if let Some((x0, y0, x1, y1)) = g.bounds() {
                        visit(x0, y0);
                        visit(x1, y1);
                    }
                }
            }
        }

        if min_x.is_infinite() { None } else { Some((min_x, min_y, max_x, max_y)) }
    }
}

/// Simple WKT parser. Handles POINT, MULTIPOINT, LINESTRING,
/// MULTILINESTRING, POLYGON, MULTIPOLYGON, GEOMETRYCOLLECTION.
pub fn parse_wkt(wkt: &str) -> Result<Geometry, String> {
    let wkt = wkt.trim();
    let upper = wkt.to_uppercase();

    if upper.starts_with("POINT") {
        let coords = extract_coords_paren(wkt)?;
        if coords.len() != 1 {
            return Err(format!("POINT expects 1 coordinate pair, got {}", coords.len()));
        }
        Ok(Geometry::Point(coords[0].0, coords[0].1))
    } else if upper.starts_with("MULTIPOINT") {
        let coords = extract_coords_paren(wkt)?;
        Ok(Geometry::MultiPoint(coords))
    } else if upper.starts_with("LINESTRING") {
        let coords = extract_coords_paren(wkt)?;
        Ok(Geometry::LineString(coords))
    } else if upper.starts_with("MULTILINESTRING") {
        let lines = extract_nested_coords(wkt)?;
        Ok(Geometry::MultiLineString(lines))
    } else if upper.starts_with("POLYGON") {
        let rings = extract_nested_coords(wkt)?;
        Ok(Geometry::Polygon(rings))
    } else if upper.starts_with("MULTIPOLYGON") {
        // MULTIPOLYGON has 3 levels: polygons → rings → coords
        let polys = extract_double_nested_coords(wkt)?;
        Ok(Geometry::MultiPolygon(polys))
    } else if upper.starts_with("GEOMETRYCOLLECTION") {
        // Not commonly needed for geobr data
        Err("GEOMETRYCOLLECTION not supported".into())
    } else {
        Err(format!("unknown WKT type: {}", &wkt[..wkt.len().min(20)]))
    }
}

/// Extract coordinate pairs from the innermost parentheses.
/// e.g. "POINT (1 2)" → [(1.0, 2.0)]
/// e.g. "LINESTRING (0 0, 1 1)" → [(0,0), (1,1)]
fn extract_coords_paren(wkt: &str) -> Result<Vec<(f64, f64)>, String> {
    let start = wkt.find('(')
        .ok_or_else(|| format!("expected '(' in WKT: {}", &wkt[..wkt.len().min(40)]))?;
    let end = wkt.rfind(')')
        .ok_or("expected ')' in WKT")?;
    let inner = &wkt[start + 1..end];
    parse_coord_list(inner)
}

/// Extract nested coordinate lists (one level of nesting).
/// e.g. "POLYGON ((0 0, 1 1, 0 0), (2 2, 3 3, 2 2))"
/// → [[(0,0), (1,1), (0,0)], [(2,2), (3,3), (2,2)]]
fn extract_nested_coords(wkt: &str) -> Result<Vec<Vec<(f64, f64)>>, String> {
    let start = wkt.find('(')
        .ok_or("expected '(' in WKT")?;
    let end = wkt.rfind(')')
        .ok_or("expected ')' in WKT")?;
    let inner = &wkt[start + 1..end];

    // Split by top-level parentheses
    let mut rings = Vec::new();
    let mut depth = 0;
    let mut current_start = 0;

    for (i, ch) in inner.char_indices() {
        match ch {
            '(' => {
                if depth == 0 { current_start = i; }
                depth += 1;
            }
            ')' => {
                depth -= 1;
                if depth == 0 {
                    let ring_str = &inner[current_start + 1..i];
                    let coords = parse_coord_list(ring_str)?;
                    rings.push(coords);
                }
            }
            _ => {}
        }
    }

    if rings.is_empty() {
        return Err("no rings found in WKT".into());
    }
    Ok(rings)
}

/// Extract double-nested coordinate lists (two levels of nesting).
/// e.g. "MULTIPOLYGON (((0 0, 1 1, 0 0)), ((2 2, 3 3, 2 2)))"
/// → [[[(0,0), (1,1), (0,0)]], [[(2,2), (3,3), (2,2)]]]
fn extract_double_nested_coords(wkt: &str) -> Result<Vec<Vec<Vec<(f64, f64)>>>, String> {
    // Find the content between the first '(' and the matching last ')'
    let start = wkt.find('(')
        .ok_or("expected '(' in WKT")?;
    let inner = &wkt[start + 1..];

    // We need to split by top-level groups (depth 1) which contain
    // ring groups (depth 2) which contain coordinates.
    // Strategy: find all depth-2 groups (rings) and group them by
    // their parent depth-1 group.

    let mut polygons: Vec<Vec<Vec<(f64, f64)>>> = Vec::new();
    let mut current_polygon: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut depth = 0;
    let mut ring_buf = String::new();

    for ch in inner.chars() {
        match ch {
            '(' => {
                depth += 1;
                if depth >= 2 {
                    if depth == 2 {
                        ring_buf.clear();
                    } else {
                        ring_buf.push(ch);
                    }
                }
            }
            ')' => {
                if depth >= 2 {
                    if depth == 2 {
                        // End of ring
                        let coords = parse_coord_list(&ring_buf)?;
                        current_polygon.push(coords);
                        ring_buf.clear();
                    } else {
                        ring_buf.push(ch);
                    }
                }
                depth -= 1;
                if depth == 1 && !current_polygon.is_empty() {
                    // End of polygon
                    polygons.push(std::mem::take(&mut current_polygon));
                }
            }
            _ => {
                if depth >= 2 {
                    ring_buf.push(ch);
                }
            }
        }
    }

    if polygons.is_empty() {
        return Err("no polygons found in MULTIPOLYGON WKT".into());
    }
    Ok(polygons)
}

/// Parse "1.5 2.3, 4.5 6.7" into [(1.5, 2.3), (4.5, 6.7)]
fn parse_coord_list(s: &str) -> Result<Vec<(f64, f64)>, String> {
    let mut coords = Vec::new();
    for part in s.split(',') {
        let part = part.trim();
        if part.is_empty() { continue; }
        let nums: Vec<f64> = part.split_whitespace()
            .filter_map(|n| n.parse::<f64>().ok())
            .collect();
        if nums.len() < 2 {
            return Err(format!("expected at least 2 coordinates, got '{}' in '{}'", part, s));
        }
        coords.push((nums[0], nums[1]));
    }
    if coords.is_empty() {
        return Err("no coordinate pairs found".into());
    }
    Ok(coords)
}

/// Project geographic coordinates (lon, lat) to pixel coordinates (px, py).
/// Uses simple linear scaling (equirectangular projection).
/// Returns (px, py) where py is flipped (SVG y-axis goes down).
pub fn project(x: f64, y: f64, bounds: (f64, f64, f64, f64), width: f64, height: f64, padding: f64) -> (f64, f64) {
    let (min_x, min_y, max_x, max_y) = bounds;
    let range_x = max_x - min_x;
    let range_y = max_y - min_y;

    if range_x == 0.0 && range_y == 0.0 {
        return (width / 2.0, height / 2.0);
    }

    // Scale independently to fill entire canvas (no aspect ratio preservation)
    let scale_x = (width - 2.0 * padding) / range_x.max(1e-10);
    let scale_y = (height - 2.0 * padding) / range_y.max(1e-10);

    let px = padding + (x - min_x) * scale_x;
    let py = padding + (max_y - y) * scale_y; // flip y
    (px, py)
}

/// Convert a geometry to an SVG path string.
pub fn geometry_to_svg_path(
    geom: &Geometry,
    bounds: (f64, f64, f64, f64),
    width: f64,
    height: f64,
    padding: f64,
) -> String {
    let mut path = String::new();

    match geom {
        Geometry::Point(x, y) => {
            let (px, py) = project(*x, *y, bounds, width, height, padding);
            // Draw a small circle
            let _ = write!(path, "M {} {} m -2,0 a 2,2 0 1,0 4,0 a 2,2 0 1,0 -4,0", px, py);
        }
        Geometry::MultiPoint(pts) => {
            for &(x, y) in pts {
                let (px, py) = project(x, y, bounds, width, height, padding);
                let _ = write!(path, "M {} {} m -2,0 a 2,2 0 1,0 4,0 a 2,2 0 1,0 -4,0 ", px, py);
            }
        }
        Geometry::LineString(pts) => {
            ring_to_path(&mut path, pts, bounds, width, height, padding, false);
        }
        Geometry::MultiLineString(lines) => {
            for line in lines {
                ring_to_path(&mut path, line, bounds, width, height, padding, false);
            }
        }
        Geometry::Polygon(rings) => {
            for (i, ring) in rings.iter().enumerate() {
                // First ring = outer (fill), subsequent = holes (evenodd)
                ring_to_path(&mut path, ring, bounds, width, height, padding, i == 0);
            }
        }
        Geometry::MultiPolygon(polys) => {
            for rings in polys {
                for ring in rings {
                    ring_to_path(&mut path, ring, bounds, width, height, padding, true);
                }
            }
        }
        Geometry::GeometryCollection(geoms) => {
            for g in geoms {
                path.push_str(&geometry_to_svg_path(g, bounds, width, height, padding));
            }
        }
    }

    path
}

/// Write a ring (closed polygon) as an SVG path.
fn ring_to_path(
    path: &mut String,
    ring: &[(f64, f64)],
    bounds: (f64, f64, f64, f64),
    width: f64,
    height: f64,
    padding: f64,
    close: bool,
) {
    if ring.is_empty() { return; }

    let (first_x, first_y) = project(ring[0].0, ring[0].1, bounds, width, height, padding);
    let _ = write!(path, "M {} {} ", first_x, first_y);

    for &(x, y) in &ring[1..] {
        let (px, py) = project(x, y, bounds, width, height, padding);
        let _ = write!(path, "L {} {} ", px, py);
    }

    if close {
        let _ = write!(path, "Z ");
    }
}

/// Compute the overall bounding box from multiple geometries.
pub fn compute_bounds(geometries: &[Geometry]) -> Option<(f64, f64, f64, f64)> {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for geom in geometries {
        if let Some((x0, y0, x1, y1)) = geom.bounds() {
            if x0 < min_x { min_x = x0; }
            if y0 < min_y { min_y = y0; }
            if x1 > max_x { max_x = x1; }
            if y1 > max_y { max_y = y1; }
        }
    }

    if min_x.is_infinite() { None } else { Some((min_x, min_y, max_x, max_y)) }
}
