use crate::model::app_state::QuickCache;

use super::cubs_model::{ModelData, ModelVersionNumber};
use anyhow::anyhow;
use flate2::bufread::GzDecoder;
use std::error::Error;
use std::time::Instant;
use std::io::Read;
use uuid::Uuid;
#[derive(Debug, sqlx::FromRow)]
struct SavedModel {
    pub model_id: String,
    pub vers_no: i32,
    pub saved_gzip: Vec<u8>,
}

// pub fn read_model_response_from_file<P>(path: P) -> Result<ModelResponse, Box<dyn Error>>
// where
//     P: AsRef<Path>,
// {
//     //Open the fie in read-only model with buffer
//     let file = File::open(path).expect("Should have been able to open the input file");
//     let reader = BufReader::new(file);

//     //Read the JSON contents of the file as Model Response
//     let model_respose = serde_json::from_reader(reader)?;

//     Ok(model_respose)
// }

// pub fn read_model_data_from_file<P>(path: P) -> Result<ModelData, Box<dyn Error>>
// where
//     P: AsRef<Path>,
// {
//     //Open the fie in read-only model with buffer
//     let file = File::open(path).expect("Should have been able to open the input file");
//     let reader = BufReader::new(file);

//     //Read the JSON contents of the file as Model Response
//     let result = serde_json::from_reader(reader)?;

//     Ok(result)
// }

// async fn read_latest_model_data_from_db(
//     pg_pool: &sqlx::Pool<sqlx::Postgres>,
//     model_id: &String,
//     cache: &QuickCache<ModelData>,
// ) -> Result<ModelData, Box<dyn Error>> {
//     let start_time = Instant::now();

//     println!(
//         "[read_model_data_from_db] Retrieving {} model from DB...",
//         &model_id
//     );

//     // Retrieve from DB
//     println!("[read_model_data_from_db] Retreiving from DB ...");
//     let saved_model = sqlx::query_as!(
//         SavedModel,
//         r#"SELECT model_id, vers_no, saved_gzip FROM cubs_object_model.saved_model WHERE model_id = $1 ORDER BY vers_no DESC
// LIMIT 1"#,
//         model_id
//     )
//     .fetch_one(pg_pool)
//     .await?;
//     println!(
//         "[read_model_data_from_db]  Load saved model with model id: {} version: {} from DB",
//         saved_model.model_id, saved_model.vers_no
//     );

//     // Unzip
//     println!("[read_model_data_from_db] Unzip ...");
//     let decompressed_model = decompress_gzip_to_string(&saved_model.saved_gzip)?;

//     //Convert to ModelData
//     println!("[read_model_data_from_db] Convert to internal format ...");
//     let model_data: ModelData = serde_json::from_str(&decompressed_model)?;

//     // Store in cache
//     let key = model_id.clone() + "_" + &model_data.version.to_string();
//     println!("[read_model_data_from_db] Cache Model data key: {}", key);
//     cache.insert(
//         &model_id.clone(),
//         &model_data.version.to_string(),
//         &model_data,
//     );

//     //Log time
//     let elapsed_time = start_time.elapsed();
//     println!(
//         "[Execution time] {} - {:?}",
//         "read_model_data_from_db + cache", elapsed_time
//     );

//     Ok(model_data)
// }

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

pub async fn read_model_data(
    pg_pool: &sqlx::Pool<sqlx::Postgres>,
    cache: &QuickCache<ModelData>,
    model_id: &String,
    version_num: i32,
) -> Result<ModelData, Box<dyn Error>> {
    let start_time = Instant::now();

    println!(
        "[read_model_data] Retrievig {} model with version {:?} ...",
        &model_id, version_num
    );

    // Check format
    match Uuid::parse_str(&model_id) {
        Ok(_) => (),
        Err(e) => {
            eprintln!(
                "[read_model_data] Error parsing uuid string {}",
                e.to_string()
            );
            return Err(anyhow!("Model id is not uuid").into());
        }
    }

    // Get from cache
    if let Some(cached_model_data) = cache.get(&model_id.clone(), &version_num.to_string()) {
        println!("[read_model_data] Found model data {} wiht version {} in cache", model_id, version_num);

        let elapsed_time = start_time.elapsed();
        println!(
            "[Execution time] {} - {:?}",
            "read_model_data_from_cache", elapsed_time
        );

        return Ok(cached_model_data);
    }

    // Get from DB
    let model_data =
        read_model_data_from_db_with_version(pg_pool, model_id, version_num, &cache).await;

    //Log time
    let elapsed_time = start_time.elapsed();
    println!(
        "[Execution time] {} - {:?}",
        "read_model_data", elapsed_time
    );

    model_data
}

fn decompress_gzip_to_string(gzip: &Vec<u8>) -> Result<String, Box<dyn Error>> {
    let mut decoder = GzDecoder::new(gzip.as_slice());
    let mut decompressed_data = String::new();
    decoder.read_to_string(&mut decompressed_data)?;
    Ok(decompressed_data)
}
