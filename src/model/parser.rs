use crate::model::app_state::QuickCache;

use super::cubs_model::{ModelData, ModelVersionNumber};
use anyhow::anyhow;
use flate2::bufread::GzDecoder;
use std::error::Error;
use std::sync::Arc;
use std::time::Instant;
use std::io::Read;
use uuid::Uuid;
#[derive(Debug, sqlx::FromRow)]
struct SavedModel {
    pub model_id: String,
    pub vers_no: i32,
    pub saved_gzip: Vec<u8>,
}

pub async fn read_model_data_versions(
    pg_pool: &sqlx::Pool<sqlx::Postgres>,
    model_id: &String,
) -> Result<Vec<ModelVersionNumber>, Box<dyn Error>> {
    let start_time = Instant::now();

    println!(
        "[read_model_data_versions] Retrieving {} model version from DB...",
        &model_id
    );

    // Retrieve from DB
    let model_versions = sqlx::query_as!(
        ModelVersionNumber,
        r#"SELECT vers_no FROM cubs_object_model.saved_model WHERE model_id = $1 ORDER BY vers_no DESC"#,
        model_id
    )
    .fetch_all(pg_pool)
    .await?;

    //Log time
    let elapsed_time = start_time.elapsed();
    println!(
        "[Execution time] {} - {:?}",
        "read_model_data_versions", elapsed_time
    );

    Ok(model_versions)
}

async fn read_model_data_from_db_with_version(
    pg_pool: &sqlx::Pool<sqlx::Postgres>,
    model_id: &String,
    version_no: i32,
    cache: &QuickCache<ModelData>,
) -> Result<ModelData, Box<dyn Error>> {
    let start_time = Instant::now();

    println!(
        "[read_model_data_from_db_with_version] Retrieving {} model from DB...",
        &model_id
    );

    // Retrieve from DB
    println!("[read_model_data_from_db_with_version] Retreiving from DB ...");
    let saved_model = sqlx::query_as!(
        SavedModel,
        r#"SELECT model_id, vers_no, saved_gzip FROM cubs_object_model.saved_model WHERE model_id = $1 and vers_no = $2"#,
        model_id, version_no
    )
    .fetch_one(pg_pool)
    .await?;
    println!(
        "[read_model_data_from_db_with_version]  Load saved model with model id: {} version: {} from DB",
        saved_model.model_id, saved_model.vers_no
    );

    // Unzip
    println!("[read_model_data_from_db_with_version] Unzip ...");
    let decompressed_model = decompress_gzip_to_string(&saved_model.saved_gzip)?;

    //Convert to ModelData
    println!("[read_model_data_from_db_with_version] Convert to internal format ...");
    let model_data: ModelData = serde_json::from_str(&decompressed_model)?;

    // Store in cache
    let key = model_id.clone() + "_" + &model_data.version.to_string();
    println!(
        "[read_model_data_from_db_with_version] Cache Model data key: {}",
        key
    );
    cache.insert(
        &model_id.clone(),
        &model_data.version.to_string(),
        &model_data,
    );

    //Log time
    let elapsed_time = start_time.elapsed();
    println!(
        "[Execution time] {} - {:?}",
        "read_model_data_from_db_with_version + cache", elapsed_time
    );

    Ok(model_data)
}

pub fn get_model_from_cache(
    cache: &QuickCache<ModelData>,
    model_id: &String,
    version_num: i32,
) -> Option<Arc<ModelData>> {
    let start_time = Instant::now();

    println!(
        "[get_model_from_cache] Retrievig {} model with version {:?} from cache.",
        &model_id, version_num
    );

    let cached_model_data = cache.get_ref(&model_id.clone(), &version_num.to_string());
    //Log time
    let elapsed_time = start_time.elapsed();
    println!(
        "[Execution time] {} - {:?}",
        "get_model_from_cache", elapsed_time
    );

    cached_model_data
}

pub async fn get_model_from_db(
    pg_pool: &sqlx::Pool<sqlx::Postgres>,
    cache: &QuickCache<ModelData>,
    model_id: &String,
    version_num: i32,
) -> Option<Arc<ModelData>> {
    let start_time = Instant::now();

    // G_model_data
    let _model_data =
        read_model_data_from_db_with_version(pg_pool, model_id, version_num, &cache).await;

    // Get reference from cache
    let cached_model_data = cache.get_ref(&model_id.clone(), &version_num.to_string());

    //Log time
    let elapsed_time = start_time.elapsed();
    println!(
        "[Execution time] {} - {:?}",
        "read_model_data", elapsed_time
    );

    cached_model_data
}

fn decompress_gzip_to_string(gzip: &Vec<u8>) -> Result<String, Box<dyn Error>> {
    let mut decoder = GzDecoder::new(gzip.as_slice());
    let mut decompressed_data = String::new();
    decoder.read_to_string(&mut decompressed_data)?;
    Ok(decompressed_data)
}
