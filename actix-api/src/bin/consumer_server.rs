use std::{borrow::Cow, collections::{BTreeMap, HashMap}, str::FromStr, sync::{Arc, RwLock}, time::{SystemTime, UNIX_EPOCH}};

use object_store::{aws::AmazonS3Builder, path::Path, ObjectStore, WriteMultipart};
use protobuf::Message as _;
use rand::Rng;
use rdkafka::{message::{BorrowedHeaders, Headers}, Message};
use sec_api::{kafka::{kafka_consumer::KafkaConsumer, kafka_rdclient::KafkaClient, SystemEvent}, s3::s3_writer::S3Writer};
use tokio_stream::StreamExt;
use tracing::info;
use types::protos::{media_packet::{self, media_packet::MediaType, MediaPacket}, packet_wrapper::PacketWrapper};
use dotenv::dotenv;

const SYSTEM_TOPIC_NAME: &str = "system_events";
const SILENCE_PACKET: &[u8] = &[0xF8, 0xFF, 0xFE]; 

#[derive(Debug)]
pub enum ConsumerError {
    Error
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

    if let Ok(system_consumer) = system_consumer.create_consumer(SYSTEM_TOPIC_NAME, "system_group").await {
        while let Some(message) = system_consumer.stream().next().await {
            match message {
                Ok(msg) => {
                    if let Some(payload) = msg.payload() {
                        let payload_str = std::str::from_utf8(payload).unwrap();
                        if SystemEvent::Create == SystemEvent::from_str(payload_str).expect("cannot parsing SystemEvent") {
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
                            let create_time = headers.get("timestamp")
                                .expect("cannot get a create_time")
                                .expect("cannot get a create_time");
                            let create_time = std::str::from_utf8(create_time)
                                .expect("cannot parse a topic_name from &[u8]");
                            let create_time = String::from(create_time); 
                            println!("Получено сообщение: {}, {}", payload_str, group_id);
                            tokio::spawn(async move {

                                let topic_name = String::from(topic_name);
                                let group_id = String::from(group_id); 
                                let create_time = String::from(create_time); 

                                let origin_duration = Arc::new(RwLock::new(0));
                                let duration = Arc::new(RwLock::new(0));
                                let sequence = Arc::new(RwLock::new(0));
                                let mut cache: BTreeMap<u64, MediaPacket> = BTreeMap::new();
                                let serial = rand::thread_rng().gen();
                                let file_name = format!("{}/{}.{}.ogg", topic_name, group_id, create_time);

                                let bucket_name = "test";
                                let object_store = get_mini_store(bucket_name)
                                    .expect("cannot get a object sotre");

                                let path = Path::from(file_name);

                                let upload = object_store.put_multipart(&path).await.expect("cannot get upload");
                                let write = WriteMultipart::new(upload);
                                let mut multipart_writer = S3Writer::new(write, 128);
                                let mut ogg_writer = ogg::writing::PacketWriter::new(&mut multipart_writer);
                                let mut builder = KafkaConsumer::new();
                                let consumer = builder.create_consumer(&topic_name, &group_id).await;
                                if let Ok(common_consumer) = consumer {
                                    while let Some(message) = common_consumer.stream().next().await {
                                        match message {
                                            Ok(msg) => {
                                                let user_key = std::str::from_utf8(msg.key().unwrap())
                                                    .expect("cannot get a user_key");
                                                if group_id.eq(user_key) {
                                                    if let Some(payload) = msg.payload() {
                                                        match PacketWrapper::parse_from_bytes(&payload.to_vec()) {
                                                            Ok(packet_wrapper) => {
                                                                let packet: MediaPacket  = parse_media_packet(&packet_wrapper.data)
                                                                    .expect("cannot get MediaPacket");
                                                                let media_type = packet.media_type.enum_value().unwrap();
                                                                if media_type == MediaType::AUDIO {
                                                                    let current_sequence = packet.video_metadata.sequence;
                                                                    let write_sequence = sequence.write().unwrap();
                                                                    if current_sequence < *write_sequence {
                                                                        info!("curren sequence < write sequence {} < {}", current_sequence, *write_sequence);
                                                                    }
                                                                    emit_packet(
                                                                        packet,
                                                                        &mut ogg_writer,
                                                                        duration.clone(),
                                                                        origin_duration.clone(),
                                                                        serial
                                                                    );
                                                                }
                                                            },
                                                            Err(_err) => {
                                                                tracing::error!("failed to parse media packet");
                                                            },
                                                        };
                                                    } else {
                                                        break;
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("Ошибка при получении сообщения: {:?}", e);
                                                return;
                                            }
                                        }
                                    }
                                    info!("end duration {}", *duration.read().unwrap());
                                    info!("cache len {}", cache.len());
                                    let _ = ogg_writer.write_packet(
                                        Vec::new(),
                                        serial,
                                        ogg::PacketWriteEndInfo::EndStream,
                                        *duration.read().unwrap());
                                    multipart_writer.finish().await.unwrap();
                                    let producer = KafkaClient::new();
                                    info!("leaved {}", group_id);
                                    let mut additional_info = HashMap::new();
                                    additional_info.insert("end_duration".to_string(), origin_duration.read().unwrap().to_string());
                                    producer
                                        .send_system_event(SystemEvent::Leave, &topic_name, &group_id, Some(additional_info))
                                        .await
                                        .expect("cannot send SystemEvent::Leave");
                                }
                            });
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

fn emit_packet(
    packet: MediaPacket,
    ogg_writer: &mut ogg::writing::PacketWriter<&mut S3Writer>, 
    duration: Arc<RwLock<u64>>,
    origin_duration: Arc<RwLock<u64>>,
    serial: u32,
) {
    
    let packet_timestamp = packet.timestamp as u64;
    let mut write_duration = duration.write().expect("cannot get a duration");
    let mut write_origin_duration = origin_duration.write().expect("cannot get a duration");

    if *write_origin_duration == 0 {
        *write_origin_duration = packet_timestamp;
        info!("start_time {}", packet_timestamp);
        let absgp = write_duration.clone();
        let _ = ogg_writer.write_packet(
            generate_identification_header(),
            serial,
            ogg::PacketWriteEndInfo::EndPage,
            absgp);
        let _ = ogg_writer.write_packet(
            generate_comment_header(packet_timestamp),
            serial,
            ogg::PacketWriteEndInfo::EndPage,
            absgp);    
    } else {
        if packet_timestamp < *write_origin_duration {
            info!("shpion");
            return;
        }
        let diff = packet_timestamp - *write_origin_duration;
        let amount = diff as f64 / 20.0;
        if amount as i64 > 1 {
            info!("packet_timestam {}", packet_timestamp);
            info!("original duration {}", *write_origin_duration);
            info!("tishina {}, {}", amount, diff);
            let whole_part = amount.trunc();
            let remainder = amount - whole_part;
            println!("remainder {}", remainder);
            for _i in 0..whole_part as i64 {
                *write_duration += 960;
                let absgp = write_duration.clone();
                if let Err(err) = ogg_writer.write_packet(
                    Cow::Borrowed(SILENCE_PACKET),
                    serial,
                    ogg::PacketWriteEndInfo::EndPage,
                    absgp
                ) {
                    tracing::error!("{}", err);
                } 
            }
            *write_duration += (remainder * 960.0) as u64;
            let absgp = write_duration.clone();
            if let Err(err) = ogg_writer.write_packet(
                Cow::Borrowed(SILENCE_PACKET),
                serial,
                ogg::PacketWriteEndInfo::EndPage,
                absgp
            ) {
                tracing::error!("{}", err);
            } 
        }
    
        *write_origin_duration = packet_timestamp; 
        let data = packet.data;
        *write_duration += 960;
        let absgp = write_duration.clone();
        if let Err(err) = ogg_writer.write_packet(
            Cow::Owned(data),
            serial,
            ogg::PacketWriteEndInfo::EndPage,
            absgp
        ) {
            tracing::error!("{}", err)
        }
    }
}

fn parse_media_packet(data: &[u8]) -> Result<MediaPacket, ConsumerError> {
    Ok(MediaPacket::parse_from_bytes(data).map_err(|_| ConsumerError::Error)?)
}

fn generate_identification_header() -> Vec<u8> {
    let mut header = Vec::new();
    header.extend_from_slice(b"OpusHead"); // Magic signature
    header.push(0x01); // Version
    header.push(0x01); // Channel count (2 = stereo)
    header.extend_from_slice(&312u16.to_le_bytes()); // Pre-skip (312 samples)
    header.extend_from_slice(&48000u32.to_le_bytes()); // Input sample rate (48 kHz)
    header.extend_from_slice(&[0x00, 0x00]); // Output gain
    header.push(0x00); // Channel mapping family (0 for mono/stereo)
    header
}

fn generate_comment_header(start_time: u64) -> Vec<u8> {
    let start_time_str = format!("START_TIME={}", start_time);
    let vendor = b"Rust Ogg Writer";
    let mut header = Vec::new();
    header.extend_from_slice(b"OpusTags"); // Magic signature
    header.extend_from_slice(&(vendor.len() as u32).to_le_bytes()); // Vendor length
    header.extend_from_slice(vendor); // Vendor string
    header.extend_from_slice(&1u32.to_le_bytes()); // 1 комментарий
    header.extend_from_slice(&(start_time_str.len() as u32).to_le_bytes());
    header.extend_from_slice(start_time_str.as_bytes());
    header
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

fn _get_current_time() -> u64 {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("cannot get timestamp")
        .as_millis() as u64;
    time
}   