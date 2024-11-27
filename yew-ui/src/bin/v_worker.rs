use videocall_client::workers::{worker_start, DecoderType};


fn main() {
    worker_start(DecoderType::Video);
}