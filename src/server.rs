use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use reqwest::Client;
use serde_json::{json, Value};

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GeocodeInput {
    /// Address or place name to geocode
    pub query: String,
    /// Max results (default 5)
    pub limit: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ReverseInput {
    /// Latitude
    pub lat: f64,
    /// Longitude
    pub lon: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RouteInput {
    /// Origin latitude
    pub origin_lat: f64,
    /// Origin longitude
    pub origin_lon: f64,
    /// Destination latitude
    pub dest_lat: f64,
    /// Destination longitude
    pub dest_lon: f64,
    /// Travel mode: driving, walking, cycling (default: driving)
    pub mode: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ElevationInput {
    /// Latitude
    pub lat: f64,
    /// Longitude
    pub lon: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PoiInput {
    /// Latitude of center point
    pub lat: f64,
    /// Longitude of center point
    pub lon: f64,
    /// POI type (hospital, restaurant, school, bank, pharmacy, fuel, hotel, supermarket, park, police)
    pub poi_type: String,
    /// Search radius in meters (default 2000)
    pub radius: Option<u32>,
    /// Max results (default 10)
    pub limit: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DistanceInput {
    /// List of coordinate pairs [[lat,lon], [lat,lon], ...] (2-25 points)
    pub points: Vec<[f64; 2]>,
    /// Travel mode: driving, walking, cycling (default: driving)
    pub mode: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TimezoneInput {
    /// Latitude
    pub lat: f64,
    /// Longitude
    pub lon: f64,
}

#[derive(Clone)]
pub struct MapsServer {
    pub client: Client,
}

impl MapsServer {
    pub fn new() -> Self {
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .http1_only()
            .user_agent("mcp-maps/1.0 (https://github.com/zavora-ai/mcp-maps)")
            .build()
            .unwrap_or_default();
        Self { client }
    }
    fn ua(&self) -> &'static str { "mcp-maps/1.0 (https://github.com/zavora-ai/mcp-maps)" }
}

#[tool_router(server_handler)]
impl MapsServer {
    #[tool(description = "Geocode an address or place name to coordinates (lat/lon). Supports worldwide locations.")]
    async fn geocode(&self, Parameters(input): Parameters<GeocodeInput>) -> String {
        let limit = input.limit.unwrap_or(5);
        let url = format!(
            "https://nominatim.openstreetmap.org/search?q={}&format=json&limit={}&addressdetails=1",
            urlencoding::encode(&input.query), limit
        );
        match self.client.get(&url).header("User-Agent", self.ua()).send().await {
            Ok(resp) => match resp.json::<Vec<Value>>().await {
                Ok(results) => {
                    let places: Vec<Value> = results.iter().map(|r| json!({
                        "lat": r["lat"].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                        "lon": r["lon"].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                        "display_name": r["display_name"],
                        "type": r["type"],
                        "importance": r["importance"],
                        "address": r["address"]
                    })).collect();
                    json!({"query": input.query, "results": places.len(), "places": places}).to_string()
                }
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Reverse geocode coordinates to an address/place name")]
    async fn reverse_geocode(&self, Parameters(input): Parameters<ReverseInput>) -> String {
        let url = format!(
            "https://nominatim.openstreetmap.org/reverse?lat={}&lon={}&format=json&addressdetails=1",
            input.lat, input.lon
        );
        match self.client.get(&url).header("User-Agent", self.ua()).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => json!({
                    "lat": input.lat, "lon": input.lon,
                    "display_name": data["display_name"],
                    "address": data["address"],
                    "osm_type": data["osm_type"],
                    "osm_id": data["osm_id"]
                }).to_string(),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Calculate route between two points with distance, duration, and turn-by-turn steps")]
    async fn get_route(&self, Parameters(input): Parameters<RouteInput>) -> String {
        let mode = input.mode.as_deref().unwrap_or("driving");
        let profile = match mode {
            "walking" | "foot" => "foot",
            "cycling" | "bike" => "bike",
            _ => "car",
        };
        let url_str = format!(
            "https://router.project-osrm.org/route/v1/{}/{},{};{},{}?overview=simplified&steps=true",
            profile, input.origin_lon, input.origin_lat, input.dest_lon, input.dest_lat
        );
        let url = reqwest::Url::parse(&url_str).unwrap();
        let request = self.client.get(url).header("User-Agent", self.ua());
        match request.send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => {
                    if let Some(route) = data["routes"].as_array().and_then(|r| r.first()) {
                        let steps: Vec<Value> = route["legs"][0]["steps"].as_array().unwrap_or(&vec![]).iter().map(|s| json!({
                            "instruction": s["maneuver"]["type"],
                            "modifier": s["maneuver"]["modifier"],
                            "name": s["name"],
                            "distance_m": s["distance"],
                            "duration_s": s["duration"]
                        })).collect();
                        json!({
                            "distance_km": route["distance"].as_f64().unwrap_or(0.0) / 1000.0,
                            "duration_min": route["duration"].as_f64().unwrap_or(0.0) / 60.0,
                            "mode": mode,
                            "steps": steps.len(),
                            "directions": steps
                        }).to_string()
                    } else {
                        json!({"error": "No route found", "code": data["code"]}).to_string()
                    }
                }
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get elevation (altitude) for a coordinate point")]
    async fn get_elevation(&self, Parameters(input): Parameters<ElevationInput>) -> String {
        let url = format!("https://api.opentopodata.org/v1/srtm90m?locations={},{}", input.lat, input.lon);
        match self.client.get(&url).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => {
                    let elev = data["results"][0]["elevation"].as_f64().unwrap_or(0.0);
                    json!({"lat": input.lat, "lon": input.lon, "elevation_m": elev, "dataset": "SRTM 90m"}).to_string()
                }
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Search for points of interest (POI) near a location. Types: hospital, restaurant, school, bank, pharmacy, fuel, hotel, supermarket, park, police, cafe, atm")]
    async fn search_poi(&self, Parameters(input): Parameters<PoiInput>) -> String {
        let radius = input.radius.unwrap_or(2000);
        let limit = input.limit.unwrap_or(10);
        let query = format!(
            "[out:json][timeout:15];node[\"amenity\"=\"{}\"](around:{},{},{});out {};",
            input.poi_type, radius, input.lat, input.lon, limit
        );
        let url = format!("https://overpass-api.de/api/interpreter?data={}", urlencoding::encode(&query));
        match self.client.get(&url).header("User-Agent", self.ua()).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => {
                    let pois: Vec<Value> = data["elements"].as_array().unwrap_or(&vec![]).iter().map(|e| {
                        let tags = e.get("tags").cloned().unwrap_or(json!({}));
                        json!({
                            "name": tags["name"],
                            "lat": e["lat"], "lon": e["lon"],
                            "phone": tags["phone"],
                            "website": tags["website"],
                            "address": tags.get("addr:street").map(|s| format!("{} {}", s.as_str().unwrap_or(""), tags.get("addr:housenumber").and_then(|h| h.as_str()).unwrap_or("")))
                        })
                    }).collect();
                    json!({"poi_type": input.poi_type, "radius_m": radius, "results": pois.len(), "pois": pois}).to_string()
                }
                Err(_) => json!({"poi_type": input.poi_type, "results": 0, "pois": [], "note": "Overpass API may be rate-limited. Retry in a few seconds."}).to_string(),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Calculate distance matrix between multiple points (driving/walking/cycling)")]
    async fn distance_matrix(&self, Parameters(input): Parameters<DistanceInput>) -> String {
        let mode = input.mode.as_deref().unwrap_or("driving");
        let profile = match mode { "walking" | "foot" => "foot", "cycling" | "bike" => "bike", _ => "car" };
        let coords: Vec<String> = input.points.iter().map(|p| format!("{},{}", p[1], p[0])).collect();
        let url = format!(
            "https://router.project-osrm.org/table/v1/{}/{}?annotations=distance,duration",
            profile, coords.join(";")
        );
        match self.client.get(&url).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => json!({
                    "mode": mode,
                    "points": input.points.len(),
                    "durations_seconds": data["durations"],
                    "distances_meters": data["distances"]
                }).to_string(),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get timezone for a coordinate (uses Nominatim address data)")]
    async fn get_timezone(&self, Parameters(input): Parameters<TimezoneInput>) -> String {
        let url = format!(
            "https://nominatim.openstreetmap.org/reverse?lat={}&lon={}&format=json&zoom=3",
            input.lat, input.lon
        );
        match self.client.get(&url).header("User-Agent", self.ua()).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => json!({
                    "lat": input.lat, "lon": input.lon,
                    "country": data["address"]["country"],
                    "country_code": data["address"]["country_code"],
                    "display_name": data["display_name"]
                }).to_string(),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }
}
