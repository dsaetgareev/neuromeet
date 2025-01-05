use std::time::Duration;

use rdkafka::{admin::{AdminClient, AdminOptions, NewTopic, TopicReplication}, client::DefaultClientContext, error::KafkaError, message::{Header, OwnedHeaders}, producer::{self, FutureProducer, FutureRecord}, types::RDKafkaErrorCode, util::Timeout, ClientConfig};
use tracing::{error, info};

const SYSTEM_TOPIC_NAME: &str = "system_events";

pub struct KafkaClient {
    client_config: ClientConfig,
    system_producer: Result<FutureProducer, KafkaError>,
}

impl KafkaClient {

    pub fn new() -> Self {
        let mut client_config = ClientConfig::new();
        client_config
            .set("bootstrap.servers", "localhost:9092")
            .set("message.timeout.ms", "5000");
        let system_producer: Result<FutureProducer, KafkaError> = client_config
            .create();
            
        Self {
            client_config,
            system_producer,
        }
    }

    pub async fn create_producer(&mut self) -> Result<FutureProducer, KafkaError> {
        let producer: Result<FutureProducer, KafkaError> = self.client_config
            .create();
        producer
    }

    pub async fn create_topic(&self, topic_name: &str, key: &String) {
        let admin_client: AdminClient<DefaultClientContext> = self.client_config.create().expect("cannot create admin client");
        let partitions = 1; 
        let replication = 1;
        let new_topic = NewTopic::new(
            topic_name,
            partitions,
            TopicReplication::Fixed(replication)
        );
        let result = admin_client.create_topics(&[new_topic], &AdminOptions::new()).await.expect("cannot create a topic");
        let headers = OwnedHeaders::new()
            .insert(Header { key: "key", value: Some(key) })
            .insert(Header { key: "topic_name", value: Some(topic_name) });
        for res in result {
            match res {
                Ok(name) => {
                    info!("Топик '{}' успешно создан.", name);
                    if let Ok(producer) = &self.system_producer {
                        let message = &"common".as_bytes().to_vec();
                        let common_headers = OwnedHeaders::new()
                            .insert(Header { key: "key", value: Some(topic_name) })
                            .insert(Header { key: "topic_name", value: Some(topic_name) });
                        let record: FutureRecord<'_, String, Vec<u8>> = FutureRecord::to(SYSTEM_TOPIC_NAME)
                            .key(&name)
                            .headers(common_headers)
                            .payload(message);
                        let _ = producer.send(record, Timeout::After(Duration::from_millis(10))).await;
                        // let message = &"unit".as_bytes().to_vec();
                        // let record: FutureRecord<'_, String, Vec<u8>> = FutureRecord::to(SYSTEM_TOPIC_NAME)
                        //     .key(key)
                        //     .headers(headers.clone())
                        //     .payload(message);
                        // let _ = producer.send(record, Timeout::After(Duration::from_millis(10))).await;
                    }
                },
                Err((topic_name, code)) => {
                    error!("Ошибка при создании топика: {:?}, {:?}", topic_name, code);
                    // if RDKafkaErrorCode::TopicAlreadyExists == code {
                    //     if let Ok(producer) = &self.system_producer {
                    //         let message = &"unit".as_bytes().to_vec();
                    //         let record: FutureRecord<'_, String, Vec<u8>> = FutureRecord::to(SYSTEM_TOPIC_NAME)
                    //             .key(key)
                    //             .headers(headers.clone())
                    //             .payload(message);
                    //         let _ = producer.send(record, Timeout::After(Duration::from_millis(10))).await;
                    //     }
                    // }
                }
            }
        }
    }
}
