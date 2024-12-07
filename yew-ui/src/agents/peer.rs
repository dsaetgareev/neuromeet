use web_sys::MediaStream;


#[derive(Clone, PartialEq, Debug)]
pub struct Peer {
    peer_id: String,
    video_ms: MediaStream,
    screen_ms: MediaStream,
}

impl Peer {
    pub fn new(
        peer_id: String, 
        video_ms: MediaStream,
        screen_ms: MediaStream,
    ) -> Self {
        Self { peer_id, video_ms, screen_ms }
    }

    pub fn get_video_ms(&self) -> &MediaStream {
        &self.video_ms
    }

    pub fn get_screen_ms(&self) -> &MediaStream {
        &self.screen_ms
    }

    pub fn get_video_ms_cloned(&self) -> MediaStream {
        self.video_ms.clone()
    }

    pub fn get_screen_ms_cloned(&self) -> MediaStream {
        self.screen_ms.clone()
    }
}