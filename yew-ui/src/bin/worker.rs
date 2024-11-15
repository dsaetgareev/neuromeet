use videocall_client::VideoWorker;
use gloo_worker::Registrable;

fn main() {
    VideoWorker::registrar().register();
}

