#![allow(clippy::not_unsafe_ptr_arg_deref)]
//! haygeom - Geographic mapping plugin for Hayashi
//!
//! Provides functionality for rendering geographic maps from WKT geometry data.
//! Supports multiple layers (polygons, points, lines) with independent styling.

use hayashi_plugin_sdk::{hayashi_fn, hayashi_plugin};
use hayashi_plugin_sdk::arrow::array::{Array, ArrayRef, StructArray};
use hayashi_plugin_sdk::value::HayashiValue;
use std::collections::HashMap;

// Exposes dynamic library C ABI deallocation hooks
hayashi_plugin!();

mod wkt;



/// Create a new map with specified dimensions
#[hayashi_fn]
pub fn map(
    width: f64,
    height: f64
) -> HashMap<String, HayashiValue> {
    let mut map_dict = HashMap::new();
    map_dict.insert("width".to_string(), HayashiValue::Float(width));
    map_dict.insert("height".to_string(), HayashiValue::Float(height));
    map_dict.insert("background".to_string(), HayashiValue::Str("white".to_string()));
    map_dict.insert("layers".to_string(), HayashiValue::List(vec![]));

    map_dict
}

/// Add a layer to the map
#[hayashi_fn]
pub fn add_layer(
    mut map: HashMap<String, HayashiValue>,
    data: ArrayRef,
    options: HashMap<String, HayashiValue>
) -> HashMap<String, HayashiValue> {

    // Parse options
    let fill = options.get("fill")
        .and_then(|v| match v {
            HayashiValue::Str(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| "#2D3E50".to_string());

    let stroke = options.get("stroke")
        .and_then(|v| match v {
            HayashiValue::Str(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| "none".to_string());

    let stroke_width = options.get("stroke_width")
        .and_then(|v| match v {
            HayashiValue::Float(f) => Some(*f),
            HayashiValue::Int(i) => Some(*i as f64),
            _ => None,
        })
        .unwrap_or(0.5);

    let point_size = options.get("size")
        .and_then(|v| match v {
            HayashiValue::Float(f) => Some(*f),
            HayashiValue::Int(i) => Some(*i as f64),
            _ => None,
        })
        .unwrap_or(5.0);

    // Parse geometries from data
    let struct_arr = data.as_any()
        .downcast_ref::<StructArray>();

    let geom_strings = if let Some(arr) = struct_arr {
        let geom_col_name = "geometry";
        let geom_col_idx = arr.fields().iter()
            .position(|f| f.name() == geom_col_name);

        if let Some(idx) = geom_col_idx {
            extract_column_string(arr, idx).unwrap_or_else(|_| vec![])
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    // Add to layers list - store layer as serialized data for now
    let layers = map.get("layers")
        .and_then(|v| match v {
            HayashiValue::List(l) => Some(l.clone()),
            _ => None,
        })
        .unwrap_or_default();

    let mut layer_dict = HashMap::new();
    layer_dict.insert("fill".to_string(), HayashiValue::Str(fill.clone()));
    layer_dict.insert("stroke".to_string(), HayashiValue::Str(stroke.clone()));
    layer_dict.insert("stroke_width".to_string(), HayashiValue::Float(stroke_width));
    layer_dict.insert("size".to_string(), HayashiValue::Float(point_size));
    
    // Store WKT strings for later parsing in render
    // Serialize as JSON string to avoid Arrow conversion
    let wkt_json = serde_json::to_string(&geom_strings).unwrap_or_else(|_| "[]".to_string());
    layer_dict.insert("wkt_strings_json".to_string(), HayashiValue::Str(wkt_json));
    
    let mut new_layers = layers;
    new_layers.push(HayashiValue::Dict(layer_dict));

    // Return updated map
    map.insert("layers".to_string(), HayashiValue::List(new_layers));

    map
}

/// Render the map to SVG
#[hayashi_fn]
pub fn render(
    map: HashMap<String, HayashiValue>
) -> String {
    use wkt::{parse_wkt, geometry_to_svg_path, compute_bounds};

    let width = map.get("width")
        .and_then(|v| match v {
            HayashiValue::Float(f) => Some(*f),
            HayashiValue::Int(i) => Some(*i as f64),
            _ => None,
        })
        .unwrap_or(800.0);

    let height = map.get("height")
        .and_then(|v| match v {
            HayashiValue::Float(f) => Some(*f),
            HayashiValue::Int(i) => Some(*i as f64),
            _ => None,
        })
        .unwrap_or(600.0);

    let background = map.get("background")
        .and_then(|v| match v {
            HayashiValue::Str(s) => Some(s.as_str()),
            _ => None,
        })
        .unwrap_or("white");

    // Collect all geometries from all layers
    let mut all_geometries: Vec<wkt::Geometry> = Vec::new();
    let mut layer_configs: Vec<(String, String, f64, f64)> = Vec::new(); // (fill, stroke, stroke_width, point_size)

    if let Some(HayashiValue::List(layers)) = map.get("layers") {
        for layer in layers {
            if let HayashiValue::Dict(d) = layer {
                let fill = d.get("fill")
                    .and_then(|v| match v {
                        HayashiValue::Str(s) => Some(s.clone()),
                        _ => None,
                    })
                    .unwrap_or_else(|| "#2D3E50".to_string());

                let stroke = d.get("stroke")
                    .and_then(|v| match v {
                        HayashiValue::Str(s) => Some(s.clone()),
                        _ => None,
                    })
                    .unwrap_or_else(|| "none".to_string());

                let stroke_width = d.get("stroke_width")
                    .and_then(|v| match v {
                        HayashiValue::Float(f) => Some(*f),
                        HayashiValue::Int(i) => Some(*i as f64),
                        _ => None,
                    })
                    .unwrap_or(0.5);

                let point_size = d.get("size")
                    .and_then(|v| match v {
                        HayashiValue::Float(f) => Some(*f),
                        HayashiValue::Int(i) => Some(*i as f64),
                        _ => None,
                    })
                    .unwrap_or(5.0);

                // Parse WKT strings from JSON
                if let Some(HayashiValue::Str(wkt_json)) = d.get("wkt_strings_json") {
                    if let Ok(geom_strings) = serde_json::from_str::<Vec<String>>(wkt_json) {
                        for wkt in geom_strings {
                            if let Ok(g) = parse_wkt(&wkt) {
                                all_geometries.push(g);
                                layer_configs.push((fill.clone(), stroke.clone(), stroke_width, point_size));
                            }
                        }
                    }
                }
            }
        }
    }

    if all_geometries.is_empty() {
        // Return empty SVG if no geometries
        let mut svg = format!(r#"<svg width="{}" height="{}" viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">"#, width, height, width, height);
        svg = format!(r#"{}<rect width="100%" height="100%" fill="{}"/>"#, svg, background);
        svg.push_str("</svg>");
        return svg;
    }

    // Compute overall bounds
    let bounds = compute_bounds(&all_geometries).unwrap_or((0.0, 0.0, 10.0, 10.0));

    let padding = 0.0;

    // Build SVG
    let mut svg = format!(r#"<svg width="{}" height="{}" viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">"#, width, height, width, height);
    svg = format!(r#"{}<rect width="100%" height="100%" fill="{}"/>"#, svg, background);

    // Render each geometry
    for (i, geom) in all_geometries.iter().enumerate() {
        let (fill, stroke, stroke_width, _point_size) = layer_configs.get(i)
            .cloned()
            .unwrap_or_else(|| ("#2D3E50".to_string(), "none".to_string(), 0.5, 5.0));

        let path_d = geometry_to_svg_path(geom, bounds, width, height, padding);

        let stroke_attr = if stroke == "none" {
            "stroke=\"none\"".to_string()
        } else {
            format!("stroke=\"{}\" stroke-width=\"{}\"", stroke, stroke_width)
        };

        svg = format!(r#"{}<path d="{}" fill="{}" {}/>"#, svg, path_d, fill, stroke_attr);
    }

    svg.push_str("</svg>");

    svg
}

#[cfg(test)]
mod tests {
    use super::wkt::{parse_wkt, compute_bounds};

    // ── parse_wkt ─────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_wkt_point() {
        let g = parse_wkt("POINT(-43.5 -22.9)").unwrap();
        match g {
            super::wkt::Geometry::Point(x, y) => {
                assert!((x - (-43.5)).abs() < 1e-9);
                assert!((y - (-22.9)).abs() < 1e-9);
            }
            _ => panic!("esperado Point"),
        }
    }

    #[test]
    fn test_parse_wkt_linestring() {
        let g = parse_wkt("LINESTRING(0 0, 1 1, 2 0)").unwrap();
        match g {
            super::wkt::Geometry::LineString(pts) => {
                assert_eq!(pts.len(), 3);
                assert_eq!(pts[0], (0.0, 0.0));
                assert_eq!(pts[2], (2.0, 0.0));
            }
            _ => panic!("esperado LineString"),
        }
    }

    #[test]
    fn test_parse_wkt_polygon() {
        let wkt = "POLYGON((0 0, 1 0, 1 1, 0 1, 0 0))";
        let g = parse_wkt(wkt).unwrap();
        match g {
            super::wkt::Geometry::Polygon(rings) => {
                assert_eq!(rings.len(), 1);
                assert_eq!(rings[0].len(), 5);
            }
            _ => panic!("esperado Polygon"),
        }
    }

    #[test]
    fn test_parse_wkt_empty_string() {
        assert!(parse_wkt("").is_err());
    }

    #[test]
    fn test_parse_wkt_invalid() {
        assert!(parse_wkt("NOTAGEOMETRY(0 0)").is_err());
    }

    // ── compute_bounds ────────────────────────────────────────────────────────

    #[test]
    fn test_compute_bounds_point() {
        let g = parse_wkt("POINT(10.0 20.0)").unwrap();
        let b = compute_bounds(&[g]).unwrap();
        assert_eq!(b, (10.0, 20.0, 10.0, 20.0));
    }

    #[test]
    fn test_compute_bounds_multipoint() {
        let g1 = parse_wkt("POINT(0 0)").unwrap();
        let g2 = parse_wkt("POINT(5 -3)").unwrap();
        let b = compute_bounds(&[g1, g2]).unwrap();
        // min_x=0, min_y=-3, max_x=5, max_y=0
        assert_eq!(b.0, 0.0);
        assert_eq!(b.1, -3.0);
        assert_eq!(b.2, 5.0);
        assert_eq!(b.3, 0.0);
    }

    #[test]
    fn test_compute_bounds_empty() {
        assert_eq!(compute_bounds(&[]), None);
    }

    // ── __hayashi_impl_map ────────────────────────────────────────────────────

    #[test]
    fn test_map_creates_dict() {
        use hayashi_plugin_sdk::value::HayashiValue;
        let m = super::__hayashi_impl_map(800.0, 600.0);
        assert_eq!(m.get("width"),  Some(&HayashiValue::Float(800.0)));
        assert_eq!(m.get("height"), Some(&HayashiValue::Float(600.0)));
        match m.get("layers") {
            Some(HayashiValue::List(l)) => assert!(l.is_empty()),
            _ => panic!("esperado layers = lista vazia"),
        }
    }
}

/// Extract a string column from an Arrow StructArray
fn extract_column_string(struct_arr: &StructArray, col_idx: usize) -> Result<Vec<String>, String> {
    let col = struct_arr.column(col_idx);
    let string_array = col.as_any()
        .downcast_ref::<hayashi_plugin_sdk::arrow::array::StringArray>()
        .ok_or_else(|| "Column must be a StringArray".to_string())?;

    string_array.iter()
        .map(|opt: Option<&str>| opt.ok_or_else(|| "null value in column".to_string()).map(|s| s.to_string()))
        .collect()
}
