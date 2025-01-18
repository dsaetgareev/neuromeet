use std::{collections::HashMap, fs::File, io::{BufReader, Cursor, Read, Write}, path::PathBuf, process::Command, str::FromStr, sync::Arc};

use futures::StreamExt;
use object_store::{aws::AmazonS3Builder, path::Path, ObjectStore};
use ogg::PacketReader;
use rdkafka::{message::{BorrowedHeaders, Headers}, Message};
use sec_api::kafka::{kafka_consumer::KafkaConsumer, SystemEvent};
use tracing::{error, info};
use dotenv::dotenv;


const SYSTEM_TOPIC_NAME: &str = "system_events";

#[derive(Debug)]
pub enum ConsumerError {
    Error
}

pub struct Unit {
    topic_key: String,
    is_leaved: bool,
}

pub struct Room {
    units: Vec<Unit>,
    unit_count: u32,
}

impl Room {
    pub fn new() -> Self {
        let units = Vec::new();
        Self {
            units,
            unit_count: 0u32,
        }
    }

    pub fn add_unit(&mut self, topic_key: &str) {
        self.units.push(Unit { topic_key: topic_key.to_string(), is_leaved: false });
        self.unit_count += 1;
    }

    pub fn leave_unit(&mut self, topic_key: &str) -> Result<(), ()> {
        self.units
        .iter_mut()
        .for_each(|unit| {
            if unit.topic_key.eq(topic_key) && !unit.is_leaved {
                unit.is_leaved = true;
            }
        });
        self.unit_count -= 1;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), ()> {

    dotenv().ok();

    tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .compact()
    .with_file(true)
    .with_line_number(true)
    .with_thread_ids(true)
    .with_target(false)
    .init();

    let mut  system_consumer = KafkaConsumer::new();

    let mut rooms: HashMap<String, Room> = HashMap::new();

    if let Ok(system_consumer) = system_consumer.create_consumer(SYSTEM_TOPIC_NAME, "file_system_group").await {
        while let Some(message) = system_consumer.stream().next().await {
            match message {
                Ok(msg) => {
                    if let Some(payload) = msg.payload() {
                        let payload_str = std::str::from_utf8(payload).unwrap();
                        let headers = msg.headers().expect("cannot get headers");
                        let headers = headers_to_map(headers);
                        let group_id = headers.get("key")
                            .expect("cannot get a key")
                            .expect("cannot get a group_id");
                        let group_id = std::str::from_utf8(group_id)
                            .expect("cannot parse a group_id from &[u8]");
                        let group_id = String::from(group_id);
                        let topic_name = headers.get("topic_name")
                            .expect("cannot get a topic_name")
                            .expect("cannot get a topic_name");
                        let topic_name = std::str::from_utf8(topic_name)
                            .expect("cannot parse a topic_name from &[u8]");
                        let topic_name = String::from(topic_name);
                        println!("Получено сообщение: {}, {}", payload_str, group_id);
                        let system_event = SystemEvent::from_str(payload_str).expect("cannot parsing SystemEvent");
                        let room = rooms.get_mut(&topic_name);

                        match system_event {
                            SystemEvent::Create => {
                                match room {
                                    Some(room) => {
                                        room.add_unit(&group_id);
                                    },
                                    None => {
                                        let mut room = Room::new();
                                        room.add_unit(&group_id);
                                        rooms.insert(topic_name, room);

                                    },
                                }
                            },
                            SystemEvent::Leave => {
                                if let Some(room) = room {
                                    println!("Участник {} leaved", group_id);
                                    if let Err(err) = room.leave_unit(&group_id) {
                                        error!("Error leaved unit {}, err: {:?}", group_id, err);
                                    }
                                    if room.unit_count == 0 {
                                        println!("allreade for job");

                                        let end_duration = headers.get("end_duration")
                                            .expect("cannot get end_duration")
                                            .expect("cannot get end_duration");
                                        let end_duration = std::str::from_utf8(end_duration)
                                            .expect("cannot parse end_duration from &[u8]")
                                            .parse::<u64>()
                                            .expect("cannot parse end_duration from &[u8]");

                                        let mut temp_files = Vec::new();
                                        let mut timestamps = Vec::new();

                                        let bucket_name = "test";
                                        let object_store = get_mini_store(bucket_name)
                                            .expect("cannot get a object sotre");
                                        let path = format!("{}/", topic_name);
                                        let directory_path = Path::from(path);
                                        let mut list_stream = object_store.list(Some(&directory_path));
                                    
                                        while let Some(file) = list_stream.next().await {
                                            match file {
                                                Ok(object_meta) => {
                                                    println!("File: {}", object_meta.location);
                                                    println!("Size: {} bytes", object_meta.size);
                                                    println!("Last modified: {:?}", object_meta.last_modified);

                                                    let is_common_file =  object_meta
                                                        .location
                                                        .filename()
                                                        .unwrap()
                                                        .starts_with("common");
                                                    if is_common_file {
                                                        continue;
                                                    }

                                                    let path = Path::from(object_meta.location);
                                                    let object = object_store.get(&path).await.unwrap();
                                                    let bytes = object.bytes().await.unwrap();

                                                    let mut reader = PacketReader::new(Cursor::new(&bytes));
                                                    let mut timestamp = 0;
                                                    while let Ok(packet) = reader.read_packet() {
                                                        if let Some(packet)  = packet {
                                                            if packet.data.starts_with(b"OpusTags") {
                                                                timestamp = extract_timestamp_from_ogg_header(&packet.data).unwrap();
                                                                break;
                                                            }

                                                        }
                                                    }

                                                    if timestamp > end_duration {
                                                        continue;
                                                    }

                                                    let temp_file = PathBuf::from(format!("{}.ogg", timestamp));
                                                    let mut file = File::create(&temp_file).expect("Не удалось создать временный файл");
                                                    file.write_all(&bytes).expect("Не удалось записать данные в файл");

                                                    println!("atimestamp {}", timestamp);

                                                    timestamps.push(timestamp);
                                                    temp_files.push((temp_file, timestamp));

                                                }
                                                Err(e) => eprintln!("Error listing file: {}", e),
                                            }
                                        }

                                        let min_timestamp = timestamps.iter().min().expect("Нет файлов для обработки");

                                        let mut ffmpeg_command = Command::new("ffmpeg");

                                        let mut filter_graph = String::new();
                                        let mut filter_graph_suffix = String::new();
                                                                        
                                        for (i, (temp_file, timestamp)) in temp_files.iter().enumerate() {
                                            println!("min timestamp {}", min_timestamp);
                                            println!("timestamp {}", timestamp);
                                            let delay = timestamp - min_timestamp;
                                            filter_graph.push_str(&format!("[{}:a]adelay={}ms[delayed{}];", i, delay, i + 1));
                                            filter_graph_suffix.push_str(&format!("[delayed{}]", i + 1));
                                            ffmpeg_command
                                                .arg("-i")
                                                .arg(temp_file);
                                        }

                                        filter_graph.push_str(&filter_graph_suffix);

                                        filter_graph
                                            .push_str(&format!("amix=inputs={}", temp_files.len()));

                                        let diff_duration = (end_duration - min_timestamp) / 1000 + 2;
                                        ffmpeg_command
                                            .arg("-to")
                                            .arg(diff_duration.to_string());

                                        println!("{}", filter_graph);
                                    
                                        let output_file = format!("common_{}_{}_{}.ogg", topic_name, min_timestamp, end_duration);
                                        ffmpeg_command
                                            .arg("-filter_complex")
                                            .arg(filter_graph)
                                            .arg(output_file.clone());
                                    
                                    
                                        let output = ffmpeg_command.output().expect("Не удалось запустить ffmpeg");

                                        if output.status.success() {
                                            println!("Аудио успешно наложено и сохранено.");
                                            let output_path = Path::from(format!("{}/{}.ogg", &topic_name, &output_file));
                                            let mut file = File::open(&output_file).expect("Не удалось открыть временный файл");

                                            let mut multipart_upload = object_store
                                                .put_multipart(&output_path)
                                                .await
                                                .expect("Не удалось начать multipart-загрузку");

                                            let part_size = 5 * 1024 * 1024; // 5 МБ
                                            let mut buffer = vec![0; part_size];
                                            let mut part_number = 1;

                                            loop {
                                                let bytes_read = file.read(&mut buffer).expect("Не удалось прочитать файл");
                                                if bytes_read == 0 {
                                                    break; // Файл полностью прочитан
                                                }
                                        
                                                // Загружаем часть
                                                let part_data = buffer[..bytes_read].to_vec();
                                                multipart_upload
                                                    .put_part(part_data.into())
                                                    .await
                                                    .expect("Не удалось загрузить часть файла");
                                        
                                        
                                                println!("Часть {} загружена.", part_number);
                                                part_number += 1;
                                            }

                                            multipart_upload
                                                .complete()
                                                .await
                                                .expect("Не удалось завершить multipart-загрузку");

                                            println!("Файл успешно загружен в S3: {}", output_path);
                                        } else {
                                            eprintln!("Ошибка при наложении аудио:");
                                            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                                        }
                                    
                                        for (temp_file, _timestamp) in temp_files {
                                            std::fs::remove_file(temp_file).expect("Не удалось удалить временный файл");
                                        }
                                        std::fs::remove_file(output_file).expect("Не удалось удалить временный выходной файл");

                                        rooms.remove(&topic_name);
                                    }
                                }
                            },
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Ошибка при получении сообщения: {:?}", e);
                }
            }
        }
    }

    Ok(())
}

fn headers_to_map(headers: &BorrowedHeaders) -> HashMap<&str, Option<&[u8]>> {
    let mut map =  HashMap::new();
    headers
        .iter()
        .for_each(|header| {
            map.insert(header.key, header.value);
        });

    map
}

fn get_mini_store(bucket_name: &str) -> Result<Arc<dyn ObjectStore>, String> {
    let minio_access_key_id = std::env::var("MINIO_ACCESS_KEY_ID").expect("MINIO_ACCESS_KEY_ID env var must be defined");
    let minio_secret_access_key = std::env::var("MINIO_SECRET_ACCESS_KEY").expect("MINIO_SECRET_ACCESS_KEY env var must be defined");
    let minio_endpoint = "http://127.0.0.1:9000";

    let minio = AmazonS3Builder::new()
    .with_access_key_id(minio_access_key_id)
    .with_secret_access_key(minio_secret_access_key)
    .with_endpoint(minio_endpoint) 
    .with_bucket_name(bucket_name)
    .with_region("us-east-1") 
    .with_allow_http(true) 
    .build()
    .map_err(|e| format!("Error creating MinIO client: {}", e))?;

    Ok(Arc::new(minio))
}

fn extract_timestamp_from_ogg_header(header: &[u8]) -> Option<u64> {
    if !header.starts_with(b"OpusTags") {
        return None;
    }

    let mut offset = 8; // Пропускаем "OpusTags" (8 байт)

    // Читаем длину vendor string (4 байта)
    let vendor_len = u32::from_le_bytes(header[offset..offset + 4].try_into().ok()?) as usize;
    offset += 4;

    // Пропускаем vendor string
    offset += vendor_len;

    // Читаем количество комментариев (4 байта)
    let comments_count = u32::from_le_bytes(header[offset..offset + 4].try_into().ok()?);
    offset += 4;

    // Ищем комментарий с START_TIME
    for i in 0..comments_count {
        // Читаем длину комментария (4 байта)
        let comment_len = u32::from_le_bytes(header[offset..offset + 4].try_into().ok()?) as usize;
        offset += 4;

        // Читаем сам комментарий
        let comment = &header[offset..offset + comment_len];
        offset += comment_len;

        // Пытаемся найти START_TIME
        if let Some(start_time_str) = std::str::from_utf8(comment).ok() {
            if let Some(start_time) = start_time_str.strip_prefix("START_TIME=") {
                return start_time.parse::<u64>().ok();
            }
        }
    }
    None
}