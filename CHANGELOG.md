# Changelog

## [1.0.0] - 2026-05-27

### Added
- `geocode` — convert address/place name to coordinates (Nominatim)
- `reverse_geocode` — convert coordinates to address (Nominatim)
- `get_route` — calculate route with turn-by-turn directions (OSRM)
- `get_elevation` — get altitude for any coordinate (OpenTopoData SRTM 90m)
- `search_poi` — find points of interest near a location (Overpass API)
- `distance_matrix` — distance/duration between multiple points (OSRM)
- `get_timezone` — get country/timezone for coordinates (Nominatim)
- Support for driving, walking, and cycling transport modes
- Global coverage via OpenStreetMap data
- Zero configuration — no API keys required
- ADK-Rust Enterprise registry manifest (`mcp-server.toml`)
