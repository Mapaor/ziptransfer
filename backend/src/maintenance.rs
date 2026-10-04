use sqlx::{SqlitePool, Row};
use tokio::time::{sleep, Duration};
use chrono::Utc;

pub fn start_cleanup_task(pool: SqlitePool) {
    tokio::spawn(async move {
        loop {
            // Wake up every hour
            sleep(Duration::from_secs(3600)).await;
            
            println!("Running automated maintenance to clean up expired transfers...");
            
            let expired_transfers_result = sqlx::query("SELECT id FROM transfers WHERE expires_at < ?")
                .bind(Utc::now().naive_utc())
                .fetch_all(&pool)
                .await;

            match expired_transfers_result {
                Ok(expired_transfers) => {
                    for row in expired_transfers {
                        let id: String = row.get("id");
                        let file_path = format!("./uploads/{}", id);
                        
                        // Delete the file from the disk, ignoring if it doesn't exist
                        if let Err(e) = tokio::fs::remove_file(&file_path).await {
                            if e.kind() != std::io::ErrorKind::NotFound {
                                eprintln!("Failed to delete file {}: {}", file_path, e);
                            }
                        }
                        
                        // Delete the record from SQLite database
                        if let Err(e) = sqlx::query("DELETE FROM transfers WHERE id = ?")
                            .bind(&id)
                            .execute(&pool)
                            .await 
                        {
                            eprintln!("Failed to delete transfer record {}: {}", id, e);
                        } else {
                            println!("Successfully cleaned up expired transfer {}", id);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to fetch expired transfers: {}", e);
                }
            }
        }
    });
}
