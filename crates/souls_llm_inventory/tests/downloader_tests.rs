use souls_llm_inventory::downloader::{DownloadManager, DownloadState};
use tempfile::tempdir;

#[tokio::test]
async fn test_downloader_queue_priority_and_pause_resume() {
    let manager = DownloadManager::new();
    let temp = tempdir().unwrap();

    let item1 = manager
        .enqueue_download(
            "Qwen/Test-7B",
            "model-1.gguf",
            "http://127.0.0.1:9/dummy-1",
            None,
            temp.path(),
        )
        .await
        .unwrap();

    let item2 = manager
        .enqueue_download(
            "Qwen/Test-3B",
            "model-2.gguf",
            "http://127.0.0.1:9/dummy-2",
            None,
            temp.path(),
        )
        .await
        .unwrap();

    let list = manager.list_downloads().await;
    assert_eq!(list.len(), 2);

    // Pause item2
    manager.pause_download(&item2.id).await.unwrap();
    let paused_item = manager.get_download(&item2.id).await.unwrap();
    assert_eq!(paused_item.state, DownloadState::Paused);

    // Resume item2
    manager.resume_download(&item2.id).await.unwrap();
    let resumed_item = manager.get_download(&item2.id).await.unwrap();
    assert_eq!(resumed_item.state, DownloadState::Queued);

    // Cancel both
    manager.cancel_download(&item1.id).await.unwrap();
    manager.cancel_download(&item2.id).await.unwrap();

    assert_eq!(
        manager.get_download(&item1.id).await.unwrap().state,
        DownloadState::Cancelled
    );
    assert_eq!(
        manager.get_download(&item2.id).await.unwrap().state,
        DownloadState::Cancelled
    );
}
