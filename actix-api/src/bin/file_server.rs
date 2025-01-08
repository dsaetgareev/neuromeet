use std::{collections::HashMap, io::Cursor, str::FromStr, sync::Arc};

use futures::StreamExt;
use object_store::{aws::AmazonS3Builder, path::Path, ObjectStore};
use ogg::{PacketReader, PacketWriteEndInfo, PacketWriter};
use rdkafka::{message::{BorrowedHeaders, Headers}, Message};
use sec_api::kafka::{kafka_consumer::KafkaConsumer, SystemEvent};
use tracing::error;


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
                                        let file_name = format!("{}.ogg", topic_name);
                                        let ogg_file = std::fs::File::create(file_name).unwrap();
                                        let mut ogg_writer = PacketWriter::new(ogg_file);

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

                                                    let path = Path::from(object_meta.location);
                                                    let object = object_store.get(&path).await.unwrap();
                                                    let bytes = object.bytes().await.unwrap();
                                            
                                                    // let mut ogg_reader = PacketReader::new(bytes.as_ref());
                                                    let mut ogg_reader = PacketReader::new(Cursor::new(bytes.as_ref()));
                                                    ogg_reader.delete_unread_packets();

                                                    while let Some(pck) = ogg_reader.read_packet().unwrap() {
                                                        let inf = if pck.last_in_stream() {
                                                            PacketWriteEndInfo::EndStream
                                                        } else if pck.last_in_page() {
                                                            PacketWriteEndInfo::EndPage
                                                        } else {
                                                            PacketWriteEndInfo::NormalPacket
                                                        };
                                                        let stream_serial = pck.stream_serial();
                                                        let absgp_page = pck.absgp_page();
                                                        let _ = ogg_writer.write_packet(pck.data,
                                                            stream_serial,
                                                            inf,
                                                            absgp_page);
                                                    }
                                                }
                                                Err(e) => eprintln!("Error listing file: {}", e),
                                            }
                                        }

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