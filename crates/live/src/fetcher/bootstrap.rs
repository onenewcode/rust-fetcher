use crate::error::Result;

pub struct BootstrapContext<'a> {
    pub room_id: &'a mut Option<String>,
}

pub fn cached_room_id(room_id: Option<String>) -> Result<Option<String>> {
    Ok(room_id)
}

pub fn room_id_from_cache(context: &BootstrapContext<'_>) -> Result<Option<String>> {
    cached_room_id(context.room_id.clone())
}

pub async fn ensure_room_id(fetcher: &mut super::DouyinLiveRustFetcher) -> Result<String> {
    if let Some(room_id) = room_id_from_cache(&BootstrapContext {
        room_id: &mut fetcher.room_id,
    })? {
        return Ok(room_id);
    }

    fetcher.ensure_ttwid().await?;
    let nonce = fetcher.fetch_ac_nonce().await?;
    let room_id = fetcher.resolve_room_id_from_live_page(&nonce).await?;
    fetcher.room_id = Some(room_id.clone());
    Ok(room_id)
}

#[cfg(test)]
mod tests {
    use super::{BootstrapContext, cached_room_id, room_id_from_cache};

    #[test]
    fn uses_cached_room_id_when_available() {
        let room_id = cached_room_id(Some("room-42".to_string())).unwrap();
        assert_eq!(room_id, Some("room-42".to_string()));
    }

    #[test]
    fn skips_cached_room_id_when_missing() {
        let room_id = cached_room_id(None).unwrap();
        assert_eq!(room_id, None);
    }

    #[test]
    fn reads_room_id_from_context_cache() {
        let mut cached = Some("room-99".to_string());
        let room_id = room_id_from_cache(&BootstrapContext {
            room_id: &mut cached,
        })
        .unwrap();
        assert_eq!(room_id, Some("room-99".to_string()));
    }
}
