use std::{borrow::Cow, cell::RefCell, fs::File, io::{BufWriter, Write}, rc::Rc};

use protobuf::Message;
use samsa::prelude::{ConsumeMessage, ConsumerGroupBuilder, TcpConnection, TopicPartitionsBuilder};
use tokio_stream::StreamExt;
use tracing::info;
use types::protos::{media_packet::{media_packet::MediaType, MediaPacket}, packet_wrapper::PacketWrapper};

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

    let bootstrap_addrs = vec![samsa::prelude::BrokerAddress {
        host: "127.0.0.1".to_owned(),
        port: 9093,
    }];

    let group_id = "Squad".to_string();
    let src_topic = "test".to_string();

    let stream = ConsumerGroupBuilder::<TcpConnection>::new(
        bootstrap_addrs,
        group_id,
        TopicPartitionsBuilder::new()
            .assign(src_topic, vec![0, 1, 2, 3])
            .build(),
    )
        .await
        .map_err(|err| tracing::error!("{:?}", err))?
        .build()
        .await
        .map_err(|err| tracing::error!("{:?}", err))?
        .into_stream();
        // .throttle(Duration::from_secs(2));

    tokio::pin!(stream);
    let file = File::create("output.ogg").expect("cannot create file");
    let writer = BufWriter::new(file);
    let duration = Rc::new(RefCell::new(0u64));

    let mut ogg_writer = ogg::writing::PacketWriter::new(writer);
    let _ = ogg_writer.write_packet(
        generate_identification_header(),
        12345678,
        ogg::PacketWriteEndInfo::EndPage,
        0);
    let _ = ogg_writer.write_packet(
        generate_comment_header(),
        12345678,
        ogg::PacketWriteEndInfo::EndPage,
        0);    

    while let Some(message) = stream.next().await {
        let messages: Vec<ConsumeMessage> = message.unwrap().collect();
        messages.iter().for_each(|item| {
            emit_packet(item.value.to_vec(), &mut ogg_writer, duration.clone());
        });
    }
    Ok(())
}

fn emit_packet(
    bytes: Vec<u8>,
        ogg_writer: &mut ogg::writing::PacketWriter<BufWriter<File>>, 
        duration: Rc<RefCell<u64>>
    ) {
    match PacketWrapper::parse_from_bytes(&bytes) {
        Ok(media_packet) => {
            let packet: MediaPacket  = parse_media_packet(&media_packet.data)
                .expect("cannot get MediaPacket");
            let media_type = packet.media_type.enum_value().unwrap();
            if media_type == MediaType::AUDIO && packet.duration > 0.0 {

                info!("media_type {}", media_type);
                let data = packet.data;
                let dur = duration.borrow().clone() + 960 as u64;
                duration.replace(dur);
                let absgp = duration.borrow().clone();
                match ogg_writer.write_packet(
                    Cow::Owned(data),
                    12345678,
                    ogg::PacketWriteEndInfo::EndPage,
                    absgp
                ) {
                    Ok(_) => {
                        info!("all ok");
                    }
                    Err(err) => {
                        tracing::error!("{}", err)
                    },
                } ;
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