use samsa::prelude::TcpConnection;

pub struct KafkaClient {

}

impl KafkaClient {
    pub async fn producer_client() -> Result<samsa::prelude::Producer, samsa::prelude::Error> {

        let bootstrap_addrs = vec![samsa::prelude::BrokerAddress {
            host: "127.0.0.1".to_owned(),
            port: 9092,
        }];
        let topic_name = "test";
        let producer_client = samsa::prelude::ProducerBuilder::<TcpConnection>::new(
            bootstrap_addrs,
            vec![topic_name.to_string()]
        )
            .await?
            .batch_timeout_ms(1)
            .max_batch_size(2)
            .clone()
            .build()
            .await;

        Ok(producer_client)
    }
}