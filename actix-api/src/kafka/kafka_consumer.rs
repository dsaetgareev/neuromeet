use rdkafka::{consumer::{Consumer, StreamConsumer}, error::KafkaError, ClientConfig, TopicPartitionList};

const KAFKA_CONNECTION_URL: &str = "localhost:9092";

pub struct KafkaConsumer {
    client_config: ClientConfig,
}

impl KafkaConsumer {
    pub fn new() -> Self {
        let mut client_config = ClientConfig::new();
        client_config
            .set("bootstrap.servers", KAFKA_CONNECTION_URL);
        Self {
            client_config,
        }
    }    

    pub async fn create_consumer(&mut self, topic_name: &str, key: &str) -> Result<StreamConsumer, KafkaError> {

        let consumer: StreamConsumer = self.client_config
            .set("group.id", key)
            .set("bootstrap.servers", KAFKA_CONNECTION_URL)
            .create()
            .expect("cannot create a consumer");
        let partitions = vec![0];
        let mut tpl = TopicPartitionList::new();
        for &partition in &partitions {
            tpl.add_partition(topic_name, partition);
        }
        consumer.assign(&tpl).expect("cannot assign partitions");

        consumer.subscribe(&[topic_name]).expect("cannot subscribe a topic");
        Ok(consumer)
    }
}