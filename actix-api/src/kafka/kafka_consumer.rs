use rdkafka::{consumer::{Consumer, StreamConsumer}, error::KafkaError, ClientConfig, TopicPartitionList};

pub struct KafkaConsumer {
    client_config: ClientConfig,
    kafka_connection_url: String,
}

impl KafkaConsumer {
    pub fn new() -> Self {
        let kafka_connection_url = std::env::var("KAFKA_CONNECTION_URL").expect("KAFKA_CONNECTION_URL env var must be defined");
        let mut client_config = ClientConfig::new();
        client_config
            .set("bootstrap.servers", &kafka_connection_url);
        Self {
            client_config,
            kafka_connection_url
        }
    }    

    pub async fn create_consumer(&mut self, topic_name: &str, key: &str) -> Result<StreamConsumer, KafkaError> {

        let consumer: StreamConsumer = self.client_config
            .set("group.id", key)
            .set("bootstrap.servers", &self.kafka_connection_url)
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