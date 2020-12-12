use crate::CensusArea;
use abstutil::Timer;
use geojson::GeoJson;
use map_model::Map;
use std::convert::TryFrom;

impl CensusArea {
    pub fn find_data_for_map(map: &Map, timer: &mut Timer) -> Result<Vec<CensusArea>, String> {
        // TODO eventually it'd be nice to lazily download the info needed. For now we expect a
        // prepared geojson file to exist in data/system/<city>/population_areas.geojson
        //
        // When we implement downloading, importer/src/utils.rs has a download() helper that we
        // could copy here. (And later dedupe, after deciding how this crate will integrate with
        // the importer)
        let path = abstutil::path(format!(
            "system/{}/population_areas.geojson",
            map.get_name().city
        ));
        let bytes = abstutil::slurp_file(&path)?;
        debug!("parsing geojson at path: {}", &path);

        // TODO - can we change to Result<_,Box<dyn std::error::Error>> and avoid all these map_err?
        let str = String::from_utf8(bytes).map_err(|e| e.to_string())?;
        timer.start("parsing geojson");
        let geojson = str.parse::<GeoJson>().map_err(|e| e.to_string())?;
        timer.stop("parsing geojson");
        let mut results = vec![];
        let collection =
            geojson::FeatureCollection::try_from(geojson).map_err(|e| e.to_string())?;

        debug!("collection.features: {}", &collection.features.len());
        timer.start("converting to `CensusArea`s");
        for feature in collection.features.into_iter() {
            let population = feature.property("population");
            let total_population = match population {
                Some(serde_json::Value::Number(n)) => n
                    .as_u64()
                    .expect(&format!("unexpected total population number: {:?}", n))
                    as usize,
                _ => {
                    return Err(format!(
                        "unexpected format for 'population': {:?}",
                        population
                    ));
                }
            };

            let geometry = feature.geometry.expect("geojson feature missing geometry");
            let mut multi_poly =
                geo::MultiPolygon::<f64>::try_from(geometry.value).map_err(|e| e.to_string())?;
            let geo_polygon = multi_poly
                .0
                .pop()
                .expect("multipolygon was unexpectedly empty");
            if !multi_poly.0.is_empty() {
                // Annoyingly upstream GIS has packaged all these individual polygons into
                // "multipolygon" of length 1 Make sure nothing surprising is
                // happening since we only use the first poly
                error!(
                    "unexpectedly had {} extra area polygons",
                    multi_poly.0.len()
                );
            }

            let polygon = geom::Polygon::from(geo_polygon);
            results.push(CensusArea {
                polygon,
                total_population,
            });
        }
        timer.stop("converting to `CensusArea`s");
        Ok(results)
    }
}
