#[derive(Debug)]
pub struct MediaSessionInfo {
    pub title: String,
    pub artist: String,
    pub source_app_id: String,
    pub thumbnail: windows::Storage::Streams::IRandomAccessStreamReference,
}

pub fn get_current_media_sessions() -> Result<Vec<MediaSessionInfo>, windows::core::Error> {
    use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;
    let sessions = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?
        .join()?
        .GetSessions()?;
    let mut ret: Vec<MediaSessionInfo> = vec![];
    for session in sessions.into_iter() {
        if let Ok(info) = MediaSessionInfo::from_session(&session) {
            ret.push(info);
        }
    }
    Ok(ret)
}

impl MediaSessionInfo {
    fn from_session(
        session: &windows::Media::Control::GlobalSystemMediaTransportControlsSession,
    ) -> Result<Self, windows::core::Error> {
        use windows::Media::Control::GlobalSystemMediaTransportControlsSessionMediaProperties;
        let source_app_id = session
            .SourceAppUserModelId()
            .map(|v| v.to_string_lossy())
            .unwrap_or_default();
        let media_properties: GlobalSystemMediaTransportControlsSessionMediaProperties =
            session.TryGetMediaPropertiesAsync()?.join()?;
        let ret: MediaSessionInfo = Self {
            title: media_properties
                .Title()
                .map(|v| v.to_string_lossy())
                .unwrap_or("<Unknown Title>".into()),
            artist: media_properties
                .Artist()
                .map(|v| v.to_string_lossy())
                .unwrap_or("<Unknown Artist>".into()),
            source_app_id,
            thumbnail: media_properties.Thumbnail()?,
        };
        Ok(ret)
    }

    pub fn get_thumnail(&self) -> Result<Vec<u8>, windows::core::Error> {
        use windows::Storage::Streams::{Buffer, DataReader, IBuffer};
        let thumbnail: Vec<u8> = {
            let thumbnail = self.thumbnail.OpenReadAsync()?.join()?;
            let size: u64 = thumbnail.Size()?;
            let buf: IBuffer = thumbnail
                .ReadAsync(
                    &Buffer::Create(size as u32).unwrap(),
                    size as u32,
                    windows::Storage::Streams::InputStreamOptions::None,
                )?
                .join()?;
            let mut thumbnail_data: Vec<u8> = vec![0u8; size as usize];
            DataReader::FromBuffer(&buf)?.ReadBytes(&mut thumbnail_data)?;
            thumbnail_data
        };
        Ok(thumbnail)
    }
}
