use uuid::Uuid;

use crate::{
    models::query::{ResponseVideo, Video},
    repositories::{videos_repo::VideosRepository, PostgresRepo},
    Result,
};

#[derive(Clone)]
pub struct VideosService {
    repo: PostgresRepo,
}

impl VideosService {
    pub fn new(repo: PostgresRepo) -> Self {
        Self { repo }
    }
    pub async fn videos(&self) -> Result<Vec<Video>> {
        let videos = self.repo.videos().await?;
        Ok(videos)
    }

    pub async fn create_video(
        &self,
        title: &str,
        youtube_id: &str,
        duration: &str,
        views: Option<i32>,
    ) -> Result<()> {
        let id = Uuid::now_v7();
        self.repo
            .create_video(id, title, youtube_id, duration, views)
            .await?;
        Ok(())
    }

    pub async fn update_video(
        &self,
        video_id: &str,
        title: Option<&str>,
        youtube_id: Option<&str>,
        duration: Option<&str>,
        views: Option<i32>,
    ) -> Result<()> {
        let video_id = Uuid::parse_str(video_id).unwrap();
        self.repo
            .update_video(video_id, title, youtube_id, duration, views)
            .await?;

        Ok(())
    }

    pub async fn delete_video(&self, video_id: &str) -> Result<()> {
        let video_id = Uuid::parse_str(video_id).unwrap();
        self.repo.delete_video(video_id).await?;

        Ok(())
    }

    pub async fn create_category(&self, category: &str) -> Result<()> {
        let category_id = Uuid::now_v7();
        self.repo.create_category(category_id, category).await?;

        Ok(())
    }

    pub async fn delete_category(&self, category_id: &str) -> Result<()> {
        let category_id = Uuid::parse_str(category_id).unwrap();
        self.repo.delete_category(category_id).await?;
        Ok(())
    }

    pub async fn add_category_to_video(&self, video_id: &str, category_id: &str) -> Result<()> {
        let video_id = Uuid::parse_str(video_id).unwrap();
        let category_id = Uuid::parse_str(category_id).unwrap();
        self.repo
            .add_category_to_video(video_id, category_id)
            .await?;

        Ok(())
    }
    pub async fn get_video_by_youtube_id(&self, youtube_id: &str) -> Result<ResponseVideo> {
        let video = self.repo.get_video_by_youtube_id(youtube_id).await?;

        Ok(video)
    }
    pub async fn remove_category_from_video(
        &self,
        video_id: &str,
        category_id: &str,
    ) -> Result<()> {
        let video_id = Uuid::parse_str(video_id).unwrap();
        let category_id = Uuid::parse_str(category_id).unwrap();
        self.repo
            .remove_category_from_video(video_id, category_id)
            .await?;
        Ok(())
    }
}
