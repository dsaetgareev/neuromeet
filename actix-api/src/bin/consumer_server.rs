use std::{borrow::Cow, collections::HashMap, str::FromStr, sync::{Arc, RwLock}, time::{SystemTime, UNIX_EPOCH}};

use object_store::{aws::AmazonS3Builder, path::Path, ObjectStore, WriteMultipart};
use protobuf::Message as _;
use rand::Rng;
use rdkafka::{message::{BorrowedHeaders, Headers}, Message};
use sec_api::{kafka::{kafka_consumer::KafkaConsumer, kafka_rdclient::KafkaClient, SystemEvent}, s3::s3_writer::S3Writer};
use tokio_stream::StreamExt;
use tracing::info;
use types::protos::{media_packet::{media_packet::MediaType, MediaPacket}, packet_wrapper::PacketWrapper};

const SYSTEM_TOPIC_NAME: &str = "system_events";

#[derive(Debug)]
pub enum ConsumerError {
    Error
}

#[tokio::main]
async fn main() -> Result<(), ()> {
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
                            println!("Получено сообщение: {}, {}", payload_str, group_id);
                            tokio::spawn(async move {

                                let topic_name = String::from(topic_name);
                                let group_id = String::from(group_id); 

                                let time = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .expect("cannot get timestamp")
                                    .as_millis() as u64;
                                let duration = Arc::new(RwLock::new(time));
                                let serial = rand::thread_rng().gen();
                                let file_name = format!("{}/{}.{}.ogg", topic_name, group_id, time);

                                let bucket_name = "test";
                                let object_store = get_mini_store(bucket_name)
                                    .expect("cannot get a object sotre");

                                let path = Path::from(file_name);

                                let upload = object_store.put_multipart(&path).await.expect("cannot get upload");
                                let write = WriteMultipart::new(upload);
                                let mut multipart_writer = S3Writer::new(write, 128);
                                let mut ogg_writer = ogg::writing::PacketWriter::new(&mut multipart_writer);
                                let _ = ogg_writer.write_packet(
                                    generate_identification_header(),
                                    serial,
                                    ogg::PacketWriteEndInfo::EndPage,
                                    0);
                                let _ = ogg_writer.write_packet(
                                    generate_comment_header(),
                                    serial,
                                    ogg::PacketWriteEndInfo::EndPage,
                                    0);    
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
                                                        emit_packet(
                                                            payload.to_vec(),
                                                            &mut ogg_writer,
                                                            duration.clone(),
                                                            serial
                                                        );
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
                                    let _ = ogg_writer.write_packet(
                                        Vec::new(),
                                        serial,
                                        ogg::PacketWriteEndInfo::EndStream,
                                        *duration.read().unwrap());
                                    multipart_writer.finish().await.unwrap();
                                    let producer = KafkaClient::new();
                                    info!("leaved {}", group_id);
                                    producer
                                        .send_system_event(SystemEvent::Leave, &topic_name, &group_id)
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
    bytes: Vec<u8>,
        ogg_writer: &mut ogg::writing::PacketWriter<&mut S3Writer>, 
        duration: Arc<RwLock<u64>>,
        serial: u32,
    ) {
    match PacketWrapper::parse_from_bytes(&bytes) {
        Ok(media_packet) => {
            let packet: MediaPacket  = parse_media_packet(&media_packet.data)
                .expect("cannot get MediaPacket");
            let media_type = packet.media_type.enum_value().unwrap();
            if media_type == MediaType::AUDIO && packet.duration > 0.0 {

                let data = packet.data;
                let mut write_duration = duration.write().expect("cannot get a duration");
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

        },
        Err(_err) => {
            tracing::error!("failed to parse media packet");
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

fn generate_comment_header() -> Vec<u8> {
    let vendor = b"Rust Ogg Writer";
    let mut header = Vec::new();
    header.extend_from_slice(b"OpusTags"); // Magic signature
    header.extend_from_slice(&(vendor.len() as u32).to_le_bytes()); // Vendor length
    header.extend_from_slice(vendor); // Vendor string
    header.extend_from_slice(&[0, 0, 0, 0]); // User comments length (0 comments)
    header.extend_from_slice("Duration: 10".as_bytes());
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
    let minio_access_key_id = "Nw7N0DMb0fo7OpfJDYxE";
    let minio_secret_access_key = "wQ9rV5ZWaL7ooccjaGYAGqcLUNjWbm5dg0Tr7z7X";
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