use std::{collections::HashMap, str::FromStr, time::{Duration, SystemTime, UNIX_EPOCH}};

use rdkafka::{admin::{AdminClient, AdminOptions, NewTopic, TopicReplication}, client::DefaultClientContext, error::KafkaError, message::{Header, OwnedHeaders}, producer::{FutureProducer, FutureRecord}, types::RDKafkaErrorCode, util::Timeout, ClientConfig};
use tracing::{error, info};

const SYSTEM_TOPIC_NAME: &str = "system_events";
const KAFKA_CONNECTION_URL: &str = "localhost:9092";

#[derive(PartialEq)]
pub enum SystemEvent {
    Create,
    Leave,
}

impl SystemEvent {
    pub fn to_string(&self) -> &'static str {
        match self {
            SystemEvent::Create => "Create",
            SystemEvent::Leave => "Leave",
        }
    }
}

impl FromStr for SystemEvent {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Create" => Ok(SystemEvent::Create),
            "Leave" => Ok(SystemEvent::Leave),
            _ => Err(format!("Unknown variant: {}", s)),
        }
    }
}
pub struct KafkaClient {
    client_config: ClientConfig,
    system_producer: Result<FutureProducer, KafkaError>,
}

impl KafkaClient {

    pub fn new() -> Self {
        let mut client_config = ClientConfig::new();
        client_config
            .set("bootstrap.servers", KAFKA_CONNECTION_URL)
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
        for res in result {
            match res {
                Ok(name) => {
                    info!("Topic '{}' created successfully.", name);
                    if let Err(err) = self.create_system_event(topic_name, key).await {
                        error!("Error sending create events: {:?}, {:?}, {:?}", topic_name, key, err);
                    }
                },
                Err((topic_name, code)) => {
                    error!("Error creating topic: {:?}, {:?}", topic_name, code);
                    if RDKafkaErrorCode::TopicAlreadyExists == code {
                        if let Err(err) = self.create_system_event(&topic_name, key).await {
                            error!("Error sending create events: {:?}, {:?}, {:?}", topic_name, key, err);
                        }
                    }
                }
            }
        }
    }

    pub async fn send_system_event(&self, system_event: SystemEvent, topic_name: &str, key: &String, additional_info: Option<HashMap<String, String>>) -> Result<(), ()> {
        if let Ok(producer) = &self.system_producer {
            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("cannot get timestamp")
                .as_millis() as u64;
            let mut headers = OwnedHeaders::new()
                .insert(Header { key: "key", value: Some(key) })
                .insert(Header { key: "topic_name", value: Some(topic_name) })
                .insert(Header { key: "timestamp", value: Some(&time.to_string()) });
            if let Some(additional_info) = additional_info {
                for (key, value) in additional_info.iter() {
                    headers = headers.insert(Header { key, value: Some(value) });                
                }
            }
            let message = system_event.to_string().as_bytes().to_vec();
            let record: FutureRecord<'_, String, Vec<u8>> = FutureRecord::to(SYSTEM_TOPIC_NAME)
                .key(key)
                .headers(headers.clone())
                .payload(&message);
            let _ = producer.send(record, Timeout::After(Duration::from_millis(10))).await;
        }
        Ok(())
    }

    pub async fn create_system_event(&self, topic_name: &str, key: &String) -> Result<(), ()> {
        self.send_system_event(SystemEvent::Create, topic_name, key, None).await
    }
    pub async fn leave_system_event(&self, topic_name: &str, key: &String) -> Result<(), ()> {  
        self.send_system_event(SystemEvent::Leave, topic_name, key, None).await
    }
}
