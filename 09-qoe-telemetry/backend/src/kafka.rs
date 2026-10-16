use std::time::Duration;

use futures::future::try_join_all;
use rdkafka::ClientConfig;
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::types::RDKafkaErrorCode;
use tokio::sync::mpsc;

use crate::beacon::Beacon;
use crate::error::AppError;
use crate::sink::BeaconSink;

pub struct KafkaSink {
    producer: FutureProducer,
    topic: String,
}

impl KafkaSink {
    pub fn new(brokers: &str, topic: &str) -> Result<Self, AppError> {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("linger.ms", "5")
            .set("message.timeout.ms", "5000")
            .create()?;
        Ok(Self {
            producer,
            topic: topic.to_string(),
        })
    }
}

impl BeaconSink for KafkaSink {
    async fn publish(&self, beacons: &[Beacon]) -> Result<usize, AppError> {
        let payloads = beacons
            .iter()
            .map(|b| Ok((b.session_id.as_str(), b.to_json()?)))
            .collect::<Result<Vec<_>, AppError>>()?;
        let sends = payloads.iter().map(|(key, payload)| async move {
            self.producer
                .send(
                    FutureRecord::to(&self.topic).key(*key).payload(payload),
                    Duration::from_secs(5),
                )
                .await
                .map_err(|(e, _)| AppError::Kafka(e))
        });
        try_join_all(sends).await?;
        Ok(beacons.len())
    }
}

pub async fn ensure_topic(brokers: &str, topic: &str, partitions: i32) -> Result<(), AppError> {
    let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .create()?;
    let results = admin
        .create_topics(
            &[NewTopic::new(topic, partitions, TopicReplication::Fixed(1))],
            &AdminOptions::new().operation_timeout(Some(Duration::from_secs(10))),
        )
        .await?;
    for result in results {
        match result {
            Ok(_) | Err((_, RDKafkaErrorCode::TopicAlreadyExists)) => {}
            Err((name, code)) => return Err(AppError::Topic(format!("{name}: {code}"))),
        }
    }
    Ok(())
}

pub fn consumer(brokers: &str, topic: &str, group: &str) -> Result<StreamConsumer, AppError> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", group)
        .set("auto.offset.reset", "latest")
        .set("enable.auto.commit", "true")
        .create()?;
    consumer.subscribe(&[topic])?;
    Ok(consumer)
}

pub async fn pump(consumer: StreamConsumer, tx: mpsc::Sender<Beacon>) -> Result<(), AppError> {
    loop {
        let message = consumer.recv().await?;
        let Some(payload) = message.payload() else {
            eprintln!("skipping empty kafka message at offset {}", message.offset());
            continue;
        };
        match Beacon::from_json(payload) {
            Ok(beacon) => {
                if tx.send(beacon).await.is_err() {
                    return Ok(());
                }
            }
            Err(e) => eprintln!("skipping malformed beacon at offset {}: {e}", message.offset()),
        }
    }
}
