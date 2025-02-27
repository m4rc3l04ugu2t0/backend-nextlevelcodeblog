use crate::{
    models::news_post::{
        CreateNewsPostDto, NewsPost, PostCommentWithAuthor, PostCommentWithComments,
    },
    repositories::{news_post_repo::NewsPostsRepository, PostgresRepo},
    Result,
};

#[derive(Clone)]
pub struct NewsPostsService {
    repo: PostgresRepo,
}

impl NewsPostsService {
    pub fn new(repo: PostgresRepo) -> Self {
        Self { repo }
    }
    pub async fn get_news_posts(&self) -> Result<Vec<NewsPost>> {
        let newspost = self.repo.get_news_posts().await?;

        Ok(newspost)
    }

    pub async fn create_news_post(
        &self,
        news_post: CreateNewsPostDto,
        author_id: &str,
    ) -> Result<()> {
         self
            .repo
            .create_news_post(
                &news_post.url,
                &news_post.description,
                author_id,
                &news_post.author_name,
            )
            .await?;

        Ok(())
    }

    pub async fn update_news_post(
        &self,
        news_post_id: &str,
        update_news_post_url: Option<&str>,
        update_news_post_description: Option<&str>,
    ) -> Result<()> {
         self
            .repo
            .update_news_post(
                news_post_id,
                update_news_post_url,
                update_news_post_description,
            )
            .await?;

        Ok(())
    }

    pub async fn delete_news_post(&self, news_post_id: &str) -> Result<()> {
        self.repo.delete_news_post(news_post_id).await?;
        Ok(())
    }

    pub async fn get_posts_with_comments(&self, post_id: &str) -> Result<PostCommentWithComments> {
        let posts = self.repo.get_posts_with_comments(post_id).await?;

        Ok(posts)
    }

    pub async fn get_all_posts_with_comments(&self) -> Result<Vec<PostCommentWithComments>> {
        let posts_with_comments = self.repo.get_all_posts_with_comments().await?;

        Ok(posts_with_comments)
    }

    pub async fn get_comments_for_post(&self, post_id: &str) -> Result<Vec<PostCommentWithAuthor>> {
        let comments = self.repo.get_comments_for_post(post_id).await?;

        Ok(comments)
    }

    pub async fn create_comment(
        &self,
        post_id: &str,
        content: &str,
        author_id: &str,
        author_name: &str,
    ) -> Result<()> {
         self
            .repo
            .create_comment(post_id, content, author_id, author_name)
            .await?;
        Ok(())
    }

    pub async fn update_comment(
        &self,
        comment_id: &str,
        content: Option<&str>,
    ) -> Result<()> {
         self.repo.update_comment(comment_id, content).await?;
        Ok(())
    }

    pub async fn delete_comment(&self, comment_id: &str) -> Result<()> {
        self.repo.delete_comment(comment_id).await?;

        Ok(())
    }
}
