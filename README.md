# Maps & Geolocation MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-maps.svg)](https://crates.io/crates/mcp-maps)
[![Docs.rs](https://docs.rs/mcp-maps/badge.svg)](https://docs.rs/mcp-maps)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)
[![Registry Ready](https://img.shields.io/badge/ADK_Registry-Ready-green.svg)](https://enterprise.adk-rust.com)

Geolocation and routing for [ADK-Rust Enterprise](https://enterprise.adk-rust.com) agents. Provides 7 MCP tools for geocoding, routing, elevation, POI search, and distance matrices — **zero configuration, no API keys, powered by OpenStreetMap**.

## Architecture

<p align="center">
  <img src="https://raw.githubusercontent.com/zavora-ai/mcp-maps/main/docs/assets/architecture.svg" alt="Maps MCP Architecture" width="800"/>
</p>

## Key Principles

- **Zero configuration** — works out of the box with no API keys or environment variables.
- **Global coverage** — OpenStreetMap data covers every country on Earth.
- **Multiple transport modes** — driving, walking, and cycling routes.
- **Turn-by-turn directions** — full step-by-step navigation instructions.
- **POI discovery** — find hospitals, restaurants, ATMs, and more near any point.
- **Registry-ready** — ships with `mcp-server.toml` for automatic ADK-Rust Enterprise onboarding.

## Tools

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `geocode` | Convert address/place name to coordinates | Read-only |
| `reverse_geocode` | Convert coordinates to address | Read-only |
| `get_route` | Calculate route with turn-by-turn directions | Read-only |
| `get_elevation` | Get altitude for any coordinate | Read-only |
| `search_poi` | Find points of interest near a location | Read-only |
| `distance_matrix` | Distance/duration between multiple points | Read-only |
| `get_timezone` | Get country/timezone for coordinates | Read-only |

## Backends

| Backend | Purpose | Rate Limit |
|---------|---------|-----------|
| Nominatim (OpenStreetMap) | Geocoding & reverse geocoding | 1 req/sec |
| OSRM | Routing & distance matrices | Unlimited (demo server) |
| OpenTopoData | Elevation data (SRTM 90m) | 1 req/sec |
| Overpass API | POI search | 2 req/10sec |

## Installation

### From crates.io

```bash
cargo install mcp-maps
```

### Build from source

```bash
git clone https://github.com/zavora-ai/mcp-maps
cd mcp-maps
cargo build --release
```

### Claude Desktop

```json
{
  "mcpServers": {
    "maps": { "command": "mcp-maps" }
  }
}
```

### Kiro

Add to `.kiro/settings/mcp.json`:

```json
{
  "mcpServers": {
    "maps": { "command": "mcp-maps" }
  }
}
```

### Cursor

Add to `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "maps": { "command": "mcp-maps" }
  }
}
```

## Quick Start

### Geocode an address

```json
{
  "name": "geocode",
  "arguments": { "query": "Eiffel Tower Paris", "limit": 2 }
}
```

**Response:**

```json
{
  "query": "Eiffel Tower Paris",
  "results": 1,
  "places": [
    {
      "lat": 48.8582599,
      "lon": 2.2945006,
      "display_name": "Tour Eiffel, 5, Avenue Anatole France, Quartier du Gros-Caillou, Paris 7e, Paris, Île-de-France, France",
      "type": "attraction",
      "importance": 0.82,
      "address": { "tourism": "Tour Eiffel", "city": "Paris", "country": "France" }
    }
  ]
}
```

### Calculate a route

```json
{
  "name": "get_route",
  "arguments": {
    "origin_lat": -1.2921, "origin_lon": 36.8219,
    "dest_lat": -1.1631, "dest_lon": 36.9519,
    "mode": "driving"
  }
}
```

**Response:**

```json
{
  "distance_km": 23.6,
  "duration_min": 21,
  "mode": "driving",
  "steps": 22,
  "directions": [
    { "instruction": "depart", "name": "Haile Selassie Avenue", "distance_m": 683 },
    { "instruction": "rotary", "modifier": "slight left", "name": "Haile Selassie Avenue", "distance_m": 57 },
    { "instruction": "turn", "modifier": "right", "name": "Mombasa Road", "distance_m": 4200 }
  ]
}
```

### Get elevation

```json
{
  "name": "get_elevation",
  "arguments": { "lat": -1.2921, "lon": 36.8219 }
}
```

**Response:**

```json
{
  "lat": -1.2921,
  "lon": 36.8219,
  "elevation_m": 1663.0,
  "dataset": "SRTM 90m"
}
```

### Search for nearby POIs

```json
{
  "name": "search_poi",
  "arguments": { "lat": 48.8582, "lon": 2.2945, "poi_type": "restaurant", "radius": 500, "limit": 5 }
}
```

### Distance matrix

```json
{
  "name": "distance_matrix",
  "arguments": {
    "points": [[-1.2921, 36.8219], [-1.1631, 36.9519], [-1.3028, 36.7073]],
    "mode": "driving"
  }
}
```

## POI Types

| Type | Description | Type | Description |
|------|-------------|------|-------------|
| `hospital` | Hospitals & clinics | `restaurant` | Restaurants |
| `school` | Schools & universities | `bank` | Banks |
| `pharmacy` | Pharmacies | `fuel` | Petrol/gas stations |
| `hotel` | Hotels & lodging | `supermarket` | Supermarkets |
| `park` | Parks & gardens | `police` | Police stations |
| `cafe` | Cafés & coffee shops | `atm` | ATMs |

## Transport Modes

| Mode | OSRM Profile | Use Case |
|------|-------------|----------|
| `driving` (default) | car | Vehicle navigation |
| `walking` / `foot` | foot | Pedestrian routes |
| `cycling` / `bike` | bike | Bicycle routes |

## Configuration

### Environment Variables

| Variable | Required | Purpose |
|----------|:--------:|---------|
| `RUST_LOG` | No | Log level (default: `info`) |

No API keys needed. All backends are free and public.

### MCP Server Manifest

```toml
server_id = "mcp_maps"
display_name = "Maps"
version = "1.0.0"
domain = "geolocation"
risk_level = "low"
writes_allowed = "none"
```

## Use Cases

### Ride-hailing / Delivery
```
Customer provides address → geocode → get coordinates
Calculate route → get_route → distance + ETA
Find nearby drivers → distance_matrix → closest available
```

### Real Estate
```
Property address → geocode → coordinates
→ search_poi (school, hospital, supermarket) → walkability score
→ get_elevation → flood risk assessment
```

### Logistics & Fleet
```
Warehouse + delivery points → distance_matrix → optimal ordering
Each leg → get_route → turn-by-turn for drivers
```

### Travel Planning
```
Destination → geocode → coordinates
→ search_poi (hotel, restaurant, attraction) → itinerary
→ get_route between stops → total trip time
```

## Documentation

| Document | Description |
|----------|-------------|
| [mcp-server.toml](mcp-server.toml) | ADK-Rust Enterprise registry manifest |
| [Rust Docs](https://docs.rs/mcp-maps) | Generated API documentation |

## Contributing

Contributions welcome. Priority areas:
- Isochrone calculations (reachable area within X minutes)
- Geocoding result ranking improvements
- Additional POI categories
- Traffic-aware routing (when OSRM supports it)

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START -->
| [<img src="https://github.com/jkmaina.png" width="80px;" alt=""/><br /><sub><b>James Karanja Maina</b></sub>](https://github.com/jkmaina) |
|:---:|
<!-- ALL-CONTRIBUTORS-LIST:END -->

## License

Apache-2.0 — see [LICENSE](LICENSE) for details.

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.

Built with ❤️ by [Zavora AI](https://zavora.ai)

## Registry Compliance

This server implements the [ADK MCP SDK](https://crates.io/crates/adk-mcp-sdk) contract:

- **HealthCheck** — async health probe for registry monitoring
- **mcp-server.toml** — manifest declaring tools, risk classes, and credentials
- **Structured tracing** — `RUST_LOG` env-filter for observability
